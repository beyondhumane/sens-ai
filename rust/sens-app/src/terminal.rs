use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};
use sens_agent::process::Family;
use sens_agent::said;
use serde::Serialize;

const CHUNK: usize = 16 * 1024;
const ENDING: Duration = Duration::from_secs(3);
const RUN_COLS: u16 = 120;
const RUN_ROWS: u16 = 30;
const HISTORY: usize = 10_000;
const ASKS_WHERE: &str = "\x1b[6n";
const BACKLOG: usize = 512 * 1024;

fn broken() -> String {
    said!(
        en: "the list of terminals broke",
        es: "el registro de terminales se rompió",
        fr: "la liste des terminaux est corrompue",
        de: "die Liste der Terminals ist beschädigt",
        ja: "ターミナルの一覧が壊れました",
        zh: "终端列表已损坏",
    )
}

fn gone() -> String {
    said!(
        en: "that terminal is already closed",
        es: "esa terminal ya se cerró",
        fr: "ce terminal est déjà fermé",
        de: "dieses Terminal ist bereits geschlossen",
        ja: "そのターミナルはすでに閉じています",
        zh: "该终端已关闭",
    )
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Heard {
    Out { id: u32, data: String },
    Ended { id: u32, code: Option<u32> },
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Opened {
    pub id: u32,
    pub shell: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Listed {
    pub id: u32,
    pub cwd: PathBuf,
    pub shell: String,
    pub title: Option<String>,
    pub running: bool,
}

pub struct Ran {
    pub code: Option<Option<u32>>,
    pub text: String,
}

type Input = Arc<Mutex<Box<dyn Write + Send>>>;

struct Console {
    cwd: PathBuf,
    master: Box<dyn MasterPty + Send>,
    input: Input,
    killer: Box<dyn ChildKiller + Send + Sync>,
    family: Option<Family>,
}

impl Console {
    fn end(&mut self) {
        match &self.family {
            Some(family) => family.end(),
            None => {
                let _ = self.killer.kill();
            }
        }
    }
}

struct Screen {
    parser: vt100::Parser,
    backlog: String,
    shown: bool,
}

struct Capture {
    screen: Mutex<Screen>,
    ended: Mutex<Option<Option<u32>>>,
    done: Condvar,
}

impl Capture {
    fn new() -> Capture {
        let screen = Screen { parser: vt100::Parser::new(RUN_ROWS, RUN_COLS, HISTORY), backlog: String::new(), shown: false };
        Capture { screen: Mutex::new(screen), ended: Mutex::new(None), done: Condvar::new() }
    }

    fn take(&self, text: &str, input: &Input) -> bool {
        let Ok(mut screen) = self.screen.lock() else {
            return true;
        };
        screen.parser.process(text.as_bytes());
        if !screen.shown {
            screen.backlog.push_str(text);
            let over = screen.backlog.len().saturating_sub(BACKLOG);
            let cut = (over..=screen.backlog.len()).find(|at| screen.backlog.is_char_boundary(*at)).unwrap_or_default();
            screen.backlog.drain(..cut);
        }
        if text.contains(ASKS_WHERE) {
            let (row, col) = screen.parser.screen().cursor_position();
            if let Ok(mut input) = input.lock() {
                let _ = input.write_all(format!("\x1b[{};{}R", row + 1, col + 1).as_bytes()).and_then(|()| input.flush());
            }
        }
        screen.shown
    }

    fn show(&self) -> String {
        self.screen.lock().map(|mut screen| {
            screen.shown = true;
            std::mem::take(&mut screen.backlog)
        })
        .unwrap_or_default()
    }

    fn shown(&self) -> bool {
        self.screen.lock().is_ok_and(|screen| screen.shown)
    }

    fn finish(&self, code: Option<u32>) {
        if let Ok(mut ended) = self.ended.lock() {
            *ended = Some(code);
        }
        self.done.notify_all();
    }

    fn wait(&self, patience: Duration) -> Option<Option<u32>> {
        let ended = self.ended.lock().ok()?;
        let (ended, _) = self.done.wait_timeout_while(ended, patience, |ended| ended.is_none()).ok()?;
        *ended
    }

    fn lines(&self) -> Vec<String> {
        self.screen.lock().map(|mut screen| written(&mut screen.parser)).unwrap_or_default()
    }
}

struct Known {
    cwd: PathBuf,
    shell: String,
    title: Option<String>,
    capture: Option<Arc<Capture>>,
}

#[derive(Default)]
pub struct Consoles {
    open: Arc<Mutex<HashMap<u32, Console>>>,
    known: Mutex<HashMap<u32, Known>>,
    made: AtomicU32,
}

impl Consoles {
    pub fn open(&self, root: &Path, cols: u16, rows: u16, tell: impl Fn(Heard) + Send + 'static) -> Result<Opened, String> {
        let cwd = folder(root);
        let (command, shell) = shell(&cwd);
        self.spawn(Known { cwd, shell, title: None, capture: None }, command, size(cols, rows), tell)
    }

    pub fn run(&self, cwd: &Path, command: &str, title: &str, tell: impl Fn(Heard) + Send + 'static) -> Result<Opened, String> {
        let cwd = folder(cwd);
        let (built, shell) = running(&cwd, command);
        let known = Known { cwd, shell, title: Some(title.to_string()), capture: Some(Arc::new(Capture::new())) };
        self.spawn(known, built, size(RUN_COLS, RUN_ROWS), tell)
    }

    fn spawn(&self, known: Known, command: CommandBuilder, size: PtySize, tell: impl Fn(Heard) + Send + 'static) -> Result<Opened, String> {
        let pair = native_pty_system().openpty(size).map_err(failed)?;
        let mut child = pair.slave.spawn_command(command).map_err(failed)?;
        drop(pair.slave);
        let reader = pair.master.try_clone_reader().map_err(failed)?;
        let writer = pair.master.take_writer().map_err(failed)?;

        let id = self.made.fetch_add(1, Ordering::SeqCst) + 1;
        let input: Input = Arc::new(Mutex::new(writer));
        let console = Console {
            cwd: known.cwd.clone(),
            master: pair.master,
            input: input.clone(),
            killer: child.clone_killer(),
            family: family(child.as_ref()),
        };
        let opened = Opened { id, shell: known.shell.clone() };
        let capture = known.capture.clone();
        self.open.lock().map_err(|_| broken())?.insert(id, console);
        self.known.lock().map_err(|_| broken())?.insert(id, known);

        let (ended, code) = mpsc::channel();
        let open = self.open.clone();
        thread::spawn(move || {
            let status = child.wait().ok().map(|status| status.exit_code());
            let gone = open.lock().ok().and_then(|mut open| open.remove(&id));
            drop(gone);
            let _ = ended.send(status);
        });
        let watched = capture.map(|capture| (capture, input));
        thread::spawn(move || relay(id, reader, code, watched, tell));
        Ok(opened)
    }

    pub fn wait(&self, id: u32, patience: Duration) -> Option<Ran> {
        let capture = self.capture(id)?;
        let code = capture.wait(patience);
        Some(Ran { code, text: capture.lines().join("\n") })
    }

    pub fn last_lines(&self, id: u32, count: usize) -> Option<String> {
        let lines = self.capture(id)?.lines();
        Some(lines[lines.len().saturating_sub(count)..].join("\n"))
    }

    pub fn show(&self, id: u32) -> Option<String> {
        Some(self.capture(id)?.show())
    }

    fn capture(&self, id: u32) -> Option<Arc<Capture>> {
        self.known.lock().ok()?.get(&id)?.capture.clone()
    }

    pub fn listed(&self) -> Vec<Listed> {
        let running: Vec<u32> = self.open.lock().map(|open| open.keys().copied().collect()).unwrap_or_default();
        let mut listed: Vec<Listed> = self
            .known
            .lock()
            .map(|known| {
                known
                    .iter()
                    .map(|(id, one)| Listed { id: *id, cwd: one.cwd.clone(), shell: one.shell.clone(), title: one.title.clone(), running: running.contains(id) })
                    .collect()
            })
            .unwrap_or_default();
        listed.sort_by_key(|one| one.id);
        listed
    }

    pub fn write(&self, id: u32, data: &str) -> Result<(), String> {
        let input = self.open.lock().map_err(|_| broken())?.get(&id).ok_or_else(gone)?.input.clone();
        let mut input = input.lock().map_err(|_| broken())?;
        input.write_all(data.as_bytes()).and_then(|()| input.flush()).map_err(|error| {
            said!(
                en: "the terminal won’t take what you type: {error}",
                es: "la terminal no acepta lo que escribes: {error}",
                fr: "le terminal n’accepte pas ce que vous saisissez : {error}",
                de: "das Terminal nimmt deine Eingabe nicht an: {error}",
                ja: "ターミナルが入力を受け付けません: {error}",
                zh: "终端不接受你的输入：{error}",
            )
        })
    }

    pub fn resize(&self, id: u32, cols: u16, rows: u16) -> Result<(), String> {
        let open = self.open.lock().map_err(|_| broken())?;
        let console = open.get(&id).ok_or_else(gone)?;
        console.master.resize(size(cols, rows)).map_err(|error| {
            said!(
                en: "couldn’t resize the terminal: {error}",
                es: "no pude ajustar la terminal: {error}",
                fr: "impossible de redimensionner le terminal : {error}",
                de: "die Größe des Terminals konnte nicht angepasst werden: {error}",
                ja: "ターミナルのサイズを変更できませんでした: {error}",
                zh: "无法调整终端大小：{error}",
            )
        })?;
        if let Some(capture) = self.capture(id)
            && let Ok(mut screen) = capture.screen.lock()
        {
            screen.parser.screen_mut().set_size(rows.max(1), cols.max(2));
        }
        Ok(())
    }

    pub fn stop(&self, id: u32) -> Result<(), String> {
        if let Some(console) = self.open.lock().map_err(|_| broken())?.get_mut(&id) {
            console.end();
        }
        Ok(())
    }

    pub fn close(&self, id: u32) -> Result<(), String> {
        self.stop(id)?;
        self.known.lock().map_err(|_| broken())?.remove(&id);
        Ok(())
    }

    pub fn close_within(&self, folder: &Path) {
        if let Ok(mut known) = self.known.lock() {
            known.retain(|_, one| !one.cwd.starts_with(folder));
        }
        let closing: Vec<u32> = match self.open.lock() {
            Ok(mut open) => open
                .iter_mut()
                .filter(|(_, console)| console.cwd.starts_with(folder))
                .map(|(id, console)| {
                    console.end();
                    *id
                })
                .collect(),
            Err(_) => return,
        };
        let until = Instant::now() + ENDING;
        while Instant::now() < until && self.open.lock().is_ok_and(|open| closing.iter().any(|id| open.contains_key(id))) {
            thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn shutdown(&self) {
        if let Ok(mut open) = self.open.lock() {
            for (_, mut console) in open.drain() {
                console.end();
            }
        }
    }
}

fn written(parser: &mut vt100::Parser) -> Vec<String> {
    let (_, cols) = parser.screen().size();
    parser.screen_mut().set_scrollback(usize::MAX);
    let back = parser.screen().scrollback();
    let mut rows = Vec::new();
    for offset in (1..=back).rev() {
        parser.screen_mut().set_scrollback(offset);
        let screen = parser.screen();
        rows.extend(screen.rows(0, cols).next().map(|row| (row, screen.row_wrapped(0))));
    }
    parser.screen_mut().set_scrollback(0);
    let screen = parser.screen();
    rows.extend(screen.rows(0, cols).enumerate().map(|(at, row)| (row, screen.row_wrapped(at as u16))));
    let mut lines: Vec<String> = Vec::new();
    let mut joining = false;
    for (row, wrapped) in rows {
        match (joining, lines.last_mut()) {
            (true, Some(last)) => last.push_str(&row),
            _ => lines.push(row),
        }
        joining = wrapped;
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    lines
}

fn running(cwd: &Path, command: &str) -> (CommandBuilder, String) {
    let (mut built, name) = shell(cwd);
    if cfg!(windows) {
        built.args(["-NoProfile", "-EncodedCommand", &encoded(command)]);
    } else {
        built.args(["-c", command]);
    }
    (built, name)
}

fn encoded(command: &str) -> String {
    let script = format!("[Console]::OutputEncoding = [Text.Encoding]::UTF8\n{command}\nif ($?) {{ exit 0 }}\nif ($LASTEXITCODE) {{ exit $LASTEXITCODE }}\nexit 1");
    let wide: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    STANDARD.encode(wide)
}

fn failed(error: impl std::fmt::Display) -> String {
    said!(
        en: "couldn’t open the terminal: {error}",
        es: "no pude abrir la terminal: {error}",
        fr: "impossible d’ouvrir le terminal : {error}",
        de: "das Terminal konnte nicht geöffnet werden: {error}",
        ja: "ターミナルを開けませんでした: {error}",
        zh: "无法打开终端：{error}",
    )
}

fn size(cols: u16, rows: u16) -> PtySize {
    PtySize {
        cols: cols.max(2),
        rows: rows.max(1),
        ..PtySize::default()
    }
}

fn shell(cwd: &Path) -> (CommandBuilder, String) {
    let program = program();
    let name = program
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut command = CommandBuilder::new(&program);
    if cfg!(windows) {
        command.arg("-NoLogo");
    } else {
        command.arg("-l");
        command.env("TERM", "xterm-256color");
    }
    command.env("COLORTERM", "truecolor");
    command.cwd(cwd);
    (command, name)
}

fn program() -> PathBuf {
    if cfg!(windows) {
        return on_path("pwsh.exe").unwrap_or_else(|| PathBuf::from("powershell.exe"));
    }
    std::env::var_os("SHELL")
        .filter(|shell| !shell.is_empty())
        .map_or_else(|| PathBuf::from("/bin/sh"), PathBuf::from)
}

fn on_path(name: &str) -> Option<PathBuf> {
    let listed = std::env::var_os("PATH")?;
    std::env::split_paths(&listed)
        .map(|folder| folder.join(name))
        .find(|candidate| candidate.is_absolute() && candidate.is_file())
}

fn folder(root: &Path) -> PathBuf {
    if root.is_dir() {
        return root.to_path_buf();
    }
    std::env::home_dir().unwrap_or_else(std::env::temp_dir)
}

#[cfg(windows)]
fn family(child: &(dyn portable_pty::Child + Send + Sync)) -> Option<Family> {
    child.as_raw_handle().map(Family::of)
}

#[cfg(not(windows))]
fn family(_child: &(dyn portable_pty::Child + Send + Sync)) -> Option<Family> {
    None
}

fn relay(id: u32, mut reader: Box<dyn Read + Send>, code: mpsc::Receiver<Option<u32>>, watched: Option<(Arc<Capture>, Input)>, tell: impl Fn(Heard)) {
    let mut buffer = vec![0; CHUNK];
    let mut pending = Vec::new();
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                pending.extend_from_slice(&buffer[..read]);
                let data = whole(&mut pending);
                let shown = watched.as_ref().is_none_or(|(capture, input)| capture.take(&data, input));
                if shown && !data.is_empty() {
                    tell(Heard::Out { id, data });
                }
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(_) => break,
        }
    }
    if !pending.is_empty() {
        let data = String::from_utf8_lossy(&pending).into_owned();
        if watched.as_ref().is_none_or(|(capture, input)| capture.take(&data, input)) {
            tell(Heard::Out { id, data });
        }
    }
    let ended = code.recv().ok().flatten();
    if let Some((capture, _)) = &watched {
        capture.finish(ended);
    }
    if watched.as_ref().is_none_or(|(capture, _)| capture.shown()) {
        tell(Heard::Ended { id, code: ended });
    }
}

fn whole(pending: &mut Vec<u8>) -> String {
    let rest = pending.split_off(complete(pending));
    let text = String::from_utf8_lossy(pending).into_owned();
    *pending = rest;
    text
}

fn complete(bytes: &[u8]) -> usize {
    let length = bytes.len();
    for back in 1..=length.min(3) {
        let byte = bytes[length - back];
        if byte & 0xC0 == 0x80 {
            continue;
        }
        let needs = match byte {
            0xC0..=0xDF => 2,
            0xE0..=0xEF => 3,
            0xF0..=0xF7 => 4,
            _ => 1,
        };
        return if needs > back { length - back } else { length };
    }
    length
}

#[cfg(test)]
mod tests {
    use super::*;
    use sens_agent::language::{Language, speaking};
    use std::time::{Duration, Instant};

    const ASKS_WHERE_TEXT: &str = "\x1b[6n";

    #[test]
    fn a_character_cut_in_half_waits_for_its_other_half() {
        assert_eq!(complete(b"ls\r\n"), 4);
        assert_eq!(complete(&[b'a', 0xC3]), 1);
        assert_eq!(complete(&[b'a', 0xC3, 0xA9]), 3);
        assert_eq!(complete(&[0xE2, 0x82]), 0);
        assert_eq!(complete(&[b'x', 0xE2, 0x82, 0xAC]), 4);
        assert_eq!(complete(&[0xF0, 0x9F, 0x98]), 0);
        assert_eq!(complete(&[0x80, 0x80, 0x80, 0x80]), 4);
        assert_eq!(complete(&[]), 0);
    }

    #[test]
    fn what_is_read_goes_out_whole_and_the_rest_waits() {
        let mut pending = vec![b'o', b'k', 0xE2, 0x82];
        assert_eq!(whole(&mut pending), "ok");
        assert_eq!(pending, vec![0xE2, 0x82]);
        pending.push(0xAC);
        assert_eq!(whole(&mut pending), "€");
        assert!(pending.is_empty());
    }

    #[test]
    fn the_screen_reads_back_whole_lines_beyond_its_height() {
        let mut parser = vt100::Parser::new(3, 10, 100);
        parser.process(b"uno\r\ndos\r\n0123456789abc\r\ncuatro\r\ncinco\r\n\r\n");
        assert_eq!(written(&mut parser), ["uno", "dos", "0123456789abc", "cuatro", "cinco"]);
        let mut redrawn = vt100::Parser::new(3, 20, 100);
        redrawn.process(b"bajando 10%\rbajando 100%\r\nlisto");
        assert_eq!(written(&mut redrawn), ["bajando 100%", "listo"]);
    }

    #[test]
    fn a_command_travels_whole_and_says_how_it_ended() {
        let bytes = STANDARD.decode(encoded("Write-Output \"año\"")).unwrap();
        let wide: Vec<u16> = bytes.chunks(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
        let script = String::from_utf16(&wide).unwrap();
        assert!(script.contains("\nWrite-Output \"año\"\n"));
        assert!(script.ends_with("if ($LASTEXITCODE) { exit $LASTEXITCODE }\nexit 1"));
    }

    #[test]
    fn a_size_never_reaches_zero() {
        let tiny = size(0, 0);
        assert_eq!((tiny.cols, tiny.rows), (2, 1));
        let fair = size(120, 30);
        assert_eq!((fair.cols, fair.rows), (120, 30));
    }

    #[test]
    fn a_folder_that_is_not_there_opens_at_home() {
        let here = std::env::temp_dir();
        assert_eq!(folder(&here), here);
        assert_ne!(folder(Path::new("Z:/sens/no-such-folder")), PathBuf::from("Z:/sens/no-such-folder"));
    }

    #[test]
    fn the_events_read_as_the_interface_expects() {
        let out = serde_json::to_value(Heard::Out { id: 3, data: "hola".into() }).unwrap();
        assert_eq!(out, serde_json::json!({ "kind": "out", "id": 3, "data": "hola" }));
        let ended = serde_json::to_value(Heard::Ended { id: 3, code: Some(0) }).unwrap();
        assert_eq!(ended, serde_json::json!({ "kind": "ended", "id": 3, "code": 0 }));
    }

    fn heard_until(heard: &mpsc::Receiver<Heard>, done: impl Fn(&[Heard]) -> bool) -> Vec<Heard> {
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut all = Vec::new();
        while !done(&all) && Instant::now() < deadline {
            if let Ok(one) = heard.recv_timeout(Duration::from_millis(200)) {
                all.push(one);
            }
        }
        all
    }

    fn said(all: &[Heard]) -> String {
        all.iter()
            .filter_map(|one| match one {
                Heard::Out { data, .. } => Some(data.as_str()),
                Heard::Ended { .. } => None,
            })
            .collect()
    }

    fn ended(all: &[Heard]) -> bool {
        all.iter().any(|one| matches!(one, Heard::Ended { .. }))
    }

    #[test]
    #[ignore = "opens a real shell"]
    fn closing_a_folder_ends_the_shells_started_in_it_and_no_other() {
        let base = std::env::temp_dir().join("sens-terminal-within");
        let inside = base.join("worktree");
        std::fs::create_dir_all(&inside).unwrap();
        let consoles = Consoles::default();
        let (tell, heard) = mpsc::channel();
        let answer = move |one| {
            let _ = tell.send(one);
        };
        let within = consoles.open(&inside, 80, 24, answer.clone()).unwrap();
        let outside = consoles.open(&std::env::temp_dir(), 80, 24, answer).unwrap();

        consoles.close_within(&base);

        assert_eq!(consoles.write(within.id, "x"), Err(gone()));
        if cfg!(windows) {
            let asked = heard_until(&heard, |all| said(all).contains(ASKS_WHERE_TEXT));
            assert!(!asked.is_empty());
            consoles.write(outside.id, "\x1b[1;1R").unwrap();
        }
        assert!(consoles.write(outside.id, "\r").is_ok());
        consoles.shutdown();
    }

    #[test]
    #[ignore = "opens a real shell"]
    fn a_shell_answers_and_ends_when_closed() {
        let consoles = Consoles::default();
        let (tell, heard) = mpsc::channel();
        let opened = consoles
            .open(&std::env::temp_dir(), 80, 24, move |one| {
                let _ = tell.send(one);
            })
            .unwrap();
        consoles.resize(opened.id, 100, 30).unwrap();
        let mut all = Vec::new();
        if cfg!(windows) {
            all = heard_until(&heard, |all| said(all).contains(ASKS_WHERE_TEXT));
            consoles.write(opened.id, "\x1b[1;1R").unwrap();
        }
        consoles.write(opened.id, "echo sens-$((20+22))\r").unwrap();
        all.extend(heard_until(&heard, |all| said(all).contains("sens-42")));
        assert!(said(&all).contains("sens-42"), "{:?}", said(&all));

        consoles.close(opened.id).unwrap();
        let rest = heard_until(&heard, ended);
        assert!(ended(&rest));
        assert_eq!(consoles.write(opened.id, "x"), Err(gone()));
    }

    #[test]
    fn a_terminal_already_closed_is_said_in_the_language_spoken() {
        let consoles = Consoles::default();

        assert_eq!(consoles.write(7, "x"), Err("that terminal is already closed".into()));
        assert_eq!(speaking(Language::Es, || consoles.resize(7, 80, 24)), Err("esa terminal ya se cerró".into()));
        assert_eq!(speaking(Language::Fr, || consoles.write(7, "x")), Err("ce terminal est déjà fermé".into()));
    }
}
