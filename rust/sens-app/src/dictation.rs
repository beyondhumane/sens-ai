use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, mpsc};
use std::thread::{self, JoinHandle};

use sens_agent::said;
use serde::{Deserialize, Serialize};

const PRIVACY_NOT_ACCEPTED: u32 = 0x8004_5509;
const ACCESS_DENIED: u32 = 0x8007_0005;

const SUCCESS: i32 = 0;
const TOPIC_LANGUAGE_NOT_SUPPORTED: i32 = 1;
const AUDIO_QUALITY_FAILURE: i32 = 4;
const USER_CANCELED: i32 = 5;
const TIMEOUT_EXCEEDED: i32 = 7;
const PAUSE_LIMIT_EXCEEDED: i32 = 8;
const NETWORK_FAILURE: i32 = 9;
const MICROPHONE_UNAVAILABLE: i32 = 10;

const HIGH: i32 = 0;
const MEDIUM: i32 = 1;

pub const WAKE_PHRASES: [&str; 4] = ["hey sens", "hey sense", "oye sens", "ey sens"];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Cause {
    Speech,
    Microphone,
    Unsupported,
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
    Guess { id: u32, text: String },
    Phrase { id: u32, text: String },
    Ended { id: u32, refusal: Option<Refusal> },
    Woke,
    Slept { refusal: Refusal },
}

fn refusal(cause: Cause, message: String) -> Refusal {
    Refusal { cause, message }
}

fn speech_off() -> Refusal {
    refusal(
        Cause::Speech,
        said!(
            en: "Windows online speech recognition is off, and dictation needs it",
            es: "el reconocimiento de voz en línea de Windows está desactivado, y el dictado lo necesita",
            fr: "la reconnaissance vocale en ligne de Windows est désactivée, et la dictée en a besoin",
            de: "die Online-Spracherkennung von Windows ist aus, und das Diktat braucht sie",
            ja: "Windows のオンライン音声認識がオフになっています。音声入力にはこれが必要です",
            zh: "Windows 联机语音识别已关闭，语音输入需要它",
        ),
    )
}

fn microphone_off() -> Refusal {
    refusal(
        Cause::Microphone,
        said!(
            en: "Windows isn’t letting Sens use the microphone",
            es: "Windows no deja que Sens use el micrófono",
            fr: "Windows n’autorise pas Sens à utiliser le microphone",
            de: "Windows lässt Sens das Mikrofon nicht verwenden",
            ja: "Windows が Sens にマイクの使用を許可していません",
            zh: "Windows 不允许 Sens 使用麦克风",
        ),
    )
}

fn unsupported() -> Refusal {
    refusal(
        Cause::Unsupported,
        said!(
            en: "Windows has no speech recognition for dictation on this computer",
            es: "Windows no tiene reconocimiento de voz para dictar en este equipo",
            fr: "Windows ne propose pas de reconnaissance vocale pour la dictée sur cet ordinateur",
            de: "Windows hat auf diesem Computer keine Spracherkennung zum Diktieren",
            ja: "このコンピューターの Windows には音声入力用の音声認識がありません",
            zh: "此电脑上的 Windows 没有可用于语音输入的语音识别",
        ),
    )
}

fn out_of_reach() -> Refusal {
    refusal(
        Cause::Other,
        said!(
            en: "couldn’t reach Windows speech recognition",
            es: "no pude conectar con el reconocimiento de voz de Windows",
            fr: "impossible de joindre la reconnaissance vocale de Windows",
            de: "die Spracherkennung von Windows ist nicht erreichbar",
            ja: "Windows の音声認識に接続できませんでした",
            zh: "无法连接 Windows 语音识别",
        ),
    )
}

fn failed(error: &str) -> Refusal {
    refusal(
        Cause::Other,
        said!(
            en: "dictation stopped: {error}",
            es: "el dictado se detuvo: {error}",
            fr: "la dictée s’est arrêtée : {error}",
            de: "das Diktat wurde beendet: {error}",
            ja: "音声入力が停止しました: {error}",
            zh: "语音输入已停止：{error}",
        ),
    )
}

pub fn refused(code: u32, error: &str) -> Refusal {
    match code {
        PRIVACY_NOT_ACCEPTED => speech_off(),
        ACCESS_DENIED => microphone_off(),
        _ => failed(error),
    }
}

pub fn ending(status: i32) -> Option<Refusal> {
    match status {
        SUCCESS | USER_CANCELED | TIMEOUT_EXCEEDED | PAUSE_LIMIT_EXCEEDED => None,
        TOPIC_LANGUAGE_NOT_SUPPORTED => Some(unsupported()),
        AUDIO_QUALITY_FAILURE | MICROPHONE_UNAVAILABLE => Some(microphone_off()),
        NETWORK_FAILURE => Some(out_of_reach()),
        other => Some(failed(&format!("{other}"))),
    }
}

pub fn wakes(status: i32, confidence: i32) -> bool {
    status == SUCCESS && matches!(confidence, HIGH | MEDIUM)
}

pub fn pick(wanted: &str, offered: &[String], system: &str) -> Option<String> {
    let primary = |tag: &str| tag.split('-').next().unwrap_or(tag).to_ascii_lowercase();
    offered
        .iter()
        .find(|one| one.eq_ignore_ascii_case(wanted))
        .or_else(|| offered.iter().find(|one| primary(one) == primary(wanted)))
        .or_else(|| offered.iter().find(|one| one.eq_ignore_ascii_case(system)))
        .or_else(|| offered.first())
        .cloned()
}

pub fn settings(cause: Cause) -> Option<&'static str> {
    match cause {
        Cause::Speech => Some("ms-settings:privacy-speech"),
        Cause::Microphone => Some("ms-settings:privacy-microphone"),
        Cause::Unsupported | Cause::Other => None,
    }
}

type Tell = Arc<dyn Fn(Heard) + Send + Sync>;

enum Signal {
    Stop,
    Ended(Option<Refusal>),
}

struct Live {
    signal: mpsc::Sender<Signal>,
    thread: JoinHandle<()>,
}

impl Live {
    fn end(self) {
        let _ = self.signal.send(Signal::Stop);
        let _ = self.thread.join();
    }

    fn running(&self) -> bool {
        !self.thread.is_finished()
    }
}

fn held<T>(slot: &Mutex<T>) -> MutexGuard<'_, T> {
    slot.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct Shared {
    tell: Tell,
    dictating: Mutex<Option<Live>>,
    listening: Mutex<Option<Live>>,
    wake: Mutex<Option<String>>,
    latest: AtomicU32,
}

impl Shared {
    fn listen(&self, after: Option<u32>) -> Result<(), Refusal> {
        let mut listening = held(&self.listening);
        let Some(language) = held(&self.wake).clone() else {
            return Ok(());
        };
        let dictating = held(&self.dictating).as_ref().is_some_and(Live::running);
        let superseded = after.is_some_and(|id| self.latest.load(Ordering::SeqCst) != id);
        if listening.is_some() || superseded || (after.is_none() && dictating) {
            return Ok(());
        }
        let (signal, signals) = mpsc::channel();
        let (ready, readied) = mpsc::channel();
        let ender = signal.clone();
        let tell = self.tell.clone();
        let thread = thread::spawn(move || engine::watch(&language, ender, signals, ready, tell));
        match readied.recv().unwrap_or_else(|_| Err(unsupported())) {
            Ok(()) => {
                *listening = Some(Live { signal, thread });
                Ok(())
            }
            Err(refusal) => {
                let _ = thread.join();
                Err(refusal)
            }
        }
    }
}

pub struct Dictation {
    shared: Arc<Shared>,
}

impl Dictation {
    pub fn new(tell: impl Fn(Heard) + Send + Sync + 'static) -> Self {
        Self {
            shared: Arc::new(Shared {
                tell: Arc::new(tell),
                dictating: Mutex::new(None),
                listening: Mutex::new(None),
                wake: Mutex::new(None),
                latest: AtomicU32::new(0),
            }),
        }
    }

    pub fn start(&self, language: &str, hands_free: bool) -> Result<u32, Refusal> {
        let shared = &self.shared;
        let (id, watching) = {
            let mut listening = held(&shared.listening);
            (shared.latest.fetch_add(1, Ordering::SeqCst) + 1, listening.take())
        };
        if let Some(watching) = watching {
            watching.end();
        }
        self.stop();
        let (signal, signals) = mpsc::channel();
        let (ready, readied) = mpsc::channel();
        let wanted = language.to_string();
        let ender = signal.clone();
        let after = shared.clone();
        let thread = thread::spawn(move || {
            engine::dictate(id, &wanted, hands_free, ender, signals, ready, after.tell.clone());
            if let Err(refusal) = after.listen(Some(id)) {
                (after.tell)(Heard::Slept { refusal });
            }
        });
        match readied.recv().unwrap_or_else(|_| Err(unsupported())) {
            Ok(()) => {
                *held(&shared.dictating) = Some(Live { signal, thread });
                Ok(id)
            }
            Err(refusal) => {
                let _ = thread.join();
                Err(refusal)
            }
        }
    }

    pub fn stop(&self) {
        let dictating = held(&self.shared.dictating).take();
        if let Some(dictating) = dictating {
            dictating.end();
        }
    }

    pub fn wake(&self, language: Option<&str>) -> Result<(), Refusal> {
        *held(&self.shared.wake) = language.map(str::to_string);
        if language.is_some() {
            return self.shared.listen(None).inspect_err(|_| *held(&self.shared.wake) = None);
        }
        let watching = held(&self.shared.listening).take();
        if let Some(watching) = watching {
            watching.end();
        }
        Ok(())
    }

    pub fn shutdown(&self) {
        let _ = self.wake(None);
        self.stop();
    }
}

#[cfg(windows)]
mod engine {
    use std::sync::Arc;
    use std::sync::mpsc::{Receiver, Sender};

    use windows::Foundation::{TimeSpan, TypedEventHandler};
    use windows::Globalization::Language;
    use windows::Media::SpeechRecognition::{
        ISpeechRecognitionConstraint, SpeechContinuousRecognitionCompletedEventArgs, SpeechContinuousRecognitionResultGeneratedEventArgs,
        SpeechContinuousRecognitionSession, SpeechRecognitionHypothesisGeneratedEventArgs, SpeechRecognitionListConstraint, SpeechRecognitionResultStatus,
        SpeechRecognitionScenario, SpeechRecognitionTopicConstraint, SpeechRecognizer,
    };
    use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};
    use windows_core::{HSTRING, Interface, Ref};

    use super::{Heard, Refusal, Signal, Tell, WAKE_PHRASES, ending, pick, refused, unsupported, wakes};

    const TICKS_PER_SECOND: i64 = 10_000_000;
    const HANDS_FREE_SILENCE: i64 = 4 * TICKS_PER_SECOND;
    const WATCH_SILENCE: i64 = 60 * 60 * TICKS_PER_SECOND;

    struct Apartment(bool);

    impl Apartment {
        fn join() -> Self {
            Self(unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.is_ok())
        }
    }

    impl Drop for Apartment {
        fn drop(&mut self) {
            if self.0 {
                unsafe { CoUninitialize() };
            }
        }
    }

    fn refusal_of(error: windows_core::Error) -> Refusal {
        refused(error.code().0 as u32, &error.message())
    }

    fn tags(languages: windows_core::Result<windows_collections::IVectorView<Language>>) -> Result<Vec<String>, Refusal> {
        Ok(languages.map_err(refusal_of)?.into_iter().filter_map(|one| one.LanguageTag().ok()).map(|tag| tag.to_string()).collect())
    }

    fn recognizer(wanted: &str, offered: Vec<String>) -> Result<SpeechRecognizer, Refusal> {
        let system = SpeechRecognizer::SystemSpeechLanguage().and_then(|one| one.LanguageTag()).map(|tag| tag.to_string()).unwrap_or_default();
        let tag = pick(wanted, &offered, &system).ok_or_else(unsupported)?;
        Language::CreateLanguage(&HSTRING::from(tag)).and_then(|language| SpeechRecognizer::Create(&language)).map_err(refusal_of)
    }

    fn compile(recognizer: &SpeechRecognizer, constraint: ISpeechRecognitionConstraint) -> Result<(), Refusal> {
        recognizer.Constraints().and_then(|all| all.Append(&constraint)).map_err(refusal_of)?;
        let compiled = recognizer.CompileConstraintsAsync().and_then(|compiling| compiling.get()).map_err(refusal_of)?;
        match compiled.Status().ok().and_then(|status| ending(status.0)) {
            Some(refusal) => Err(refusal),
            None => Ok(()),
        }
    }

    fn completed(session: &SpeechContinuousRecognitionSession, ending_now: Sender<Signal>) -> Result<(), Refusal> {
        session
            .Completed(&TypedEventHandler::new(move |_, args: Ref<SpeechContinuousRecognitionCompletedEventArgs>| {
                let refusal = match args.as_ref() {
                    Some(args) => ending(args.Status()?.0),
                    None => None,
                };
                let _ = ending_now.send(Signal::Ended(refusal));
                Ok(())
            }))
            .map(|_| ())
            .map_err(refusal_of)
    }

    fn start(session: &SpeechContinuousRecognitionSession) -> Result<(), Refusal> {
        session.StartAsync().and_then(|starting| starting.get()).map_err(refusal_of)
    }

    fn open_dictation(id: u32, wanted: &str, hands_free: bool, ending_now: Sender<Signal>, tell: &Tell) -> Result<(SpeechRecognizer, SpeechContinuousRecognitionSession), Refusal> {
        let recognizer = recognizer(wanted, tags(SpeechRecognizer::SupportedTopicLanguages())?)?;
        let topic = SpeechRecognitionTopicConstraint::Create(SpeechRecognitionScenario::Dictation, &HSTRING::from("dictation")).map_err(refusal_of)?;
        compile(&recognizer, topic.cast().map_err(refusal_of)?)?;

        let guessing = tell.clone();
        recognizer
            .HypothesisGenerated(&TypedEventHandler::new(move |_, args: Ref<SpeechRecognitionHypothesisGeneratedEventArgs>| {
                if let Some(args) = args.as_ref() {
                    guessing(Heard::Guess { id, text: args.Hypothesis()?.Text()?.to_string() });
                }
                Ok(())
            }))
            .map_err(refusal_of)?;

        let session = recognizer.ContinuousRecognitionSession().map_err(refusal_of)?;
        if hands_free {
            session.SetAutoStopSilenceTimeout(TimeSpan { Duration: HANDS_FREE_SILENCE }).map_err(refusal_of)?;
        }
        let saying = tell.clone();
        session
            .ResultGenerated(&TypedEventHandler::new(move |_, args: Ref<SpeechContinuousRecognitionResultGeneratedEventArgs>| {
                if let Some(result) = args.as_ref().map(|args| args.Result()).transpose()?
                    && result.Status()? == SpeechRecognitionResultStatus::Success
                {
                    let text = result.Text()?.to_string();
                    if !text.trim().is_empty() {
                        saying(Heard::Phrase { id, text });
                    }
                }
                Ok(())
            }))
            .map_err(refusal_of)?;
        completed(&session, ending_now)?;
        start(&session)?;
        Ok((recognizer, session))
    }

    pub fn dictate(id: u32, wanted: &str, hands_free: bool, ending_now: Sender<Signal>, signals: Receiver<Signal>, ready: Sender<Result<(), Refusal>>, tell: Tell) {
        let _apartment = Apartment::join();
        match open_dictation(id, wanted, hands_free, ending_now, &tell) {
            Err(refusal) => {
                let _ = ready.send(Err(refusal));
            }
            Ok((_recognizer, session)) => {
                let _ = ready.send(Ok(()));
                let refusal = match signals.recv() {
                    Ok(Signal::Ended(refusal)) => refusal,
                    Ok(Signal::Stop) | Err(_) => {
                        let _ = session.StopAsync().and_then(|stopping| stopping.get());
                        None
                    }
                };
                tell(Heard::Ended { id, refusal });
            }
        }
    }

    fn open_watch(wanted: &str, ending_now: Sender<Signal>, tell: &Tell) -> Result<(SpeechRecognizer, SpeechContinuousRecognitionSession), Refusal> {
        let recognizer = recognizer(wanted, tags(SpeechRecognizer::SupportedGrammarLanguages())?)?;
        let phrases: Vec<HSTRING> = WAKE_PHRASES.iter().map(|phrase| HSTRING::from(*phrase)).collect();
        let list = SpeechRecognitionListConstraint::Create(&windows_collections::IIterable::from(phrases)).map_err(refusal_of)?;
        compile(&recognizer, list.cast().map_err(refusal_of)?)?;

        let session = recognizer.ContinuousRecognitionSession().map_err(refusal_of)?;
        session.SetAutoStopSilenceTimeout(TimeSpan { Duration: WATCH_SILENCE }).map_err(refusal_of)?;
        let waking: Tell = Arc::clone(tell);
        session
            .ResultGenerated(&TypedEventHandler::new(move |_, args: Ref<SpeechContinuousRecognitionResultGeneratedEventArgs>| {
                if let Some(result) = args.as_ref().map(|args| args.Result()).transpose()?
                    && wakes(result.Status()?.0, result.Confidence()?.0)
                {
                    waking(Heard::Woke);
                }
                Ok(())
            }))
            .map_err(refusal_of)?;
        completed(&session, ending_now)?;
        start(&session)?;
        Ok((recognizer, session))
    }

    pub fn watch(wanted: &str, ending_now: Sender<Signal>, signals: Receiver<Signal>, ready: Sender<Result<(), Refusal>>, tell: Tell) {
        let _apartment = Apartment::join();
        let (_recognizer, session) = match open_watch(wanted, ending_now, &tell) {
            Err(refusal) => {
                let _ = ready.send(Err(refusal));
                return;
            }
            Ok(open) => open,
        };
        let _ = ready.send(Ok(()));
        loop {
            match signals.recv() {
                Ok(Signal::Ended(None)) => {
                    if let Err(refusal) = start(&session) {
                        tell(Heard::Slept { refusal });
                        return;
                    }
                }
                Ok(Signal::Ended(Some(refusal))) => {
                    tell(Heard::Slept { refusal });
                    return;
                }
                Ok(Signal::Stop) | Err(_) => {
                    let _ = session.StopAsync().and_then(|stopping| stopping.get());
                    return;
                }
            }
        }
    }
}

#[cfg(not(windows))]
mod engine {
    use std::sync::mpsc::{Receiver, Sender};

    use super::{Refusal, Signal, Tell, unsupported};

    pub fn dictate(_id: u32, _wanted: &str, _hands_free: bool, _ending: Sender<Signal>, _signals: Receiver<Signal>, ready: Sender<Result<(), Refusal>>, _tell: Tell) {
        let _ = ready.send(Err(unsupported()));
    }

    pub fn watch(_wanted: &str, _ending: Sender<Signal>, _signals: Receiver<Signal>, ready: Sender<Result<(), Refusal>>, _tell: Tell) {
        let _ = ready.send(Err(unsupported()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offered(tags: &[&str]) -> Vec<String> {
        tags.iter().map(|tag| tag.to_string()).collect()
    }

    #[test]
    fn dictation_speaks_the_language_shown_when_windows_has_it() {
        assert_eq!(pick("es-ES", &offered(&["en-US", "es-ES"]), "en-US").as_deref(), Some("es-ES"));
        assert_eq!(pick("fr-fr", &offered(&["FR-FR"]), "").as_deref(), Some("FR-FR"));
        assert_eq!(pick("es", &offered(&["en-US", "es-ES"]), "en-US").as_deref(), Some("es-ES"));
    }

    #[test]
    fn another_region_of_the_same_language_comes_before_the_system_one() {
        assert_eq!(pick("es-ES", &offered(&["en-US", "es-MX"]), "en-US").as_deref(), Some("es-MX"));
        assert_eq!(pick("zh-CN", &offered(&["en-US", "zh-Hans-CN"]), "en-US").as_deref(), Some("zh-Hans-CN"));
    }

    #[test]
    fn without_the_language_shown_it_speaks_the_system_one_or_whatever_windows_has() {
        assert_eq!(pick("ja-JP", &offered(&["de-DE", "es-ES"]), "es-ES").as_deref(), Some("es-ES"));
        assert_eq!(pick("ja-JP", &offered(&["de-DE"]), "es-ES").as_deref(), Some("de-DE"));
        assert_eq!(pick("ja-JP", &[], "es-ES"), None);
    }

    #[test]
    fn a_refusal_from_windows_names_the_setting_that_lets_it_dictate() {
        assert_eq!(refused(PRIVACY_NOT_ACCEPTED, "").cause, Cause::Speech);
        assert_eq!(refused(ACCESS_DENIED, "").cause, Cause::Microphone);
        assert_eq!(refused(0x8000_4005, "Unspecified error").cause, Cause::Other);
        assert!(refused(0x8000_4005, "Unspecified error").message.contains("Unspecified error"));
        assert_eq!(settings(Cause::Speech), Some("ms-settings:privacy-speech"));
        assert_eq!(settings(Cause::Microphone), Some("ms-settings:privacy-microphone"));
        assert_eq!(settings(Cause::Other), None);
    }

    #[test]
    fn silence_and_stopping_end_dictation_quietly_and_the_rest_say_why() {
        for quiet in [SUCCESS, USER_CANCELED, TIMEOUT_EXCEEDED, PAUSE_LIMIT_EXCEEDED] {
            assert_eq!(ending(quiet), None);
        }
        assert_eq!(ending(TOPIC_LANGUAGE_NOT_SUPPORTED).map(|one| one.cause), Some(Cause::Unsupported));
        assert_eq!(ending(MICROPHONE_UNAVAILABLE).map(|one| one.cause), Some(Cause::Microphone));
        assert_eq!(ending(AUDIO_QUALITY_FAILURE).map(|one| one.cause), Some(Cause::Microphone));
        assert_eq!(ending(NETWORK_FAILURE).map(|one| one.cause), Some(Cause::Other));
    }

    #[test]
    fn only_a_clearly_heard_wake_phrase_wakes_sens() {
        assert!(wakes(SUCCESS, HIGH));
        assert!(wakes(SUCCESS, MEDIUM));
        assert!(!wakes(SUCCESS, 2));
        assert!(!wakes(SUCCESS, 3));
        assert!(!wakes(TIMEOUT_EXCEEDED, HIGH));
        assert!(WAKE_PHRASES.iter().all(|phrase| phrase.ends_with("sens") || phrase.ends_with("sense")));
    }

    #[test]
    fn what_is_heard_reaches_the_interface_tagged_by_kind() {
        let guess = serde_json::to_value(Heard::Guess { id: 3, text: "hola".into() }).unwrap();
        assert_eq!(guess, serde_json::json!({ "kind": "guess", "id": 3, "text": "hola" }));
        let ended = serde_json::to_value(Heard::Ended { id: 3, refusal: Some(speech_off()) }).unwrap();
        assert_eq!(ended["kind"], "ended");
        assert_eq!(ended["refusal"]["cause"], "speech");
        assert_eq!(serde_json::to_value(Heard::Woke).unwrap(), serde_json::json!({ "kind": "woke" }));
        assert_eq!(serde_json::from_value::<Cause>(serde_json::json!("microphone")).unwrap(), Cause::Microphone);
    }

    #[test]
    fn stopping_what_never_started_is_harmless() {
        let dictation = Dictation::new(|_| {});
        dictation.stop();
        assert!(dictation.wake(None).is_ok());
        dictation.shutdown();
    }
}
