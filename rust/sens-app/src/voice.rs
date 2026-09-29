use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use sens_agent::said;
use serde::{Deserialize, Serialize};

use crate::{claude_code, files, store, web};

const FOLDER: &str = "voice";
const MODEL: &str = "ggml-small-q5_1.bin";
const MODEL_URL: &str = "https://github.com/iiTzSenn/Sens/releases/download/whisper-small-q5_1/ggml-small-q5_1.bin";
const MODEL_SHA256: &str = "ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb";
const MODEL_BYTES: u64 = 190_085_487;
const RETIRED: [&str; 1] = ["ggml-base-q5_1.bin"];
const CHOSEN: &str = "voice.json";

const RATE: usize = 16_000;
const FRAME: usize = RATE / 10;
const LEAD: usize = 3;
const PAUSE: usize = 12;
const TAIL_KEPT: usize = 2;
const TOKENS_PER_SECOND: usize = 10;
const TOKENS_SPARE: usize = 16;
const SPOKEN: usize = 4;
const LONGEST: usize = 250;
const IDLE: usize = 600;
const QUIETEST: f32 = 0.0015;
const OVER_FLOOR: f32 = 2.5;
const LOUDEST_GAIN: f32 = 8.0;
const PEAK: f32 = 0.9;
const PROMPT_TAIL: usize = 400;
const TICK: Duration = Duration::from_millis(100);
const GUESS_EVERY: usize = 4;
const GUESS_FROM: usize = 15 * FRAME;
const GUESS_TOKENS: i32 = 64;
const LEAST_CONTEXT: usize = 160;
const SAMPLES_PER_CONTEXT: usize = 320;
const CONTEXT_MARGIN: usize = 64;
const FULL_CONTEXT: usize = 1500;
const PASSBAND: f64 = 6_800.0;
const TAPS: usize = 95;
const DEAF: usize = 15;
const LOOPED: usize = 3;
const HANDS_FREE_IDLE: usize = 40;
const WAKE_PAUSE: usize = 5;
const WAKE_HEARD: usize = 25 * FRAME;
const WAKE_TOKENS: i32 = 12;
const WAKE_THREADS: i32 = 2;
const REOPEN: usize = 30;
const WAKE_PROMPT: &str = "Hey Sens.";
const NAMED_WITHIN: usize = 3;

const GREETINGS: [&str; 12] = ["hey", "hei", "hay", "ey", "ei", "eh", "ehi", "oye", "oie", "hola", "hi", "hello"];
const NAME_SOUNDS: [&str; 5] = ["sen", "cen", "san", "zen", "sin"];
const NAMES: [&str; 12] = ["sens", "sense", "sen", "senz", "cens", "sends", "sence", "sans", "zens", "since", "cents", "sins"];

const HALLUCINATIONS: [&str; 10] = [
    "subtitulos realizados por la comunidad de amaraorg",
    "subtitulos por la comunidad de amaraorg",
    "gracias por ver",
    "gracias por ver el video",
    "gracias por vernos",
    "thank you for watching",
    "thanks for watching",
    "sous-titrage st 501",
    "untertitel der amaraorg-community",
    "ご視聴ありがとうございました",
];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Cause {
    Microphone,
    Silent,
    Model,
    Other,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Refusal {
    pub cause: Cause,
    pub message: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Heard {
    Level { id: u32, level: f32 },
    Guess { id: u32, text: String },
    Phrase { id: u32, text: String },
    Ended { id: u32, refusal: Option<Refusal> },
    Fetching { done: u64, total: u64 },
    Ready,
    Unfetched { message: String },
    Woke,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub ready: bool,
    pub fetching: bool,
    pub bytes: u64,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Microphone {
    pub name: String,
    pub default: bool,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct Chosen {
    microphone: Option<String>,
}

fn refusal(cause: Cause, message: String) -> Refusal {
    Refusal { cause, message }
}

fn no_microphone(error: &str) -> Refusal {
    refusal(
        Cause::Microphone,
        said!(
            en: "Sens can’t listen to the microphone: {error}",
            es: "Sens no puede escuchar el micrófono: {error}",
            fr: "Sens ne peut pas écouter le microphone : {error}",
            de: "Sens kann das Mikrofon nicht hören: {error}",
            ja: "Sens がマイクの音を取得できません: {error}",
            zh: "Sens 无法使用麦克风：{error}",
        ),
    )
}

fn muted() -> Refusal {
    refusal(
        Cause::Silent,
        said!(
            en: "Your microphone is muted in Windows · turn it on with the microphone key or in Settings › System › Sound › Input",
            es: "Tu micrófono está silenciado en Windows · actívalo con la tecla del micrófono o en Configuración › Sistema › Sonido › Entrada",
            fr: "Votre microphone est coupé dans Windows · activez-le avec la touche du microphone ou dans Paramètres › Système › Son › Entrée",
            de: "Dein Mikrofon ist in Windows stummgeschaltet · schalte es mit der Mikrofontaste oder unter Einstellungen › System › Sound › Eingabe ein",
            ja: "Windows でマイクがミュートになっています · マイクキーか、設定 › システム › サウンド › 入力 でオンにしてください",
            zh: "麦克风在 Windows 中已静音 · 请按麦克风键，或在 设置 › 系统 › 声音 › 输入 中打开",
        ),
    )
}

fn deaf() -> Refusal {
    refusal(
        Cause::Silent,
        said!(
            en: "No sound reaches Sens from the microphone · check it isn’t muted, and that Settings › Privacy › Microphone lets desktop apps use it",
            es: "No llega ningún sonido del micrófono · comprueba que no esté silenciado y que Configuración › Privacidad › Micrófono permita usarlo a las aplicaciones de escritorio",
            fr: "Aucun son n’arrive du microphone · vérifiez qu’il n’est pas coupé et que Paramètres › Confidentialité › Microphone autorise les applications de bureau",
            de: "Vom Mikrofon kommt kein Ton an · prüfe, dass es nicht stummgeschaltet ist und Einstellungen › Datenschutz › Mikrofon Desktop-Apps den Zugriff erlaubt",
            ja: "マイクから音が届いていません · ミュートになっていないか、設定 › プライバシー › マイク でデスクトップアプリに許可しているか確認してください",
            zh: "麦克风没有传来任何声音 · 请确认它没有静音，并在 设置 › 隐私 › 麦克风 中允许桌面应用使用",
        ),
    )
}

fn no_model() -> Refusal {
    refusal(
        Cause::Model,
        said!(
            en: "the voice model isn’t on this computer yet",
            es: "el modelo de voz aún no está en este equipo",
            fr: "le modèle vocal n’est pas encore sur cet ordinateur",
            de: "das Sprachmodell ist noch nicht auf diesem Computer",
            ja: "音声モデルがまだこのコンピューターにありません",
            zh: "此电脑上还没有语音模型",
        ),
    )
}

fn unloaded(error: &str) -> Refusal {
    refusal(
        Cause::Model,
        said!(
            en: "the voice model doesn’t load: {error}",
            es: "el modelo de voz no carga: {error}",
            fr: "le modèle vocal ne se charge pas : {error}",
            de: "das Sprachmodell lädt nicht: {error}",
            ja: "音声モデルを読み込めません: {error}",
            zh: "无法加载语音模型：{error}",
        ),
    )
}

fn untranscribed(error: &str) -> Refusal {
    refusal(
        Cause::Other,
        said!(
            en: "what was said couldn’t be transcribed: {error}",
            es: "no pude transcribir lo que dijiste: {error}",
            fr: "impossible de transcrire ce qui a été dit : {error}",
            de: "das Gesagte konnte nicht transkribiert werden: {error}",
            ja: "話した内容を文字にできませんでした: {error}",
            zh: "无法转写所说的内容：{error}",
        ),
    )
}

fn mismatched() -> String {
    said!(
        en: "the voice model download doesn’t match the one Sens expects; it was discarded",
        es: "la descarga del modelo de voz no coincide con la que espera Sens; la he descartado",
        fr: "le téléchargement du modèle vocal ne correspond pas à celui attendu par Sens ; il a été supprimé",
        de: "der Download des Sprachmodells stimmt nicht mit dem erwarteten überein; er wurde verworfen",
        ja: "音声モデルのダウンロードが Sens の想定と一致しないため、破棄しました",
        zh: "语音模型的下载与 Sens 预期的不一致，已丢弃",
    )
}

fn held<T>(slot: &Mutex<T>) -> MutexGuard<'_, T> {
    slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn level(rms: f32) -> f32 {
    if rms <= 0.0 {
        return 0.0;
    }
    ((20.0 * rms.log10() + 60.0) / 50.0).clamp(0.0, 1.0)
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len() as f32).sqrt()
}

pub fn louder(samples: &[f32]) -> Vec<f32> {
    let peak = samples.iter().fold(0f32, |loudest, sample| loudest.max(sample.abs()));
    if peak <= 0.0 {
        return samples.to_vec();
    }
    let gain = (PEAK / peak).min(LOUDEST_GAIN);
    samples.iter().map(|sample| sample * gain).collect()
}

pub fn cleaned(text: &str) -> String {
    let mut kept = String::new();
    let mut depth = 0usize;
    for letter in text.chars() {
        match letter {
            '[' => depth += 1,
            ']' => depth = depth.saturating_sub(1),
            _ if depth == 0 => kept.push(letter),
            _ => {}
        }
    }
    let kept = kept.split_whitespace().collect::<Vec<_>>().join(" ");
    let bare: String = kept
        .to_lowercase()
        .chars()
        .filter(|letter| letter.is_alphanumeric() || letter.is_whitespace() || *letter == '-')
        .collect::<String>()
        .replace(['á', 'à'], "a")
        .replace(['é', 'è'], "e")
        .replace('í', "i")
        .replace(['ó', 'ò'], "o")
        .replace('ú', "u");
    let bare = bare.split_whitespace().collect::<Vec<_>>().join(" ");
    if bare.is_empty() || HALLUCINATIONS.contains(&bare.as_str()) {
        return String::new();
    }
    kept
}

pub fn unrepeated(text: &str) -> String {
    let bare = |sentence: &str| sentence.trim_end_matches(|letter: char| !letter.is_alphanumeric()).to_lowercase();
    let mut sentences = Vec::new();
    let mut current = String::new();
    for letter in text.chars() {
        current.push(letter);
        if matches!(letter, '.' | '!' | '?' | '…' | '。' | '！' | '？') {
            sentences.push(std::mem::take(&mut current));
        }
    }
    sentences.push(current);
    let mut kept: Vec<String> = Vec::new();
    for sentence in sentences.iter().map(|sentence| sentence.trim()).filter(|sentence| !sentence.is_empty()) {
        if kept.last().is_some_and(|last| bare(last).starts_with(&bare(sentence))) {
            continue;
        }
        kept.push(sentence.to_string());
    }
    kept.join(" ")
}

pub fn looping(text: &str) -> bool {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| word.trim_matches(|letter: char| !letter.is_alphanumeric()).to_lowercase())
        .filter(|word| !word.is_empty())
        .collect();
    let mut seen = std::collections::HashMap::new();
    words.windows(3).any(|three| {
        let count = seen.entry(three).or_insert(0);
        *count += 1;
        *count >= LOOPED
    })
}

pub fn woken(text: &str) -> bool {
    let bare: String = text.to_lowercase().chars().map(|letter| if letter.is_alphanumeric() { letter } else { ' ' }).collect();
    let words: Vec<&str> = bare.split_whitespace().collect();
    let called = |greeting: &str, name: &str| GREETINGS.contains(&greeting) && NAMES.contains(&name);
    match words.as_slice() {
        [greeting, name, ..] if called(greeting, name) => true,
        [first, ..] => GREETINGS.iter().any(|greeting| first.strip_prefix(greeting).is_some_and(|name| NAMES.contains(&name))),
        [] => false,
    }
}

pub fn named(text: &str) -> bool {
    let opening: String = text.split_whitespace().take(NAMED_WITHIN).flat_map(|word| word.chars().filter(|letter| letter.is_alphanumeric())).collect::<String>().to_lowercase();
    NAME_SOUNDS.iter().any(|sound| opening.contains(sound))
}

pub fn joined(before: &str, said: &str) -> String {
    match (before.trim(), said.trim()) {
        ("", said) => said.to_string(),
        (before, "") => before.to_string(),
        (before, said) => format!("{before} {said}"),
    }
}

pub fn low_pass(rate: usize) -> Vec<f32> {
    if rate <= RATE {
        return vec![1.0];
    }
    let cutoff = PASSBAND / rate as f64;
    let middle = (TAPS - 1) as f64 / 2.0;
    let taps: Vec<f64> = (0..TAPS)
        .map(|at| {
            let from_middle = at as f64 - middle;
            let sinc = match from_middle == 0.0 {
                true => 2.0 * cutoff,
                false => (2.0 * std::f64::consts::PI * cutoff * from_middle).sin() / (std::f64::consts::PI * from_middle),
            };
            let phase = 2.0 * std::f64::consts::PI * at as f64 / (TAPS - 1) as f64;
            sinc * (0.42 - 0.5 * phase.cos() + 0.08 * (2.0 * phase).cos())
        })
        .collect();
    let gain: f64 = taps.iter().sum();
    taps.iter().map(|tap| (tap / gain) as f32).collect()
}

pub struct Resampler {
    step: f64,
    position: f64,
    previous: f32,
    taps: Vec<f32>,
    history: Vec<f32>,
}

impl Resampler {
    pub fn new(rate: usize) -> Self {
        Self { step: rate as f64 / RATE as f64, position: 0.0, previous: 0.0, taps: low_pass(rate), history: Vec::new() }
    }

    fn filtered(&mut self, input: &[f32]) -> Vec<f32> {
        let reach = self.taps.len() - 1;
        if reach == 0 || input.is_empty() {
            return input.to_vec();
        }
        if self.history.is_empty() {
            self.history = vec![input[0]; reach];
        }
        let mut span = std::mem::take(&mut self.history);
        span.extend_from_slice(input);
        let out = span.windows(self.taps.len()).map(|window| window.iter().zip(&self.taps).map(|(sample, tap)| sample * tap).sum()).collect();
        self.history = span[span.len() - reach..].to_vec();
        out
    }

    pub fn push(&mut self, input: &[f32], out: &mut Vec<f32>) {
        let input = &self.filtered(input);
        let mut at = self.position;
        while at < input.len() as f64 {
            let low = at.floor();
            let index = low as isize - 1;
            let before = if index < 0 { self.previous } else { input[index as usize] };
            let after = input[(low as usize).min(input.len() - 1)];
            let t = (at - low) as f32;
            out.push(before * (1.0 - t) + after * t);
            at += self.step;
        }
        self.position = at - input.len() as f64;
        if let Some(last) = input.last() {
            self.previous = *last;
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Cut {
    Phrase(Vec<f32>),
    Idle,
}

pub struct Segmenter {
    pause: usize,
    rest: usize,
    floor: Option<f32>,
    pending: Vec<f32>,
    lead: Vec<Vec<f32>>,
    phrase: Vec<f32>,
    voiced: usize,
    quiet: usize,
    idle: usize,
    loudness: f32,
}

impl Default for Segmenter {
    fn default() -> Self {
        Self::new(PAUSE, IDLE)
    }
}

impl Segmenter {
    pub fn new(pause: usize, rest: usize) -> Self {
        Self { pause, rest, floor: None, pending: Vec::new(), lead: Vec::new(), phrase: Vec::new(), voiced: 0, quiet: 0, idle: 0, loudness: 0.0 }
    }

    pub fn loudness(&self) -> f32 {
        self.loudness
    }

    fn speaking(&self) -> bool {
        !self.phrase.is_empty()
    }

    pub fn current(&self) -> Option<&[f32]> {
        self.speaking().then_some(self.phrase.as_slice())
    }

    pub fn push(&mut self, samples: &[f32]) -> Vec<Cut> {
        self.pending.extend_from_slice(samples);
        let mut cuts = Vec::new();
        while self.pending.len() >= FRAME {
            let frame: Vec<f32> = self.pending.drain(..FRAME).collect();
            if let Some(cut) = self.frame(frame) {
                cuts.push(cut);
            }
        }
        cuts
    }

    fn frame(&mut self, frame: Vec<f32>) -> Option<Cut> {
        let loud = rms(&frame);
        self.loudness = loud;
        let floor = *self.floor.get_or_insert(loud);
        let voice = loud > QUIETEST && loud > floor * OVER_FLOOR;
        if !voice {
            self.floor = Some(if loud < floor { floor * 0.7 + loud * 0.3 } else { floor * 0.98 + loud * 0.02 });
        }
        if !self.speaking() {
            if !voice {
                self.lead.push(frame);
                if self.lead.len() > LEAD {
                    self.lead.remove(0);
                }
                self.idle += 1;
                if self.idle >= self.rest {
                    self.idle = 0;
                    return Some(Cut::Idle);
                }
                return None;
            }
            self.idle = 0;
            for earlier in self.lead.drain(..) {
                self.phrase.extend(earlier);
            }
        }
        self.phrase.extend(frame);
        if voice {
            self.voiced += 1;
            self.quiet = 0;
        } else {
            self.quiet += 1;
        }
        let long = self.phrase.len() >= LONGEST * FRAME;
        if self.quiet >= self.pause || long {
            return self.close();
        }
        None
    }

    fn close(&mut self) -> Option<Cut> {
        let mut phrase = std::mem::take(&mut self.phrase);
        let silence = self.quiet.saturating_sub(TAIL_KEPT) * FRAME;
        phrase.truncate(phrase.len().saturating_sub(silence));
        let worth = self.voiced >= SPOKEN;
        self.voiced = 0;
        self.quiet = 0;
        worth.then_some(Cut::Phrase(phrase))
    }

    pub fn finish(&mut self) -> Option<Vec<f32>> {
        self.phrase.append(&mut self.pending);
        match self.close() {
            Some(Cut::Phrase(phrase)) => Some(phrase),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Job {
    Guess(Vec<f32>),
    Final(Vec<f32>),
}

pub fn next(queue: &mut VecDeque<Job>) -> Option<Job> {
    while queue.len() > 1 && matches!(queue.front(), Some(Job::Guess(_))) {
        queue.pop_front();
    }
    queue.pop_front()
}

pub fn most_tokens(samples: usize) -> i32 {
    (samples * TOKENS_PER_SECOND / RATE + TOKENS_SPARE) as i32
}

pub fn context(samples: usize) -> i32 {
    (samples / SAMPLES_PER_CONTEXT + CONTEXT_MARGIN).clamp(LEAST_CONTEXT, FULL_CONTEXT) as i32
}

pub fn pick<'a>(chosen: Option<&str>, names: &'a [String]) -> Option<&'a str> {
    let chosen = chosen?;
    names.iter().find(|name| name.as_str() == chosen).map(String::as_str)
}

fn model_path(base: &Path) -> PathBuf {
    base.join(FOLDER).join(MODEL)
}

pub fn model(base: &Path, fetching: bool) -> Model {
    let bytes = std::fs::metadata(model_path(base)).map(|meta| meta.len()).unwrap_or(0);
    Model { ready: bytes == MODEL_BYTES, fetching, bytes: MODEL_BYTES }
}

fn retire(base: &Path) {
    for old in RETIRED {
        let _ = std::fs::remove_file(base.join(FOLDER).join(old));
    }
}

pub fn chosen(base: &Path) -> Option<String> {
    store::stored::<Chosen>(&base.join(CHOSEN)).microphone
}

pub fn choose(base: &Path, microphone: Option<String>) -> Result<(), String> {
    store::store(base, CHOSEN, &Chosen { microphone })
}

fn fetch(base: &Path, tell: &(dyn Fn(Heard) + Send + Sync)) -> Result<(), String> {
    let folder = base.join(FOLDER);
    std::fs::create_dir_all(&folder).map_err(|error| files::uncreated(&folder, error))?;
    let part = folder.join(format!("{MODEL}.part"));
    let shown = std::cell::Cell::new(u64::MAX);
    tell(Heard::Fetching { done: 0, total: MODEL_BYTES });
    let fetched = web::save(MODEL_URL, &part, MODEL_BYTES + web::MEGABYTE, |done| {
        let percent = done * 100 / MODEL_BYTES;
        if shown.replace(percent) != percent {
            tell(Heard::Fetching { done, total: MODEL_BYTES });
        }
    });
    let checked = fetched.and_then(|_| claude_code::digest(&part)).and_then(|digest| match digest == MODEL_SHA256 {
        true => Ok(()),
        false => Err(mismatched()),
    });
    if let Err(error) = checked {
        let _ = std::fs::remove_file(&part);
        return Err(error);
    }
    let path = model_path(base);
    std::fs::rename(&part, &path).map_err(|error| files::unwritten(&path, error))
}

type Tell = Arc<dyn Fn(Heard) + Send + Sync>;

struct Live {
    stop: mpsc::Sender<()>,
    thread: JoinHandle<()>,
}

impl Live {
    fn end(self) {
        let _ = self.stop.send(());
        let _ = self.thread.join();
    }
}

pub struct Hearing {
    chosen: Option<String>,
    model: Option<PathBuf>,
    language: String,
    rest: usize,
}

pub struct Voice {
    base: PathBuf,
    tell: Tell,
    live: Mutex<Option<Live>>,
    ear: Mutex<Option<Live>>,
    wanted: AtomicBool,
    busy: Arc<AtomicBool>,
    fetching: Arc<AtomicBool>,
    made: AtomicU32,
}

impl Voice {
    pub fn new(base: PathBuf, tell: impl Fn(Heard) + Send + Sync + 'static) -> Self {
        Self {
            base,
            tell: Arc::new(tell),
            live: Mutex::new(None),
            ear: Mutex::new(None),
            wanted: AtomicBool::new(false),
            busy: Arc::new(AtomicBool::new(false)),
            fetching: Arc::new(AtomicBool::new(false)),
            made: AtomicU32::new(0),
        }
    }

    pub fn model(&self) -> Model {
        model(&self.base, self.fetching.load(Ordering::SeqCst))
    }

    pub fn prepare(&self) {
        if self.model().ready {
            return retire(&self.base);
        }
        if self.fetching.swap(true, Ordering::SeqCst) {
            return;
        }
        let base = self.base.clone();
        let tell = self.tell.clone();
        let fetching = self.fetching.clone();
        thread::spawn(move || {
            let fetched = fetch(&base, tell.as_ref());
            fetching.store(false, Ordering::SeqCst);
            match fetched {
                Ok(()) => {
                    retire(&base);
                    tell(Heard::Ready)
                }
                Err(message) => tell(Heard::Unfetched { message }),
            }
        });
    }

    pub fn start(&self, language: &str, transcribe: bool, hands_free: bool) -> Result<u32, Refusal> {
        self.stop();
        if transcribe && !self.model().ready {
            self.prepare();
            return Err(no_model());
        }
        let id = self.made.fetch_add(1, Ordering::SeqCst) + 1;
        let (stop, stopped) = mpsc::channel();
        let (ready, readied) = mpsc::channel();
        let hearing = Hearing {
            chosen: chosen(&self.base),
            model: transcribe.then(|| model_path(&self.base)),
            language: language.to_string(),
            rest: if hands_free { HANDS_FREE_IDLE } else { IDLE },
        };
        let tell = self.tell.clone();
        let busy = self.busy.clone();
        busy.store(true, Ordering::SeqCst);
        let thread = thread::spawn(move || {
            engine::session(id, hearing, stopped, ready, tell);
            busy.store(false, Ordering::SeqCst);
        });
        match readied.recv().unwrap_or_else(|_| Err(no_microphone("—"))) {
            Ok(()) => {
                *held(&self.live) = Some(Live { stop, thread });
                Ok(id)
            }
            Err(refusal) => {
                let _ = thread.join();
                Err(refusal)
            }
        }
    }

    pub fn stop(&self) {
        let live = held(&self.live).take();
        if let Some(live) = live {
            live.end();
        }
    }

    pub fn finish(&self) {
        if let Some(live) = held(&self.live).as_ref() {
            let _ = live.stop.send(());
        }
    }

    pub fn waking(&self) -> bool {
        self.wanted.load(Ordering::SeqCst)
    }

    pub fn wake(&self, on: bool) -> Result<(), Refusal> {
        self.wanted.store(on, Ordering::SeqCst);
        self.deafen();
        self.rouse()
    }

    pub fn rouse(&self) -> Result<(), Refusal> {
        let mut ear = held(&self.ear);
        if !self.waking() || ear.is_some() {
            return Ok(());
        }
        if !self.model().ready {
            self.prepare();
            return Ok(());
        }
        let (stop, stopped) = mpsc::channel();
        let (ready, readied) = mpsc::channel();
        let chosen = chosen(&self.base);
        let model = model_path(&self.base);
        let busy = self.busy.clone();
        let tell = self.tell.clone();
        let thread = thread::spawn(move || engine::ear(chosen, model, stopped, ready, busy, tell));
        let heard = readied.recv().unwrap_or_else(|_| Err(unloaded("—")));
        if matches!(&heard, Err(refusal) if refusal.cause == Cause::Model) {
            let _ = thread.join();
            return heard;
        }
        *ear = Some(Live { stop, thread });
        heard
    }

    fn deafen(&self) {
        let ear = held(&self.ear).take();
        if let Some(ear) = ear {
            ear.end();
        }
    }

    pub fn shutdown(&self) {
        self.wanted.store(false, Ordering::SeqCst);
        self.deafen();
        self.stop();
    }
}

#[cfg(windows)]
mod engine {
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
    use std::sync::{Arc, Mutex};

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{FromSample, SizedSample};
    use sens_agent::language::{self, Language};
    use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState};

    use std::collections::VecDeque;

    use super::{
        Cut, DEAF, GUESS_EVERY, GUESS_FROM, GUESS_TOKENS, Hearing, Heard, IDLE, Job, Microphone, PAUSE, PROMPT_TAIL, RATE, REOPEN, Refusal, Resampler, Segmenter, TICK, Tell, WAKE_HEARD,
        WAKE_PAUSE, WAKE_PROMPT, WAKE_THREADS, WAKE_TOKENS, cleaned, context, deaf, held, joined, level, looping, louder, most_tokens, muted, next, no_microphone, pick, unloaded,
        named, unrepeated, untranscribed, woken,
    };

    fn name_of(device: &cpal::Device) -> String {
        device.description().map(|description| description.name().to_string()).unwrap_or_default()
    }

    pub fn microphones() -> Vec<Microphone> {
        let host = cpal::default_host();
        let default = host.default_input_device().map(|device| name_of(&device));
        let Ok(devices) = host.input_devices() else {
            return Vec::new();
        };
        devices
            .map(|device| name_of(&device))
            .filter(|name| !name.is_empty())
            .map(|name| Microphone { default: default.as_deref() == Some(name.as_str()), name })
            .collect()
    }

    fn device(chosen: Option<&str>) -> Option<cpal::Device> {
        let host = cpal::default_host();
        let devices: Vec<cpal::Device> = host.input_devices().map(|all| all.collect()).unwrap_or_default();
        let names: Vec<String> = devices.iter().map(name_of).collect();
        match pick(chosen, &names) {
            Some(name) => devices.into_iter().find(|device| name_of(device) == name),
            None => host.default_input_device(),
        }
    }

    fn listen<T>(device: &cpal::Device, config: cpal::StreamConfig, sink: Arc<Mutex<Vec<f32>>>, fault: Arc<Mutex<Option<String>>>) -> Result<cpal::Stream, String>
    where
        T: SizedSample,
        f32: FromSample<T>,
    {
        let channels = config.channels.max(1) as usize;
        device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    let mut heard = held(&sink);
                    for frame in data.chunks(channels) {
                        heard.push(frame.iter().map(|sample| sample.to_sample::<f32>()).sum::<f32>() / channels as f32);
                    }
                },
                move |error: cpal::Error| {
                    if !matches!(error.kind(), cpal::ErrorKind::Xrun | cpal::ErrorKind::RealtimeDenied) {
                        *held(&fault) = Some(error.to_string());
                    }
                },
                None,
            )
            .map_err(|error| error.to_string())
    }

    fn silenced(device: &cpal::Device) -> bool {
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator};
        use windows::Win32::System::Com::{CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx};
        use windows::core::PCWSTR;

        let Ok(id) = device.id() else {
            return false;
        };
        let wide: Vec<u16> = id.id().encode_utf16().chain(Some(0)).collect();
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let Ok(endpoints) = CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL) else {
                return false;
            };
            let Ok(endpoint) = endpoints.GetDevice(PCWSTR(wide.as_ptr())) else {
                return false;
            };
            let Ok(volume) = endpoint.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) else {
                return false;
            };
            volume.GetMute().is_ok_and(|mute| mute.as_bool())
        }
    }

    fn open(chosen: Option<&str>, sink: Arc<Mutex<Vec<f32>>>, fault: Arc<Mutex<Option<String>>>) -> Result<(cpal::Stream, usize), Refusal> {
        let device = device(chosen).ok_or_else(|| no_microphone("—"))?;
        if silenced(&device) {
            return Err(muted());
        }
        opened(&device, sink, fault).map_err(|error| no_microphone(&error))
    }

    fn opened(device: &cpal::Device, sink: Arc<Mutex<Vec<f32>>>, fault: Arc<Mutex<Option<String>>>) -> Result<(cpal::Stream, usize), String> {
        let supported = device.default_input_config().map_err(|error| error.to_string())?;
        let rate = supported.sample_rate() as usize;
        let format = supported.sample_format();
        let config: cpal::StreamConfig = supported.into();
        let stream = match format {
            cpal::SampleFormat::F32 => listen::<f32>(device, config, sink, fault),
            cpal::SampleFormat::I16 => listen::<i16>(device, config, sink, fault),
            cpal::SampleFormat::I32 => listen::<i32>(device, config, sink, fault),
            cpal::SampleFormat::U16 => listen::<u16>(device, config, sink, fault),
            other => Err(format!("{other:?}")),
        }?;
        stream.play().map_err(|error| error.to_string())?;
        Ok((stream, rate))
    }

    fn plain(language: &str, threads: i32, tokens: i32, samples: usize) -> FullParams<'_, '_> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(language));
        params.set_n_threads(threads);
        params.set_temperature_inc(0.0);
        params.set_max_tokens(tokens);
        params.set_audio_ctx(context(samples));
        params.set_translate(false);
        params.set_no_timestamps(true);
        params.set_suppress_blank(true);
        params.set_suppress_nst(true);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        params
    }

    fn written(state: &WhisperState) -> String {
        state.as_iter().filter_map(|segment| segment.to_str_lossy().ok().map(|text| text.into_owned())).collect()
    }

    fn transcribe(state: &mut WhisperState, phrase: &[f32], language: &str, threads: i32, said: &str, guess: bool) -> Result<String, Refusal> {
        let audio = louder(phrase);
        let tail: String = said.chars().rev().take(PROMPT_TAIL).collect::<Vec<_>>().into_iter().rev().collect();
        let tokens = if guess { most_tokens(audio.len()).min(GUESS_TOKENS) } else { most_tokens(audio.len()) };
        let mut params = plain(language, threads, tokens, audio.len());
        params.set_initial_prompt(&tail);
        state.full(params, &audio).map_err(|error| untranscribed(&error.to_string()))?;
        Ok(unrepeated(&cleaned(&written(state))))
    }

    pub fn called(state: &mut WhisperState, phrase: &[f32], language: &str, prompt: Option<&str>) -> Option<String> {
        let audio = louder(&phrase[..phrase.len().min(WAKE_HEARD)]);
        let mut params = plain(language, WAKE_THREADS, WAKE_TOKENS, audio.len());
        params.set_single_segment(true);
        if let Some(prompt) = prompt {
            params.set_initial_prompt(prompt);
        }
        state.full(params, &audio).ok()?;
        Some(written(state))
    }

    fn spoken(language: Language) -> &'static str {
        match language {
            Language::Ja | Language::Zh => Language::En.id(),
            other => other.id(),
        }
    }

    pub fn wakes(state: &mut WhisperState, phrase: &[f32], language: &str) -> bool {
        called(state, phrase, language, Some(WAKE_PROMPT)).is_some_and(|text| woken(&text)) && called(state, phrase, language, None).is_some_and(|text| named(&text))
    }

    pub fn loaded(model: &Path) -> Result<WhisperState, Refusal> {
        whisper_rs::install_logging_hooks();
        let path = model.to_string_lossy().into_owned();
        let whisper = WhisperContext::new_with_params(&path, WhisperContextParameters { flash_attn: true, ..WhisperContextParameters::default() }).map_err(|error| unloaded(&error.to_string()))?;
        whisper.create_state().map_err(|error| unloaded(&error.to_string()))
    }

    pub fn transcriber(id: u32, model: PathBuf, language: String, jobs: Receiver<Job>, tell: Tell) -> Result<(), Refusal> {
        let mut state = loaded(&model)?;
        let threads = std::thread::available_parallelism().map(|count| count.get().clamp(1, 8)).unwrap_or(4) as i32;
        let mut said = String::new();
        let mut queue = VecDeque::new();
        loop {
            if queue.is_empty() {
                match jobs.recv() {
                    Ok(job) => queue.push_back(job),
                    Err(_) => break,
                }
            }
            queue.extend(jobs.try_iter());
            match next(&mut queue) {
                Some(Job::Guess(phrase)) => {
                    let text = transcribe(&mut state, &phrase, &language, threads, &said, true)?;
                    if !text.is_empty() && !looping(&text) {
                        tell(Heard::Guess { id, text });
                    }
                }
                Some(Job::Final(phrase)) => {
                    let text = transcribe(&mut state, &phrase, &language, threads, &said, false)?;
                    if !text.is_empty() {
                        said = joined(&said, &text);
                        tell(Heard::Phrase { id, text });
                    }
                }
                None => {}
            }
        }
        Ok(())
    }

    pub fn session(id: u32, hearing: Hearing, stopped: Receiver<()>, ready: Sender<Result<(), Refusal>>, tell: Tell) {
        let Hearing { chosen, model, language, rest } = hearing;
        let sink = Arc::new(Mutex::new(Vec::new()));
        let fault = Arc::new(Mutex::new(None));
        let (stream, rate) = match open(chosen.as_deref(), sink.clone(), fault.clone()) {
            Ok(open) => open,
            Err(refused) => {
                let _ = ready.send(Err(refused));
                return;
            }
        };
        let (jobs, heard) = mpsc::channel::<Job>();
        let writer = model.map(|model| {
            let tell = tell.clone();
            std::thread::spawn(move || transcriber(id, model, language, heard, tell))
        });
        let _ = ready.send(Ok(()));
        let mut resampler = Resampler::new(rate);
        let mut segmenter = Segmenter::new(PAUSE, rest);
        let mut refusal = None;
        let mut since_guess = 0;
        let mut silent_ticks = 0;
        let mut heard_sound = false;
        loop {
            std::thread::sleep(TICK);
            let raw = std::mem::take(&mut *held(&sink));
            heard_sound = heard_sound || raw.iter().any(|sample| *sample != 0.0);
            silent_ticks += 1;
            if !heard_sound && silent_ticks >= DEAF {
                refusal = Some(deaf());
                break;
            }
            let mut audio = Vec::with_capacity(raw.len() / 2);
            resampler.push(&raw, &mut audio);
            let mut idle = false;
            for cut in segmenter.push(&audio) {
                match cut {
                    Cut::Phrase(phrase) => {
                        let _ = jobs.send(Job::Final(phrase));
                        since_guess = 0;
                    }
                    Cut::Idle => idle = true,
                }
            }
            since_guess += 1;
            if writer.is_some()
                && since_guess >= GUESS_EVERY
                && let Some(current) = segmenter.current().filter(|current| current.len() >= GUESS_FROM)
            {
                let _ = jobs.send(Job::Guess(current.to_vec()));
                since_guess = 0;
            }
            tell(Heard::Level { id, level: level(segmenter.loudness()) });
            if let Some(error) = held(&fault).take() {
                refusal = Some(no_microphone(&error));
                break;
            }
            if idle || !matches!(stopped.try_recv(), Err(TryRecvError::Empty)) {
                break;
            }
        }
        drop(stream);
        if let Some(rest) = segmenter.finish() {
            let _ = jobs.send(Job::Final(rest));
        }
        drop(jobs);
        if let Some(Err(failed)) = writer.map(|writer| writer.join().unwrap_or(Ok(()))) {
            refusal = refusal.or(Some(failed));
        }
        tell(Heard::Ended { id, refusal });
    }

    pub fn ear(chosen: Option<String>, model: PathBuf, stopped: Receiver<()>, ready: Sender<Result<(), Refusal>>, busy: Arc<AtomicBool>, tell: Tell) {
        let mut state = match loaded(&model) {
            Ok(state) => state,
            Err(refused) => {
                let _ = ready.send(Err(refused));
                return;
            }
        };
        let sink = Arc::new(Mutex::new(Vec::new()));
        let fault = Arc::new(Mutex::new(None));
        let mut stream = None;
        let mut resampler = Resampler::new(RATE);
        let mut segmenter = Segmenter::new(WAKE_PAUSE, IDLE);
        let mut unheard = REOPEN;
        let mut ready = Some(ready);
        loop {
            if stream.is_none() && unheard >= REOPEN {
                unheard = 0;
                let opened = open(chosen.as_deref(), sink.clone(), fault.clone());
                if let Some(ready) = ready.take() {
                    let _ = ready.send(opened.as_ref().map(|_| ()).map_err(Refusal::clone));
                }
                if let Ok((open, rate)) = opened {
                    stream = Some(open);
                    resampler = Resampler::new(rate);
                    segmenter = Segmenter::new(WAKE_PAUSE, IDLE);
                }
            }
            std::thread::sleep(TICK);
            if !matches!(stopped.try_recv(), Err(TryRecvError::Empty)) {
                break;
            }
            let raw = std::mem::take(&mut *held(&sink));
            if held(&fault).take().is_some() {
                stream = None;
            }
            if stream.is_none() {
                unheard += 1;
                continue;
            }
            if busy.load(Ordering::SeqCst) {
                segmenter = Segmenter::new(WAKE_PAUSE, IDLE);
                continue;
            }
            let mut audio = Vec::with_capacity(raw.len() / 2);
            resampler.push(&raw, &mut audio);
            let phrases: Vec<Vec<f32>> = segmenter
                .push(&audio)
                .into_iter()
                .filter_map(|cut| match cut {
                    Cut::Phrase(phrase) => Some(phrase),
                    Cut::Idle => None,
                })
                .collect();
            let language = spoken(language::now());
            if phrases.iter().any(|phrase| wakes(&mut state, phrase, language)) {
                segmenter = Segmenter::new(WAKE_PAUSE, IDLE);
                tell(Heard::Woke);
            }
        }
    }
}

#[cfg(not(windows))]
mod engine {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::mpsc::{Receiver, Sender};

    use super::{Hearing, Microphone, Refusal, Tell, no_microphone};

    pub fn microphones() -> Vec<Microphone> {
        Vec::new()
    }

    pub fn session(_id: u32, _hearing: Hearing, _stopped: Receiver<()>, ready: Sender<Result<(), Refusal>>, _tell: Tell) {
        let _ = ready.send(Err(no_microphone("Windows only")));
    }

    pub fn ear(_chosen: Option<String>, _model: PathBuf, _stopped: Receiver<()>, ready: Sender<Result<(), Refusal>>, _busy: Arc<AtomicBool>, _tell: Tell) {
        let _ = ready.send(Err(no_microphone("Windows only")));
    }
}

pub fn microphones() -> Vec<Microphone> {
    engine::microphones()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frames: usize, loud: f32) -> Vec<f32> {
        (0..frames * FRAME).map(|at| loud * ((at as f32) * 0.3).sin()).collect()
    }

    #[test]
    fn a_phrase_ends_at_a_pause_and_starts_a_little_before_the_voice() {
        let mut segmenter = Segmenter::default();
        assert!(segmenter.push(&tone(10, 0.001)).is_empty());
        assert!(segmenter.push(&tone(8, 0.2)).is_empty());
        let cuts = segmenter.push(&tone(PAUSE, 0.001));
        let [Cut::Phrase(phrase)] = cuts.as_slice() else { panic!("{cuts:?}") };
        assert_eq!(phrase.len(), (LEAD + 8 + TAIL_KEPT) * FRAME);
    }

    #[test]
    fn a_click_is_not_a_phrase_and_a_long_silence_ends_dictation() {
        let mut segmenter = Segmenter::default();
        segmenter.push(&tone(5, 0.001));
        segmenter.push(&tone(2, 0.2));
        assert!(segmenter.push(&tone(PAUSE, 0.001)).is_empty());
        let cuts = segmenter.push(&tone(IDLE, 0.001));
        assert_eq!(cuts, vec![Cut::Idle]);
    }

    #[test]
    fn a_quiet_microphone_is_heard_over_its_own_noise() {
        let mut segmenter = Segmenter::default();
        segmenter.push(&tone(10, 0.0003));
        segmenter.push(&tone(6, 0.008));
        let cuts = segmenter.push(&tone(PAUSE, 0.0003));
        assert!(matches!(cuts.as_slice(), [Cut::Phrase(_)]), "{cuts:?}");
    }

    #[test]
    fn what_is_left_when_dictation_stops_is_still_transcribed() {
        let mut segmenter = Segmenter::default();
        segmenter.push(&tone(3, 0.001));
        segmenter.push(&tone(6, 0.2));
        assert!(segmenter.finish().is_some());
        assert!(segmenter.finish().is_none());
    }

    #[test]
    fn the_microphone_is_heard_at_sixteen_kilohertz_whatever_its_rate() {
        for rate in [48_000, 44_100, 16_000] {
            let mut resampler = Resampler::new(rate);
            let mut out = Vec::new();
            for _ in 0..10 {
                resampler.push(&vec![0.5; rate / 10], &mut out);
            }
            assert!((out.len() as i64 - RATE as i64).abs() <= 2, "{rate}: {}", out.len());
            assert!(out.iter().skip(1).all(|sample| (sample - 0.5).abs() < 1e-5));
        }
    }

    fn heard_at_sixteen_kilohertz(hertz: f32) -> f32 {
        let rate = 48_000;
        let mut resampler = Resampler::new(rate);
        let mut out = Vec::new();
        for chunk in (0..rate).map(|at| (2.0 * std::f32::consts::PI * hertz * at as f32 / rate as f32).sin()).collect::<Vec<_>>().chunks(rate / 10) {
            resampler.push(chunk, &mut out);
        }
        rms(&out[RATE / 10..])
    }

    #[test]
    fn the_voice_band_passes_and_what_would_fold_into_it_does_not() {
        assert!(heard_at_sixteen_kilohertz(1_000.0) > 0.65);
        assert!(heard_at_sixteen_kilohertz(4_000.0) > 0.65);
        assert!(heard_at_sixteen_kilohertz(10_000.0) < 0.01);
        assert!(heard_at_sixteen_kilohertz(14_000.0) < 0.01);
    }

    #[test]
    fn a_quiet_voice_is_raised_but_noise_is_not_blown_up_without_end() {
        let loud = louder(&[0.01, -0.02]);
        assert!((loud[1] + 0.16).abs() < 1e-6, "{loud:?}");
        let raised = louder(&[0.05, -0.1]);
        assert!((raised[1] + 0.8).abs() < 1e-6, "{raised:?}");
        assert!((louder(&[0.5, -0.9])[1] + 0.9).abs() < 1e-6);
        assert_eq!(louder(&[0.0, 0.0]), vec![0.0, 0.0]);
    }

    #[test]
    fn music_tags_and_subtitle_credits_are_not_dictation() {
        assert_eq!(cleaned(" [Música] "), "");
        assert_eq!(cleaned("Subtítulos realizados por la comunidad de Amara.org"), "");
        assert_eq!(cleaned("¡Gracias por ver!"), "");
        assert_eq!(cleaned("Añade un test [risas] para el botón."), "Añade un test para el botón.");
        assert_eq!(cleaned(" Revisa el componente del botón. "), "Revisa el componente del botón.");
    }

    #[test]
    fn a_sentence_whisper_repeats_is_written_once() {
        assert_eq!(unrepeated("Añade un test. Añade un test. Añade un test."), "Añade un test.");
        assert_eq!(unrepeated("Añade un test para el botón. Añade un test para el bo"), "Añade un test para el botón.");
        assert_eq!(unrepeated("Hola. Revisa el botón"), "Hola. Revisa el botón");
        assert_eq!(unrepeated("¿Qué hora es? Son las tres."), "¿Qué hora es? Son las tres.");
        assert_eq!(unrepeated(""), "");
    }

    #[test]
    fn a_guess_that_goes_round_in_circles_is_not_shown() {
        assert!(looping("Y si se ha ha dicho, se ha ha dicho, se ha ha dicho, se ha ha dicho,"));
        assert!(!looping("Después añade un test para el componente del botón."));
        assert!(!looping("que sí, que sí, que no"));
    }

    #[test]
    fn hey_sens_wakes_however_whisper_spells_it_and_only_at_the_start() {
        for said in ["Hey Sens.", "¡Hey, Senz!", "Ey, Sens", "Oye Sens, revisa el botón", "Hola Sens.", "Hey, sense.", "ehi, senz", "Heysans", "Oyesens", " hey  sens ", "HEY SENS"] {
            assert!(woken(said), "{said}");
        }
        for said in ["Hay sensores en la placa.", "Revisa el botón, hey Sens.", "Hey.", "Sens.", "Hey, ¿qué tal?", "Hay que ser sensatos", "", "A sense of humour"] {
            assert!(!woken(said), "{said}");
        }
    }

    #[test]
    fn what_whisper_hears_without_help_must_sound_like_sens_near_the_start() {
        for said in ["Asens.", "¡Escens?", "A-Sense, revisa el botón", "Heysans", "ehi, senz", "Ascent's, revise the formula's button.", "Oye senz"] {
            assert!(named(said), "{said}");
        }
        for said in [" y", "¡Suscríbete!", "¡Pasar!", "I'm sorry.", "", "Revisa el componente del botón."] {
            assert!(!named(said), "{said}");
        }
    }

    #[test]
    fn dictation_started_by_voice_ends_after_a_few_seconds_of_silence() {
        let mut segmenter = Segmenter::new(PAUSE, HANDS_FREE_IDLE);
        segmenter.push(&tone(3, 0.001));
        segmenter.push(&tone(8, 0.2));
        assert!(matches!(segmenter.push(&tone(PAUSE, 0.001)).as_slice(), [Cut::Phrase(_)]));
        assert!(segmenter.push(&tone(HANDS_FREE_IDLE - 1, 0.001)).is_empty());
        assert_eq!(segmenter.push(&tone(1, 0.001)), vec![Cut::Idle]);
    }

    #[test]
    fn a_short_pause_is_enough_to_hear_the_wake_phrase() {
        let mut segmenter = Segmenter::new(WAKE_PAUSE, IDLE);
        segmenter.push(&tone(5, 0.001));
        segmenter.push(&tone(8, 0.2));
        assert!(matches!(segmenter.push(&tone(WAKE_PAUSE, 0.001)).as_slice(), [Cut::Phrase(_)]));
    }

    #[test]
    fn phrases_join_what_was_written_with_one_space() {
        assert_eq!(joined("", " Hola."), "Hola.");
        assert_eq!(joined("Revisa ", "el botón."), "Revisa el botón.");
        assert_eq!(joined("Hola", ""), "Hola");
    }

    #[test]
    fn the_level_meter_spans_sixty_decibels() {
        assert_eq!(level(0.0), 0.0);
        assert_eq!(level(0.001), 0.0);
        assert!((level(0.1) - 0.8).abs() < 1e-6);
        assert_eq!(level(1.0), 1.0);
    }

    #[test]
    fn the_chosen_microphone_is_used_while_it_is_plugged_in() {
        let names = vec!["Voicemeeter Out B2".to_string(), "Micrófono (USB)".to_string()];
        assert_eq!(pick(Some("Micrófono (USB)"), &names), Some("Micrófono (USB)"));
        assert_eq!(pick(Some("Auriculares"), &names), None);
        assert_eq!(pick(None, &names), None);
    }

    #[test]
    fn the_model_counts_only_when_it_is_whole_and_the_choice_is_kept() {
        let base = std::env::temp_dir().join(format!("sens-voice-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(FOLDER)).unwrap();
        assert!(!model(&base, false).ready);
        std::fs::write(model_path(&base), b"half").unwrap();
        assert!(!model(&base, true).ready);
        choose(&base, Some("Micrófono (USB)".into())).unwrap();
        assert_eq!(chosen(&base).as_deref(), Some("Micrófono (USB)"));
        choose(&base, None).unwrap();
        assert_eq!(chosen(&base), None);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn the_old_voice_model_goes_away_and_the_current_one_stays() {
        let base = std::env::temp_dir().join(format!("sens-voice-retire-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(FOLDER)).unwrap();
        std::fs::write(base.join(FOLDER).join(RETIRED[0]), b"old").unwrap();
        std::fs::write(model_path(&base), b"new").unwrap();
        retire(&base);
        assert!(!base.join(FOLDER).join(RETIRED[0]).exists());
        assert!(model_path(&base).exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "needs SENS_VOICE_MODEL and SENS_VOICE_WAV"]
    fn a_quiet_recording_of_two_sentences_comes_out_as_two_phrases() {
        let model = PathBuf::from(std::env::var("SENS_VOICE_MODEL").unwrap());
        let bytes = std::fs::read(std::env::var("SENS_VOICE_WAV").unwrap()).unwrap();
        let data = bytes.windows(4).position(|window| window == b"data").unwrap() + 8;
        let quiet: Vec<f32> = bytes[data..].as_chunks::<2>().0.iter().map(|pair| i16::from_le_bytes(*pair) as f32 / 32768.0 * 0.05).collect();
        let mut resampler = Resampler::new(48_000);
        let mut segmenter = Segmenter::default();
        let (phrases, heard) = mpsc::channel();
        let said = Arc::new(Mutex::new(Vec::new()));
        let guessed = Arc::new(Mutex::new(Vec::new()));
        let (sink, guesses) = (said.clone(), guessed.clone());
        let began = std::time::Instant::now();
        let writer = std::thread::spawn(move || {
            engine::transcriber(1, model, "es".into(), heard, Arc::new(move |event| match event {
                Heard::Phrase { text, .. } => {
                    println!("{:>6}ms phrase {text}", began.elapsed().as_millis());
                    held(&sink).push(text)
                }
                Heard::Guess { text, .. } => {
                    println!("{:>6}ms guess  {text}", began.elapsed().as_millis());
                    held(&guesses).push(text)
                }
                _ => {}
            }))
        });
        let mut since_guess = 0;
        for chunk in quiet.chunks(4_800) {
            std::thread::sleep(TICK);
            let mut audio = Vec::new();
            resampler.push(chunk, &mut audio);
            for cut in segmenter.push(&audio) {
                if let Cut::Phrase(phrase) = cut {
                    phrases.send(Job::Final(phrase)).unwrap();
                    since_guess = 0;
                }
            }
            since_guess += 1;
            if since_guess >= GUESS_EVERY
                && let Some(current) = segmenter.current().filter(|current| current.len() >= GUESS_FROM)
            {
                phrases.send(Job::Guess(current.to_vec())).unwrap();
                since_guess = 0;
            }
        }
        if let Some(rest) = segmenter.finish() {
            phrases.send(Job::Final(rest)).unwrap();
        }
        drop(phrases);
        writer.join().unwrap().unwrap();
        let said = held(&said).clone();
        let guessed = held(&guessed).clone();
        println!("guesses {guessed:?}");
        println!("phrases {said:?}");
        assert!(!guessed.is_empty());
        assert_eq!(said.len(), 2, "{said:?}");
        assert!(said[0].contains("formulario de contacto"), "{said:?}");
        assert!(said[1].contains("componente del botón"), "{said:?}");
    }

    #[test]
    #[ignore = "downloads the voice model from the release"]
    fn the_voice_model_comes_from_the_release_whole_and_checked() {
        let base = std::env::temp_dir().join(format!("sens-voice-fetch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (sent, heard) = mpsc::channel();
        let voice = Voice::new(base.clone(), move |event| {
            let _ = sent.send(event);
        });
        voice.prepare();
        let last = heard.iter().find(|event| matches!(event, Heard::Ready | Heard::Unfetched { .. })).unwrap();
        assert_eq!(last, Heard::Ready);
        assert!(voice.model().ready);
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    #[ignore = "opens the microphone"]
    fn testing_the_microphone_reports_its_level_until_it_is_stopped() {
        let base = std::env::temp_dir().join(format!("sens-voice-test-{}", std::process::id()));
        let (sent, heard) = mpsc::channel();
        let voice = Voice::new(base, move |event| {
            let _ = sent.send(event);
        });
        let id = voice.start("", false, false).unwrap();
        std::thread::sleep(Duration::from_millis(800));
        voice.stop();
        let events: Vec<Heard> = heard.try_iter().collect();
        assert!(events.iter().filter(|event| matches!(event, Heard::Level { .. })).count() >= 5, "{events:?}");
        assert_eq!(events.last(), Some(&Heard::Ended { id, refusal: None }));
    }

    #[test]
    #[ignore = "opens the microphone and needs SENS_VOICE_MODEL"]
    fn a_dictation_loads_the_model_listens_and_ends_cleanly() {
        let base = std::env::temp_dir().join(format!("sens-voice-session-{}", std::process::id()));
        std::fs::create_dir_all(base.join(FOLDER)).unwrap();
        std::fs::copy(std::env::var("SENS_VOICE_MODEL").unwrap(), model_path(&base)).unwrap();
        let (sent, heard) = mpsc::channel();
        let voice = Voice::new(base.clone(), move |event| {
            let _ = sent.send(event);
        });
        let id = voice.start("es", true, false).unwrap();
        std::thread::sleep(Duration::from_millis(1500));
        voice.stop();
        let events: Vec<Heard> = heard.try_iter().collect();
        assert!(events.iter().any(|event| matches!(event, Heard::Level { .. })), "{events:?}");
        assert_eq!(events.last(), Some(&Heard::Ended { id, refusal: None }), "{events:?}");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_newer_guess_or_any_phrase_makes_an_older_guess_worthless() {
        let mut queue = VecDeque::from([Job::Guess(vec![1.0]), Job::Guess(vec![2.0]), Job::Final(vec![3.0]), Job::Guess(vec![4.0]), Job::Guess(vec![5.0])]);
        assert_eq!(next(&mut queue), Some(Job::Final(vec![3.0])));
        assert_eq!(next(&mut queue), Some(Job::Guess(vec![5.0])));
        assert_eq!(next(&mut queue), None);
        let mut phrases = VecDeque::from([Job::Final(vec![1.0]), Job::Final(vec![2.0])]);
        assert_eq!(next(&mut phrases), Some(Job::Final(vec![1.0])));
        assert_eq!(next(&mut phrases), Some(Job::Final(vec![2.0])));
    }

    #[test]
    fn a_phrase_may_take_about_ten_tokens_a_second_and_no_more() {
        assert_eq!(most_tokens(RATE * 3), 30 + TOKENS_SPARE as i32);
        assert_eq!(most_tokens(0), TOKENS_SPARE as i32);
    }

    #[test]
    fn whisper_looks_only_at_as_much_audio_as_was_said() {
        assert_eq!(context(RATE * 3), 150 + CONTEXT_MARGIN as i32);
        assert_eq!(context(0), LEAST_CONTEXT as i32);
        assert_eq!(context(RATE * 60), FULL_CONTEXT as i32);
    }

    #[test]
    fn a_phrase_being_said_can_be_read_while_it_goes_on() {
        let mut segmenter = Segmenter::default();
        segmenter.push(&tone(5, 0.001));
        assert!(segmenter.current().is_none());
        segmenter.push(&tone(8, 0.2));
        assert_eq!(segmenter.current().map(<[f32]>::len), Some((LEAD + 8) * FRAME));
    }

    #[test]
    fn what_is_heard_reaches_the_interface_tagged_by_kind() {
        let phrase = serde_json::to_value(Heard::Phrase { id: 2, text: "Hola.".into() }).unwrap();
        assert_eq!(phrase, serde_json::json!({ "kind": "phrase", "id": 2, "text": "Hola." }));
        let ended = serde_json::to_value(Heard::Ended { id: 2, refusal: Some(no_model()) }).unwrap();
        assert_eq!(ended["refusal"]["cause"], "model");
        assert_eq!(serde_json::to_value(muted()).unwrap()["cause"], "silent");
        assert_eq!(serde_json::to_value(deaf()).unwrap()["cause"], "silent");
        assert_eq!(serde_json::to_value(Heard::Ready).unwrap(), serde_json::json!({ "kind": "ready" }));
        assert_eq!(serde_json::to_value(Heard::Woke).unwrap(), serde_json::json!({ "kind": "woke" }));
    }

    #[test]
    fn hey_sens_waits_for_the_voice_model_and_stops_when_switched_off() {
        let base = std::env::temp_dir().join(format!("sens-voice-wake-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join(FOLDER)).unwrap();
        std::fs::write(model_path(&base), b"half").unwrap();
        let voice = Voice::new(base.clone(), |_| {});
        voice.fetching.store(true, Ordering::SeqCst);
        assert_eq!(voice.wake(true), Ok(()));
        assert!(voice.waking());
        assert!(held(&voice.ear).is_none());
        assert_eq!(voice.wake(false), Ok(()));
        assert!(!voice.waking());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "needs SENS_VOICE_MODEL and SENS_WAKE_WAVS"]
    fn the_wake_phrase_is_heard_and_other_phrases_are_not() {
        let mut state = engine::loaded(Path::new(&std::env::var("SENS_VOICE_MODEL").unwrap())).unwrap();
        let mut wrong = Vec::new();
        for wav in std::env::var("SENS_WAKE_WAVS").unwrap().split(';') {
            let bytes = std::fs::read(wav).unwrap();
            let data = bytes.windows(4).position(|window| window == b"data").unwrap() + 8;
            let said = bytes[data..].as_chunks::<2>().0.iter().map(|pair| i16::from_le_bytes(*pair) as f32 / 32768.0);
            let audio: Vec<f32> = std::iter::repeat_n(0.0, RATE).chain(said).chain(std::iter::repeat_n(0.0, RATE * 2)).collect();
            let mut segmenter = Segmenter::new(WAKE_PAUSE, IDLE);
            let mut phrases: Vec<Vec<f32>> = segmenter.push(&audio).into_iter().filter_map(|cut| if let Cut::Phrase(phrase) = cut { Some(phrase) } else { None }).collect();
            phrases.extend(segmenter.finish());
            for language in ["es", "en"] {
                let began = std::time::Instant::now();
                let heard: Vec<(Option<String>, Option<String>)> =
                    phrases.iter().map(|phrase| (engine::called(&mut state, phrase, language, Some(WAKE_PROMPT)), engine::called(&mut state, phrase, language, None))).collect();
                let woke = phrases.iter().any(|phrase| engine::wakes(&mut state, phrase, language));
                println!("{language} {:>5}ms {woke:<5} {wav} {heard:?}", began.elapsed().as_millis());
                if woke != Path::new(wav).file_name().unwrap().to_string_lossy().starts_with("wake") {
                    wrong.push(format!("{language} {wav} {heard:?}"));
                }
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
    }
}
