use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use sens_agent::language::Language;
use serde::{Deserialize, Serialize};
use tauri::{
    App, AppHandle, Emitter, LogicalSize, Manager, Monitor, PhysicalPosition, PhysicalRect, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window,
    WindowEvent,
};

use crate::{data_dir, front, language, life, look, projects, spots};

pub const LABEL: &str = "bar";
const HOST: &str = "main";
const WIDTH: f64 = 688.0;
const FIRST_HEIGHT: f64 = 106.0;
const LEAST_HEIGHT: f64 = 90.0;
const MOST_HEIGHT: f64 = 680.0;
const FROM_TOP: f64 = 0.22;
const MINIMIZING: Duration = Duration::from_millis(240);
const SETTLING: Duration = Duration::from_millis(400);

#[derive(Default)]
struct Pin {
    on: AtomicBool,
    dragging: AtomicBool,
    moves: AtomicU64,
}

#[derive(Serialize, Clone)]
struct Opened {
    look: look::Look,
    language: Option<Language>,
    front: Option<front::Front>,
    pinned: bool,
    resume: Option<HandOver>,
    listen: bool,
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct HandOver {
    root: Option<String>,
    session: Option<String>,
    text: String,
}

#[derive(Serialize)]
pub struct Context {
    front: Option<front::Front>,
    clip: Option<front::Clip>,
}

pub fn build(app: &App) -> tauri::Result<()> {
    app.manage(Pin::default());
    let base = data_dir(app.handle()).ok();
    let look = base.as_deref().map(look::load).unwrap_or_default();
    let bar = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("bar.html".into()))
        .title("Sens")
        .inner_size(WIDTH, FIRST_HEIGHT)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .focused(false)
        .theme(look.theme())
        .initialization_script(look.script())
        .initialization_script(language::script(base.as_deref().and_then(language::load)))
        .build()?;
    tool::keep(&bar);
    Ok(())
}

pub fn toggle(app: &AppHandle) {
    let Some(bar) = app.get_webview_window(LABEL) else {
        return;
    };
    if bar.is_visible().unwrap_or(false) && bar.is_focused().unwrap_or(false) {
        let _ = bar.hide();
        return;
    }
    reveal(app, &bar);
}

pub fn open(app: &AppHandle) {
    front::note();
    if let Some(bar) = app.get_webview_window(LABEL) {
        reveal(app, &bar);
    }
}

pub fn wake(app: &AppHandle) {
    let Some(bar) = app.get_webview_window(LABEL) else {
        return;
    };
    if !bar.is_visible().unwrap_or(false) {
        front::note();
    }
    reveal_with(app, &bar, None, true);
}

fn reveal(app: &AppHandle, bar: &WebviewWindow) {
    reveal_with(app, bar, None, false);
}

fn reveal_with(app: &AppHandle, bar: &WebviewWindow, resume: Option<HandOver>, listen: bool) {
    let base = data_dir(app).ok();
    let look = base.as_deref().map(look::load).unwrap_or_default();
    let pin = app.state::<Pin>();
    let pinned = pin.on.load(Ordering::Relaxed);
    let _ = bar.set_theme(look.theme());
    pin.dragging.store(false, Ordering::Relaxed);
    if !pinned {
        place(app, bar);
    }
    let opened = Opened {
        look,
        language: base.as_deref().and_then(language::load),
        front: front::front(),
        pinned,
        resume,
        listen,
    };
    let _ = app.emit_to(LABEL, "bar-open", opened);
    let _ = bar.show();
    let _ = bar.set_focus();
}

fn place(app: &AppHandle, bar: &WebviewWindow) {
    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|at| app.monitor_from_point(at.x, at.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return;
    };
    let (area, scale) = (monitor.work_area(), monitor.scale_factor());
    let saved = data_dir(app).ok().zip(monitor.name()).and_then(|(base, screen)| spots::spot_of(&base, screen));
    let size = bar.outer_size().unwrap_or_else(|_| PhysicalSize::new((WIDTH * scale) as u32, (FIRST_HEIGHT * scale) as u32));
    let at = saved.map(|kept| spots::settled(area, scale, kept, size)).unwrap_or_else(|| spot(area, scale));
    let _ = bar.set_position(at);
}

fn remember(app: &AppHandle, monitor: &Monitor, at: Option<PhysicalPosition<i32>>) {
    let (Ok(base), Some(screen)) = (data_dir(app), monitor.name()) else {
        return;
    };
    let _ = spots::keep(&base, screen, at.map(|at| spots::offset(monitor.work_area(), monitor.scale_factor(), at)));
}

fn settle(window: Window, round: u64) {
    std::thread::sleep(SETTLING);
    let pin = window.state::<Pin>();
    if pin.moves.load(Ordering::Relaxed) != round || !pin.dragging.swap(false, Ordering::Relaxed) {
        return;
    }
    if let (Ok(Some(monitor)), Ok(at)) = (window.current_monitor(), window.outer_position()) {
        remember(window.app_handle(), &monitor, Some(at));
    }
}

pub fn keep_on_top(app: &AppHandle) {
    let Some(bar) = app.get_webview_window(LABEL) else {
        return;
    };
    if app.state::<Pin>().on.load(Ordering::Relaxed) && bar.is_visible().unwrap_or(false) {
        tool::raise(&bar);
    }
}

fn spot(area: &PhysicalRect<i32, u32>, scale: f64) -> PhysicalPosition<i32> {
    let x = f64::from(area.position.x) + (f64::from(area.size.width) - WIDTH * scale) / 2.0;
    let y = f64::from(area.position.y) + f64::from(area.size.height) * FROM_TOP;
    PhysicalPosition::new(x.round() as i32, y.round() as i32)
}

fn fitted(height: f64) -> Option<f64> {
    (!height.is_nan()).then(|| height.clamp(LEAST_HEIGHT, MOST_HEIGHT))
}

fn lifted(top: i32, tall: f64, area: &PhysicalRect<i32, u32>) -> i32 {
    let bottom = area.position.y + area.size.height as i32;
    top.min(bottom - tall.round() as i32).max(area.position.y)
}

pub fn heard(window: &Window, event: &WindowEvent) {
    if window.label() != LABEL {
        return;
    }
    match event {
        WindowEvent::Focused(false) if !window.state::<Pin>().on.load(Ordering::Relaxed) => {
            let _ = window.hide();
        }
        WindowEvent::Moved(_) if window.state::<Pin>().dragging.load(Ordering::Relaxed) => {
            let round = window.state::<Pin>().moves.fetch_add(1, Ordering::Relaxed) + 1;
            let moved = window.clone();
            std::thread::spawn(move || settle(moved, round));
        }
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            let _ = window.hide();
        }
        _ => {}
    }
}

#[tauri::command]
pub fn bar_open(app: AppHandle) {
    open(&app);
}

#[tauri::command]
pub fn bar_focus(app: AppHandle, hand: HandOver) {
    let Some(main) = app.get_webview_window(HOST) else {
        return;
    };
    let _ = main.minimize();
    std::thread::spawn(move || {
        std::thread::sleep(MINIMIZING);
        let shown = app.clone();
        let _ = app.run_on_main_thread(move || {
            let _ = main.hide();
            front::note();
            if let Some(bar) = shown.get_webview_window(LABEL) {
                reveal_with(&shown, &bar, Some(hand), false);
            }
        });
    });
}

#[tauri::command]
pub fn bar_hide(app: AppHandle) {
    if let Some(bar) = app.get_webview_window(LABEL) {
        let _ = bar.hide();
    }
}

#[tauri::command]
pub fn bar_fit(app: AppHandle, height: f64) {
    let (Some(bar), Some(height)) = (app.get_webview_window(LABEL), fitted(height)) else {
        return;
    };
    let Ok(Some(monitor)) = bar.current_monitor() else {
        let _ = bar.set_size(LogicalSize::new(WIDTH, height));
        return;
    };
    let (area, scale) = (monitor.work_area(), monitor.scale_factor());
    let height = height.min(f64::from(area.size.height) / scale);
    let _ = bar.set_size(LogicalSize::new(WIDTH, height));
    if let Ok(at) = bar.outer_position() {
        let top = lifted(at.y, height * scale, area);
        if top != at.y {
            let _ = bar.set_position(PhysicalPosition::new(at.x, top));
        }
    }
}

#[tauri::command]
pub fn bar_pin(app: AppHandle, on: bool) {
    app.state::<Pin>().on.store(on, Ordering::Relaxed);
    keep_on_top(&app);
}

#[tauri::command]
pub fn bar_drag(app: AppHandle) {
    let Some(bar) = app.get_webview_window(LABEL) else {
        return;
    };
    app.state::<Pin>().dragging.store(true, Ordering::Relaxed);
    let _ = bar.start_dragging();
}

#[tauri::command]
pub fn bar_recenter(app: AppHandle) {
    let Some(bar) = app.get_webview_window(LABEL) else {
        return;
    };
    let Ok(Some(monitor)) = bar.current_monitor() else {
        return;
    };
    remember(&app, &monitor, None);
    let _ = bar.set_position(spot(monitor.work_area(), monitor.scale_factor()));
}

#[tauri::command]
pub fn bar_hand_over(app: AppHandle, hand: HandOver) {
    bar_hide(app.clone());
    life::show(&app);
    let _ = app.emit_to(HOST, "bar-hand-over", hand);
}

#[tauri::command(async)]
pub fn bar_context() -> Context {
    Context {
        front: front::front(),
        clip: front::clip(),
    }
}

#[tauri::command(async)]
pub fn bar_clip() -> Option<String> {
    front::clipboard()
}

#[tauri::command(async)]
pub fn bar_shot() -> Option<front::Shot> {
    front::shot()
}

#[tauri::command(async)]
pub fn bar_projects(app: AppHandle) -> Result<Vec<projects::Recent>, String> {
    Ok(projects::recent(&projects::load(&data_dir(&app)?)))
}

#[cfg(windows)]
mod tool {
    use tauri::WebviewWindow;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, HWND_TOPMOST, STYLESTRUCT, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowLongPtrW, SetWindowPos, WM_STYLECHANGING,
        WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    };

    const SUBCLASS: usize = 0x5345_4e53;

    pub fn keep(bar: &WebviewWindow) {
        let Ok(window) = bar.hwnd() else {
            return;
        };
        unsafe {
            let _ = SetWindowSubclass(window, Some(stays), SUBCLASS, 0);
            SetWindowLongPtrW(window, GWL_EXSTYLE, style(GetWindowLongPtrW(window, GWL_EXSTYLE) as u32) as isize);
        }
    }

    pub fn raise(bar: &WebviewWindow) {
        if let Ok(window) = bar.hwnd() {
            let _ = unsafe { SetWindowPos(window, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) };
        }
    }

    unsafe extern "system" fn stays(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM, _subclass: usize, _data: usize) -> LRESULT {
        if message == WM_STYLECHANGING && wparam.0 as i32 == GWL_EXSTYLE.0 {
            let change = unsafe { &mut *(lparam.0 as *mut STYLESTRUCT) };
            change.styleNew = style(change.styleNew);
        }
        unsafe { DefSubclassProc(window, message, wparam, lparam) }
    }

    pub fn style(extended: u32) -> u32 {
        (extended | WS_EX_TOOLWINDOW.0) & !WS_EX_APPWINDOW.0
    }
}

#[cfg(not(windows))]
mod tool {
    use tauri::WebviewWindow;

    pub fn keep(_bar: &WebviewWindow) {}

    pub fn raise(_bar: &WebviewWindow) {}
}

#[cfg(test)]
mod tests {
    use tauri::PhysicalSize;

    use super::*;

    fn area(x: i32, y: i32, width: u32, height: u32) -> PhysicalRect<i32, u32> {
        PhysicalRect {
            position: PhysicalPosition::new(x, y),
            size: PhysicalSize::new(width, height),
        }
    }

    #[test]
    fn the_bar_opens_centred_at_22_percent_of_the_work_area() {
        assert_eq!(spot(&area(0, 0, 1920, 1032), 1.0), PhysicalPosition::new(616, 227));
        assert_eq!(spot(&area(0, 0, 1920, 1008), 1.5), PhysicalPosition::new(444, 222));
        assert_eq!(spot(&area(-2560, 40, 2560, 1400), 1.25), PhysicalPosition::new(-1710, 348));
    }

    #[test]
    fn a_fit_ignores_nan_and_stays_between_90_and_680() {
        assert_eq!(fitted(f64::NAN), None);
        assert_eq!(fitted(40.0), Some(90.0));
        assert_eq!(fitted(300.5), Some(300.5));
        assert_eq!(fitted(4000.0), Some(680.0));
        assert_eq!(fitted(f64::INFINITY), Some(680.0));
        assert_eq!(fitted(f64::NEG_INFINITY), Some(90.0));
    }

    #[test]
    fn a_bar_that_grows_past_the_work_area_rises_but_never_above_it() {
        let work = area(0, 0, 1920, 1008);
        assert_eq!(lifted(222, 600.0, &work), 222);
        assert_eq!(lifted(222, 1008.0, &work), 0);
        assert_eq!(lifted(700, 450.0, &work), 558);
        assert_eq!(lifted(-20, 100.0, &area(0, 40, 1920, 1000)), 40);
    }

    #[test]
    fn a_hand_over_without_a_session_still_carries_its_text() {
        let hand: HandOver = serde_json::from_str(r#"{ "root": "C:/sens", "session": null, "text": "sigue" }"#).unwrap();
        let sent = serde_json::to_value(&hand).unwrap();
        assert_eq!(sent, serde_json::json!({ "root": "C:/sens", "session": null, "text": "sigue" }));
        assert_eq!(serde_json::to_value(serde_json::from_str::<HandOver>("{}").unwrap()).unwrap(), serde_json::json!({ "root": null, "session": null, "text": "" }));
    }

    #[cfg(windows)]
    #[test]
    fn the_bar_is_a_tool_window_that_never_asks_for_a_taskbar_button() {
        use windows::Win32::UI::WindowsAndMessaging::{WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST};

        assert_eq!(tool::style((WS_EX_APPWINDOW | WS_EX_TOPMOST).0), (WS_EX_TOOLWINDOW | WS_EX_TOPMOST).0);
        assert_eq!(tool::style(WS_EX_TOOLWINDOW.0), WS_EX_TOOLWINDOW.0);
    }
}
