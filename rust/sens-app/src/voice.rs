use sens_agent::said;
use tauri::AppHandle;

fn elsewhere() -> String {
    said!(
        en: "Sens isn’t the window in front, so voice typing stays closed",
        es: "Sens no es la ventana de delante, así que la escritura por voz sigue cerrada",
        fr: "Sens n’est pas la fenêtre au premier plan, la saisie vocale reste donc fermée",
        de: "Sens ist nicht das vordere Fenster, deshalb bleibt die Spracheingabe zu",
        ja: "Sens が最前面のウィンドウではないため、音声入力は開きません",
        zh: "Sens 不是最前面的窗口，因此不会打开语音输入",
    )
}

fn refused(error: &str) -> String {
    said!(
        en: "Windows didn’t open voice typing: {error}",
        es: "Windows no abrió la escritura por voz: {error}",
        fr: "Windows n’a pas ouvert la saisie vocale : {error}",
        de: "Windows hat die Spracheingabe nicht geöffnet: {error}",
        ja: "Windows が音声入力を開きませんでした: {error}",
        zh: "Windows 未打开语音输入：{error}",
    )
}

#[cfg(windows)]
pub fn open(app: &AppHandle) -> Result<(), String> {
    use tauri::Manager;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY, VK_H, VK_LWIN,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    let window = app.get_window("main").ok_or_else(elsewhere)?;
    let hwnd = window.hwnd().map_err(|error| refused(&error.to_string()))?;
    if unsafe { GetForegroundWindow() } != hwnd {
        return Err(elsewhere());
    }
    let key = |vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: vk, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 } },
    };
    let chord = [key(VK_LWIN, KEYBD_EVENT_FLAGS(0)), key(VK_H, KEYBD_EVENT_FLAGS(0)), key(VK_H, KEYEVENTF_KEYUP), key(VK_LWIN, KEYEVENTF_KEYUP)];
    let sent = unsafe { SendInput(&chord, std::mem::size_of::<INPUT>() as i32) };
    if sent as usize == chord.len() {
        Ok(())
    } else {
        Err(refused(&windows::core::Error::from_win32().message()))
    }
}

#[cfg(not(windows))]
pub fn open(_app: &AppHandle) -> Result<(), String> {
    Err(refused("Windows only"))
}
