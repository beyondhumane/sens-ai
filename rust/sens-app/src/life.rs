use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

use sens_agent::chat::Engine;
use sens_agent::said;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Window, WindowEvent, Wry};

use crate::shortcut::Keys;
use crate::{bar, data_dir, profile, terminal, voice};

const HOST: &str = "main";
const HIDDEN: &str = "--hidden";
const TRAY: &str = "sens";
const OPEN: &str = "open";
const BAR: &str = "bar";
const QUIT: &str = "quit";

static TAKEN: AtomicBool = AtomicBool::new(false);
static BOUND: LazyLock<Mutex<Keys>> = LazyLock::new(Default::default);

pub enum Wish {
    Bind(Keys),
    Pause,
    Resume,
}

struct Tray {
    open: MenuItem<Wry>,
    bar: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

pub fn hidden(args: impl IntoIterator<Item = String>) -> bool {
    args.into_iter().any(|arg| arg == HIDDEN)
}

pub fn script(hidden: bool) -> String {
    format!("window.__SENS_HIDDEN__ = {hidden};")
}

pub fn claim(hidden: bool) -> bool {
    native::claim(hidden)
}

pub fn listen(app: &AppHandle) {
    native::listen(app);
}

pub fn shortcut_taken() -> bool {
    TAKEN.load(Ordering::Relaxed)
}

fn bound() -> MutexGuard<'static, Keys> {
    BOUND.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn shortcut() -> Keys {
    bound().clone()
}

pub fn rebind(wish: Wish) -> bool {
    native::rebind(wish)
}

pub fn show(app: &AppHandle) {
    let Some(main) = app.get_webview_window(HOST) else {
        return;
    };
    let _ = main.unminimize();
    let _ = main.show();
    let _ = main.set_focus();
}

pub fn quit(app: &AppHandle) -> ! {
    app.state::<Arc<Engine>>().shutdown();
    app.state::<terminal::Consoles>().shutdown();
    if let Some(voice) = app.try_state::<voice::Voice>() {
        voice.stop();
    }
    app.cleanup_before_exit();
    std::process::exit(0)
}

pub fn closing(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    if window.label() != HOST {
        return;
    }
    let app = window.app_handle();
    if !data_dir(app).map(|base| profile::load(&base).keep_in_tray).unwrap_or(true) {
        quit(app)
    }
    api.prevent_close();
    let _ = window.hide();
}

pub fn start_with_windows(on: bool) {
    if cfg!(debug_assertions) {
        return;
    }
    if !on {
        native::erase_run();
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let wanted = run_value(&exe);
    if native::run_entry().as_deref() != Some(wanted.as_str()) {
        native::write_run(&wanted);
    }
}

fn run_value(exe: &Path) -> String {
    format!("\"{}\" {HIDDEN}", exe.display())
}

pub fn tray(app: &AppHandle) -> tauri::Result<()> {
    let [open, bar, quit] = labels();
    let tray = Tray {
        open: MenuItem::with_id(app, OPEN, open, true, None::<&str>)?,
        bar: MenuItem::with_id(app, BAR, bar, true, None::<&str>)?,
        quit: MenuItem::with_id(app, QUIT, quit, true, None::<&str>)?,
    };
    let menu = Menu::with_items(app, &[&tray.open, &tray.bar, &PredefinedMenuItem::separator(app)?, &tray.quit])?;
    let mut icon = TrayIconBuilder::with_id(TRAY)
        .tooltip("Sens")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(chosen)
        .on_tray_icon_event(clicked);
    if let Some(image) = app.default_window_icon() {
        icon = icon.icon(image.clone());
    }
    icon.build(app)?;
    app.manage(tray);
    Ok(())
}

pub fn retitle(app: &AppHandle) {
    let Some(tray) = app.try_state::<Tray>() else {
        return;
    };
    let [open, bar, quit] = labels();
    let _ = tray.open.set_text(open);
    let _ = tray.bar.set_text(bar);
    let _ = tray.quit.set_text(quit);
}

fn chosen(app: &AppHandle, event: MenuEvent) {
    match event.id().0.as_str() {
        OPEN => show(app),
        BAR => bar::open(app),
        QUIT => quit(app),
        _ => {}
    }
}

fn clicked(icon: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        show(icon.app_handle());
    }
}

fn labels() -> [String; 3] {
    [
        said!(
            en: "Open Sens",
            es: "Abrir Sens",
            fr: "Ouvrir Sens",
            de: "Sens öffnen",
            ja: "Sens を開く",
            zh: "打开 Sens",
        ),
        said!(
            en: "Focus mode ({keys})",
            es: "Modo focus ({keys})",
            fr: "Mode focus ({keys})",
            de: "Fokusmodus ({keys})",
            ja: "フォーカスモード（{keys}）",
            zh: "专注模式（{keys}）",
            keys = shortcut().named(),
        ),
        said!(
            en: "Quit",
            es: "Salir",
            fr: "Quitter",
            de: "Beenden",
            ja: "終了",
            zh: "退出",
        ),
    ]
}

#[cfg(windows)]
mod native {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Mutex, OnceLock, mpsc};
    use std::time::Duration;

    use tauri::AppHandle;
    use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::DataExchange::AddClipboardFormatListener;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW};
    use windows::Win32::System::Threading::{CreateMutexW, GetCurrentThreadId};
    use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook};
    use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN, RegisterHotKey, UnregisterHotKey};
    use windows::Win32::UI::WindowsAndMessaging::{
        AllowSetForegroundWindow, CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowExW, GetMessageW, GetWindowThreadProcessId, HWND_MESSAGE, MSG,
        EVENT_SYSTEM_FOREGROUND, PostMessageW, PostThreadMessageW, RegisterClassW, RegisterWindowMessageW, WINDOW_EX_STYLE, WINDOW_STYLE, WINEVENT_OUTOFCONTEXT,
        WINEVENT_SKIPOWNPROCESS, WM_APP, WM_CLIPBOARDUPDATE, WM_HOTKEY, WNDCLASSW,
    };
    use windows::core::{PCWSTR, w};

    use super::{TAKEN, Wish, bound, quit, show};
    use crate::shortcut::Keys;
    use crate::{bar, data_dir, front, profile};

    const SHORTCUT: i32 = 1;
    const REBIND: u32 = WM_APP + 1;
    const ANSWER_WAIT: Duration = Duration::from_secs(2);

    static THREAD: AtomicU32 = AtomicU32::new(0);
    static WISH: Mutex<Option<(Wish, mpsc::Sender<bool>)>> = Mutex::new(None);
    const WAKE_TRIES: u32 = 30;
    const WAKE_PAUSE: Duration = Duration::from_millis(100);

    static APP: OnceLock<AppHandle> = OnceLock::new();

    struct Signals {
        show: u32,
        quit: u32,
    }

    fn signals() -> &'static Signals {
        static SIGNALS: OnceLock<Signals> = OnceLock::new();
        SIGNALS.get_or_init(|| unsafe {
            Signals {
                show: RegisterWindowMessageW(w!("SensShow")),
                quit: RegisterWindowMessageW(w!("SensQuit")),
            }
        })
    }

    fn listener() -> PCWSTR {
        if cfg!(debug_assertions) { w!("SensListenerDev") } else { w!("SensListener") }
    }

    fn run_key() -> PCWSTR {
        w!(r"Software\Microsoft\Windows\CurrentVersion\Run")
    }

    fn run_name() -> PCWSTR {
        w!("Sens")
    }

    pub fn claim(hidden: bool) -> bool {
        let name = if cfg!(debug_assertions) { w!(r"Local\SensDesktopDev") } else { w!(r"Local\SensDesktop") };
        let alone = match unsafe { CreateMutexW(None, false, name) } {
            Ok(_) => (unsafe { GetLastError() }) != ERROR_ALREADY_EXISTS,
            Err(_) => true,
        };
        if !alone && !hidden {
            wake();
        }
        alone
    }

    fn wake() {
        for attempt in 0..WAKE_TRIES {
            if attempt > 0 {
                std::thread::sleep(WAKE_PAUSE);
            }
            let Ok(window) = (unsafe { FindWindowExW(Some(HWND_MESSAGE), None, listener(), PCWSTR::null()) }) else {
                continue;
            };
            let mut process = 0u32;
            unsafe {
                GetWindowThreadProcessId(window, Some(&mut process));
                let _ = AllowSetForegroundWindow(process);
                let _ = PostMessageW(Some(window), signals().show, WPARAM(0), LPARAM(0));
            }
            return;
        }
    }

    pub fn listen(app: &AppHandle) {
        if APP.set(app.clone()).is_ok() {
            std::thread::spawn(listening);
        }
    }

    fn listening() {
        signals();
        let instance: Option<HINSTANCE> = unsafe { GetModuleHandleW(PCWSTR::null()) }.ok().map(Into::into);
        let class = WNDCLASSW {
            lpfnWndProc: Some(heard),
            hInstance: instance.unwrap_or_default(),
            lpszClassName: listener(),
            ..Default::default()
        };
        let window = unsafe {
            RegisterClassW(&class);
            CreateWindowExW(WINDOW_EX_STYLE::default(), listener(), PCWSTR::null(), WINDOW_STYLE::default(), 0, 0, 0, 0, Some(HWND_MESSAGE), None, instance, None).ok()
        };
        if let Some(window) = window {
            let _ = unsafe { AddClipboardFormatListener(window) };
        }
        THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
        let chosen = APP.get().and_then(|app| data_dir(app).ok()).map(|base| profile::load(&base).shortcut).unwrap_or_default();
        TAKEN.store(!bind(&chosen), Ordering::Relaxed);
        *bound() = chosen;
        let _ = unsafe { SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, None, Some(fronted), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS) };
        let mut message = MSG::default();
        while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
            if message.hwnd.0.is_null() {
                respond(message.message);
            } else {
                unsafe { DispatchMessageW(&message) };
            }
        }
    }

    unsafe extern "system" fn fronted(_hook: HWINEVENTHOOK, _event: u32, _window: HWND, _object: i32, _child: i32, _thread: u32, _time: u32) {
        if let Some(app) = APP.get() {
            on_main(app, bar::keep_on_top);
        }
    }

    fn bind(keys: &Keys) -> bool {
        let Some(key) = keys.virtual_key() else {
            return false;
        };
        let modifiers = [(keys.ctrl, MOD_CONTROL), (keys.alt, MOD_ALT), (keys.shift, MOD_SHIFT), (keys.win, MOD_WIN)]
            .into_iter()
            .filter(|(held, _)| *held)
            .fold(MOD_NOREPEAT, |all, (_, modifier)| all | modifier);
        unsafe { RegisterHotKey(None, SHORTCUT, modifiers, key) }.is_ok()
    }

    pub fn rebind(wish: Wish) -> bool {
        let thread = THREAD.load(Ordering::SeqCst);
        if thread == 0 {
            return false;
        }
        let (answer, answered) = mpsc::channel();
        *WISH.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some((wish, answer));
        if unsafe { PostThreadMessageW(thread, REBIND, WPARAM(0), LPARAM(0)) }.is_err() {
            return false;
        }
        answered.recv_timeout(ANSWER_WAIT).unwrap_or(false)
    }

    fn rebound() {
        let Some((wish, answer)) = WISH.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).take() else {
            return;
        };
        let _ = unsafe { UnregisterHotKey(None, SHORTCUT) };
        let done = match wish {
            Wish::Pause => true,
            Wish::Resume => {
                let on = bind(&bound());
                TAKEN.store(!on, Ordering::Relaxed);
                on
            }
            Wish::Bind(keys) if bind(&keys) => {
                TAKEN.store(false, Ordering::Relaxed);
                *bound() = keys;
                true
            }
            Wish::Bind(_) => {
                TAKEN.store(!bind(&bound()), Ordering::Relaxed);
                false
            }
        };
        let _ = answer.send(done);
    }

    unsafe extern "system" fn heard(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if respond(message) {
            return LRESULT(0);
        }
        unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }

    fn respond(message: u32) -> bool {
        let Some(app) = APP.get() else {
            return false;
        };
        let signals = signals();
        match message {
            0 => return false,
            WM_CLIPBOARDUPDATE => front::copied(),
            REBIND => rebound(),
            WM_HOTKEY => {
                front::note();
                on_main(app, bar::toggle);
            }
            _ if message == signals.show => on_main(app, show),
            _ if message == signals.quit => on_main(app, |app| quit(app)),
            _ => return false,
        }
        true
    }

    fn on_main(app: &AppHandle, work: fn(&AppHandle)) {
        let heard = app.clone();
        let _ = app.run_on_main_thread(move || work(&heard));
    }

    pub fn run_entry() -> Option<String> {
        let mut size = 0u32;
        let found = unsafe { RegGetValueW(HKEY_CURRENT_USER, run_key(), run_name(), RRF_RT_REG_SZ, None, None, Some(&mut size)) };
        if found.is_err() {
            return None;
        }
        let mut text = vec![0u16; (size as usize).div_ceil(size_of::<u16>())];
        let read = unsafe { RegGetValueW(HKEY_CURRENT_USER, run_key(), run_name(), RRF_RT_REG_SZ, None, Some(text.as_mut_ptr().cast()), Some(&mut size)) };
        read.is_ok().then(|| String::from_utf16_lossy(&text).trim_end_matches('\0').to_string())
    }

    pub fn write_run(value: &str) {
        let wide: Vec<u16> = value.encode_utf16().chain([0]).collect();
        let bytes = (wide.len() * size_of::<u16>()) as u32;
        let _ = unsafe { RegSetKeyValueW(HKEY_CURRENT_USER, run_key(), run_name(), REG_SZ.0, Some(wide.as_ptr().cast()), bytes) };
    }

    pub fn erase_run() {
        let _ = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, run_key(), run_name()) };
    }
}

#[cfg(not(windows))]
mod native {
    use tauri::AppHandle;

    pub fn claim(_hidden: bool) -> bool {
        true
    }

    pub fn listen(_app: &AppHandle) {}

    pub fn rebind(_wish: super::Wish) -> bool {
        false
    }

    pub fn run_entry() -> Option<String> {
        None
    }

    pub fn write_run(_value: &str) {}

    pub fn erase_run() {}
}

#[cfg(test)]
mod tests {
    use sens_agent::language::{Language, speaking};

    use super::*;

    fn arguments(given: &[&str]) -> Vec<String> {
        given.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn only_the_exact_hidden_flag_starts_sens_without_a_window() {
        assert!(hidden(arguments(&["Sens.exe", "--hidden"])));
        assert!(hidden(arguments(&["Sens.exe", "--otra", "--hidden"])));
        assert!(!hidden(arguments(&["Sens.exe"])));
        assert!(!hidden(arguments(&["Sens.exe", "--hiddenly", "-hidden", "--HIDDEN"])));
    }

    #[test]
    fn the_page_learns_before_it_draws_whether_to_stay_hidden() {
        assert_eq!(script(true), "window.__SENS_HIDDEN__ = true;");
        assert_eq!(script(false), "window.__SENS_HIDDEN__ = false;");
    }

    #[test]
    fn the_run_value_quotes_the_program_so_spaces_survive() {
        assert_eq!(run_value(Path::new(r"C:\Program Files\Sens\Sens.exe")), r#""C:\Program Files\Sens\Sens.exe" --hidden"#);
    }

    #[test]
    fn the_tray_menu_speaks_the_language_chosen() {
        assert_eq!(speaking(Language::Es, labels), ["Abrir Sens", "Modo focus (Alt+Espacio)", "Salir"]);
        assert_eq!(speaking(Language::En, labels), ["Open Sens", "Focus mode (Alt+Space)", "Quit"]);
        assert_eq!(speaking(Language::De, labels)[1], "Fokusmodus (Alt+Leertaste)");
    }
}
