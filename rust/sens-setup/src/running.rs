use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, WPARAM};
use windows::Win32::System::ProcessStatus::EnumProcesses;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE, QueryFullProcessImageNameW,
    TerminateProcess,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, GW_OWNER, GWL_EXSTYLE, GetWindow, GetWindowLongW, GetWindowThreadProcessId, HWND_MESSAGE, IsWindowVisible,
    PostMessageW, RegisterWindowMessageW, WM_CLOSE, WS_EX_TOOLWINDOW,
};
use windows::core::{BOOL, PCWSTR, PWSTR, w};

use crate::language::said;
use crate::progress::{self, CANCELLED, Report, Step};

const SHARING_VIOLATION: i32 = 32;
const PATIENCE: Duration = Duration::from_secs(30);
const GRACE: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(250);
const GLANCE: Duration = Duration::from_millis(80);
const LISTENER: PCWSTR = w!("SensListener");
const QUIT: PCWSTR = w!("SensQuit");

pub enum Closing<'a> {
    Ask(&'a dyn Fn()),
    Wait(&'a dyn Fn()),
    Force,
}

pub fn is_open(app: &Path) -> bool {
    match OpenOptions::new().write(true).open(app) {
        Ok(_) => false,
        Err(error) => error.raw_os_error() == Some(SHARING_VIOLATION),
    }
}

pub fn settle(app: &Path, closing: &Closing, cancel: &AtomicBool, report: Report) -> Result<(), String> {
    if !is_open(app) {
        return Ok(());
    }
    report(Step::Close, 0.06, &progress::waiting());
    let closed = match closing {
        Closing::Ask(ask) => {
            closed_unseen(app, report) || {
                ask();
                wait(app, None, cancel)?
            }
        }
        Closing::Wait(ask) => {
            wait(app, Some(GRACE), cancel)?
                || closed_unseen(app, report)
                || wait(app, Some(PATIENCE - GRACE), cancel)?
                || {
                    ask();
                    wait(app, None, cancel)?
                }
        }
        Closing::Force => close(app, false) || close(app, true),
    };
    if !closed {
        return Err(said!(
            en: "Sens is still open and couldn’t be closed",
            es: "Sens sigue abierta y no se pudo cerrar",
            fr: "Sens est toujours ouvert et n’a pas pu être fermé",
            de: "Sens ist noch geöffnet und konnte nicht geschlossen werden",
            ja: "Sens が開いたままで、閉じられませんでした",
            zh: "Sens 仍在运行，无法关闭",
        ));
    }
    report(Step::Close, 0.09, &progress::closed());
    Ok(())
}

pub fn close(app: &Path, force: bool) -> bool {
    shut(app, &processes_of(app), force)
}

fn closed_unseen(app: &Path, report: Report) -> bool {
    let ours = processes_of(app);
    if ours.is_empty() || !main_windows(&ours).is_empty() {
        return false;
    }
    report(Step::Close, 0.06, &progress::unseen());
    shut(app, &ours, true)
}

fn shut(app: &Path, ours: &BTreeSet<u32>, force: bool) -> bool {
    if asked_to_quit(ours) && gone(app) {
        return true;
    }
    for window in main_windows(ours) {
        unsafe {
            let _ = PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
    if force {
        ours.iter().for_each(|process| terminate(*process));
    }
    gone(app)
}

fn gone(app: &Path) -> bool {
    wait(app, Some(GRACE), &AtomicBool::new(false)).unwrap_or(false)
}

fn asked_to_quit(ours: &BTreeSet<u32>) -> bool {
    let quit = unsafe { RegisterWindowMessageW(QUIT) };
    listeners()
        .into_iter()
        .filter(|window| ours.contains(&process_of(*window)))
        .filter(|window| unsafe { PostMessageW(Some(*window), quit, WPARAM(0), LPARAM(0)) }.is_ok())
        .count()
        > 0
}

fn listeners() -> Vec<HWND> {
    let mut found: Vec<HWND> = Vec::new();
    while let Ok(window) = unsafe { FindWindowExW(Some(HWND_MESSAGE), found.last().copied(), LISTENER, PCWSTR::null()) } {
        found.push(window);
    }
    found
}

fn main_windows(ours: &BTreeSet<u32>) -> Vec<HWND> {
    top_windows()
        .into_iter()
        .filter(|(window, process)| ours.contains(process) && is_main(*window))
        .map(|(window, _)| window)
        .collect()
}

pub fn shown(app: &Path, within: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < within {
        if !main_windows(&processes_of(app)).is_empty() {
            return true;
        }
        thread::sleep(GLANCE);
    }
    false
}

pub fn wait(app: &Path, limit: Option<Duration>, cancel: &AtomicBool) -> Result<bool, String> {
    let started = Instant::now();
    loop {
        if !is_open(app) {
            return Ok(true);
        }
        if cancel.load(Ordering::SeqCst) {
            return Err(CANCELLED.into());
        }
        if limit.is_some_and(|limit| started.elapsed() >= limit) {
            return Ok(false);
        }
        thread::sleep(POLL);
    }
}

fn top_windows() -> Vec<(HWND, u32)> {
    let mut found: Vec<(HWND, u32)> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut found as *mut Vec<(HWND, u32)> as isize));
    }
    found
}

unsafe extern "system" fn collect(window: HWND, found: LPARAM) -> BOOL {
    let found = unsafe { &mut *(found.0 as *mut Vec<(HWND, u32)>) };
    found.push((window, process_of(window)));
    BOOL(1)
}

fn process_of(window: HWND) -> u32 {
    let mut process = 0u32;
    unsafe { GetWindowThreadProcessId(window, Some(&mut process)) };
    process
}

fn is_main(window: HWND) -> bool {
    unsafe {
        IsWindowVisible(window).as_bool()
            && GetWindow(window, GW_OWNER).is_err()
            && (GetWindowLongW(window, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0) == 0
    }
}

fn processes_of(app: &Path) -> BTreeSet<u32> {
    let wanted = app.to_string_lossy().to_lowercase();
    running()
        .into_iter()
        .filter(|process| image_of(*process).is_some_and(|image| image.to_lowercase() == wanted))
        .collect()
}

fn running() -> Vec<u32> {
    let mut ids = vec![0u32; 1024];
    loop {
        let mut filled = 0u32;
        let room = (ids.len() * size_of::<u32>()) as u32;
        if unsafe { EnumProcesses(ids.as_mut_ptr(), room, &mut filled) }.is_err() {
            return Vec::new();
        }
        let count = filled as usize / size_of::<u32>();
        if count < ids.len() {
            ids.truncate(count);
            return ids;
        }
        ids.resize(ids.len() * 2, 0);
    }
}

fn image_of(process: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process).ok()?;
        let mut buffer = [0u16; 1024];
        let mut length = buffer.len() as u32;
        let read = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut length);
        let _ = CloseHandle(handle);
        read.ok()?;
        Some(String::from_utf16_lossy(&buffer[..length as usize]))
    }
}

fn terminate(process: u32) {
    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, process) {
            let _ = TerminateProcess(handle, 1);
            let _ = CloseHandle(handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::process::CommandExt;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;

    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, MSG, PM_REMOVE, PeekMessageW, RegisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW,
    };

    use super::*;
    use crate::system::CREATE_NO_WINDOW;

    unsafe extern "system" fn quiet(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }

    fn listening() -> HWND {
        let class = WNDCLASSW {
            lpfnWndProc: Some(quiet),
            lpszClassName: LISTENER,
            ..Default::default()
        };
        unsafe {
            RegisterClassW(&class);
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                LISTENER,
                PCWSTR::null(),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
            .unwrap()
        }
    }

    fn heard_quit(listener: HWND) -> bool {
        let quit = unsafe { RegisterWindowMessageW(QUIT) };
        let mut message = MSG::default();
        unsafe { PeekMessageW(&mut message, Some(listener), quit, quit, PM_REMOVE) }.as_bool()
    }

    fn listens(process: u32) -> bool {
        let started = Instant::now();
        while started.elapsed() < GRACE {
            if listeners().into_iter().any(|window| process_of(window) == process) {
                return true;
            }
            thread::sleep(GLANCE);
        }
        false
    }

    fn stand_in(app: &Path, test: &str) -> Child {
        fs::copy(std::env::current_exe().unwrap(), app).unwrap();
        Command::new(app)
            .args([&format!("running::tests::{test}"), "--exact", "--ignored"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .unwrap()
    }

    fn scratch_app(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sens-setup-running-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let app = dir.join("sens-app.exe");
        fs::write(&app, b"app").unwrap();
        app
    }

    fn held(app: &Path) -> File {
        OpenOptions::new().read(true).share_mode(1).open(app).unwrap()
    }

    #[test]
    fn an_exe_nobody_holds_is_not_open() {
        let app = scratch_app("free");

        assert!(!is_open(&app));
        assert!(!is_open(&app.with_file_name("missing.exe")));
        let _ = fs::remove_dir_all(app.parent().unwrap());
    }

    #[test]
    fn an_exe_held_without_write_sharing_counts_as_open() {
        let app = scratch_app("held");
        let lock = held(&app);

        assert!(is_open(&app));
        drop(lock);
        assert!(!is_open(&app));
        let _ = fs::remove_dir_all(app.parent().unwrap());
    }

    #[test]
    fn the_close_step_asks_once_and_goes_on_when_the_app_lets_go() {
        let app = scratch_app("asks");
        let lock = held(&app);
        let asked = AtomicUsize::new(0);
        let ask = || {
            asked.fetch_add(1, Ordering::SeqCst);
        };
        let lines = Mutex::new(Vec::new());
        let report = |_: Step, _: f64, line: &str| lines.lock().unwrap().push(line.to_string());

        let settled = thread::scope(|scope| {
            scope.spawn(move || {
                thread::sleep(Duration::from_millis(600));
                drop(lock);
            });
            settle(&app, &Closing::Ask(&ask), &AtomicBool::new(false), &report)
        });

        assert_eq!(settled, Ok(()));
        assert_eq!(asked.load(Ordering::SeqCst), 1);
        assert_eq!(*lines.lock().unwrap(), [progress::waiting(), progress::closed()]);
        let _ = fs::remove_dir_all(app.parent().unwrap());
    }

    #[test]
    fn an_update_waits_quietly_before_asking() {
        let app = scratch_app("waits");
        let lock = held(&app);
        let asked = AtomicUsize::new(0);
        let ask = || {
            asked.fetch_add(1, Ordering::SeqCst);
        };

        let settled = thread::scope(|scope| {
            scope.spawn(move || {
                thread::sleep(Duration::from_millis(600));
                drop(lock);
            });
            settle(&app, &Closing::Wait(&ask), &AtomicBool::new(false), &|_, _, _| {})
        });

        assert_eq!(settled, Ok(()));
        assert_eq!(asked.load(Ordering::SeqCst), 0);
        let _ = fs::remove_dir_all(app.parent().unwrap());
    }

    #[test]
    fn cancelling_while_waiting_for_the_app_rejects_as_cancelled() {
        let app = scratch_app("cancelled");
        let lock = held(&app);
        let cancel = AtomicBool::new(false);

        let settled = thread::scope(|scope| {
            scope.spawn(|| {
                thread::sleep(Duration::from_millis(400));
                cancel.store(true, Ordering::SeqCst);
            });
            settle(&app, &Closing::Ask(&|| {}), &cancel, &|_, _, _| {})
        });

        assert_eq!(settled, Err(CANCELLED.to_string()));
        drop(lock);
        let _ = fs::remove_dir_all(app.parent().unwrap());
    }

    #[test]
    #[ignore]
    fn sleeps_as_a_stand_in_for_sens() {
        thread::sleep(Duration::from_secs(60));
    }

    #[test]
    #[ignore]
    fn listens_as_a_stand_in_for_sens() {
        let listener = listening();
        let started = Instant::now();
        while !heard_quit(listener) {
            assert!(started.elapsed() < Duration::from_secs(60));
            thread::sleep(POLL);
        }
    }

    #[test]
    fn a_sens_left_running_without_a_window_is_closed_without_asking() {
        let app = scratch_app("unseen");
        let mut stand_in = stand_in(&app, "sleeps_as_a_stand_in_for_sens");
        let asked = AtomicUsize::new(0);
        let ask = || {
            asked.fetch_add(1, Ordering::SeqCst);
        };
        let lines = Mutex::new(Vec::new());
        let report = |_: Step, _: f64, line: &str| lines.lock().unwrap().push(line.to_string());

        assert!(is_open(&app));
        let settled = settle(&app, &Closing::Ask(&ask), &AtomicBool::new(false), &report);

        assert_eq!(settled, Ok(()));
        assert_eq!(asked.load(Ordering::SeqCst), 0);
        assert_eq!(*lines.lock().unwrap(), [progress::waiting(), progress::unseen(), progress::closed()]);
        assert!(stand_in.try_wait().unwrap().is_some());
        let _ = fs::remove_dir_all(app.parent().unwrap());
    }

    #[test]
    fn a_sens_hidden_in_the_tray_is_asked_to_quit_before_anything_is_forced() {
        let app = scratch_app("tray");
        let mut stand_in = stand_in(&app, "listens_as_a_stand_in_for_sens");
        let asked = AtomicUsize::new(0);
        let ask = || {
            asked.fetch_add(1, Ordering::SeqCst);
        };

        assert!(listens(stand_in.id()));
        let settled = settle(&app, &Closing::Ask(&ask), &AtomicBool::new(false), &|_, _, _| {});

        assert_eq!(settled, Ok(()));
        assert_eq!(asked.load(Ordering::SeqCst), 0);
        assert!(stand_in.wait().unwrap().success());
        let _ = fs::remove_dir_all(app.parent().unwrap());
    }

    #[test]
    fn only_the_listener_of_the_sens_being_closed_is_asked_to_quit() {
        let us = BTreeSet::from([std::process::id()]);

        assert!(!asked_to_quit(&us));
        let listener = listening();
        assert!(!asked_to_quit(&BTreeSet::new()));
        assert!(!heard_quit(listener));
        assert!(asked_to_quit(&us));
        assert!(heard_quit(listener));
        unsafe { DestroyWindow(listener) }.unwrap();
    }

    #[test]
    fn an_app_nobody_runs_is_never_seen_and_the_wait_ends() {
        let app = scratch_app("never-shown");
        let started = Instant::now();

        assert!(!shown(&app, Duration::from_millis(300)));
        assert!(started.elapsed() < Duration::from_secs(2));
        let _ = fs::remove_dir_all(app.parent().unwrap());
    }

    #[test]
    fn closing_an_app_nobody_runs_finds_it_already_gone() {
        let app = scratch_app("nobody");

        assert!(close(&app, false));
        let _ = fs::remove_dir_all(app.parent().unwrap());
    }
}
