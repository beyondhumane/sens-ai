use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use serde::Serialize;

const FRESH: Duration = Duration::from_secs(30);
const MOST_CHARS: usize = 20_000;
const PREVIEW_CHARS: usize = 80;

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct Front {
    pub app: String,
    pub title: String,
}

#[derive(Serialize, PartialEq, Debug)]
pub struct Clip {
    preview: String,
    chars: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shot {
    media_type: &'static str,
    data: String,
    width: u32,
    height: u32,
}

struct Seen {
    window: isize,
    front: Front,
}

static SEEN: Mutex<Option<Seen>> = Mutex::new(None);
static COPIED: Mutex<Option<Instant>> = Mutex::new(None);

pub fn note() {
    *SEEN.lock().unwrap_or_else(PoisonError::into_inner) = native::foreground();
}

pub fn front() -> Option<Front> {
    SEEN.lock().unwrap_or_else(PoisonError::into_inner).as_ref().map(|seen| seen.front.clone())
}

pub fn shot() -> Option<Shot> {
    let window = SEEN.lock().unwrap_or_else(PoisonError::into_inner).as_ref()?.window;
    native::shot(window)
}

pub fn copied() {
    *COPIED.lock().unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
}

pub fn clip() -> Option<Clip> {
    let copied = *COPIED.lock().unwrap_or_else(PoisonError::into_inner);
    if !fresh(copied, Instant::now()) {
        return None;
    }
    clipboard().map(|text| summary(&text))
}

pub fn clipboard() -> Option<String> {
    native::clipboard()
}

fn fresh(copied: Option<Instant>, now: Instant) -> bool {
    copied.is_some_and(|at| now.saturating_duration_since(at) < FRESH)
}

fn summary(text: &str) -> Clip {
    Clip {
        preview: text.split_whitespace().flat_map(|word| [" ", word]).skip(1).flat_map(str::chars).take(PREVIEW_CHARS).collect(),
        chars: text.chars().count(),
    }
}

#[cfg(windows)]
mod native {
    use std::ffi::c_void;
    use std::path::Path;
    use std::time::Duration;

    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use windows::Win32::Foundation::{HGLOBAL, HWND, RECT};
    use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS, DeleteDC, DeleteObject, GdiFlush, GetDC, HALFTONE,
        HBITMAP, HDC, ReleaseDC, SRCCOPY, SelectObject, SetBrushOrgEx, SetStretchBltMode, StretchBlt,
    };
    use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow};
    use windows::Win32::System::DataExchange::{CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard};
    use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    use windows::Win32::System::Threading::{GetCurrentProcessId, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW};
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetShellWindow, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsHungAppWindow, IsIconic, IsWindow,
        PW_RENDERFULLCONTENT,
    };
    use windows::core::{Owned, PWSTR};

    use super::{Front, MOST_CHARS, Seen, Shot};

    const LONGEST_EDGE: u32 = 1600;
    const UWP_HOST: &str = "ApplicationFrameHost";
    const SHELL_CLASSES: [&str; 5] = ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd", "NotifyIconOverflowWindow"];
    const UNICODE_TEXT: u32 = 13;
    const OPEN_TRIES: u32 = 5;
    const OPEN_PAUSE: Duration = Duration::from_millis(10);

    pub fn foreground() -> Option<Seen> {
        let window = unsafe { GetForegroundWindow() };
        if window.0.is_null() || window == unsafe { GetShellWindow() } || shell(&class_of(window)) || cloaked(window) {
            return None;
        }
        let mut process = 0u32;
        unsafe { GetWindowThreadProcessId(window, Some(&mut process)) };
        if process == unsafe { GetCurrentProcessId() } {
            return None;
        }
        let title = title_of(window);
        let app = label(image_of(process).as_deref(), &title)?;
        Some(Seen {
            window: window.0 as isize,
            front: Front { app, title },
        })
    }

    pub fn shot(window: isize) -> Option<Shot> {
        let (width, height, bgra) = capture(HWND(window as *mut c_void))?;
        picture(width, height, &bgra)
    }

    pub fn clipboard() -> Option<String> {
        unsafe { IsClipboardFormatAvailable(UNICODE_TEXT) }.ok()?;
        let opened = (0..OPEN_TRIES).any(|attempt| {
            if attempt > 0 {
                std::thread::sleep(OPEN_PAUSE);
            }
            unsafe { OpenClipboard(None) }.is_ok()
        });
        if !opened {
            return None;
        }
        let text = unsafe { GetClipboardData(UNICODE_TEXT) }.ok().and_then(|handle| {
            let memory = HGLOBAL(handle.0);
            let wide = unsafe { GlobalLock(memory) } as *const u16;
            if wide.is_null() {
                return None;
            }
            let units = unsafe { GlobalSize(memory) } / size_of::<u16>();
            let text = decoded(unsafe { std::slice::from_raw_parts(wide, units) });
            let _ = unsafe { GlobalUnlock(memory) };
            text
        });
        let _ = unsafe { CloseClipboard() };
        text
    }

    fn class_of(window: HWND) -> String {
        let mut name = [0u16; 256];
        let length = unsafe { GetClassNameW(window, &mut name) }.max(0) as usize;
        String::from_utf16_lossy(&name[..length])
    }

    fn title_of(window: HWND) -> String {
        let mut text = [0u16; 512];
        let length = unsafe { GetWindowTextW(window, &mut text) }.max(0) as usize;
        String::from_utf16_lossy(&text[..length])
    }

    fn cloaked(window: HWND) -> bool {
        let mut cloaked = 0u32;
        let read = unsafe { DwmGetWindowAttribute(window, DWMWA_CLOAKED, (&raw mut cloaked).cast(), size_of::<u32>() as u32) };
        read.is_ok() && cloaked != 0
    }

    fn image_of(process: u32) -> Option<String> {
        let process = unsafe { Owned::new(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process).ok()?) };
        let mut path = [0u16; 1024];
        let mut size = path.len() as u32;
        unsafe { QueryFullProcessImageNameW(*process, PROCESS_NAME_WIN32, PWSTR(path.as_mut_ptr()), &mut size) }.ok()?;
        Some(String::from_utf16_lossy(&path[..size as usize]))
    }

    fn capture(window: HWND) -> Option<(u32, u32, Vec<u8>)> {
        let usable = unsafe { IsWindow(Some(window)).as_bool() && !IsIconic(window).as_bool() && !IsHungAppWindow(window).as_bool() };
        if !usable {
            return None;
        }
        let mut whole = RECT::default();
        unsafe { GetWindowRect(window, &mut whole) }.ok()?;
        let mut frame = whole;
        let _ = unsafe { DwmGetWindowAttribute(window, DWMWA_EXTENDED_FRAME_BOUNDS, (&raw mut frame).cast(), size_of::<RECT>() as u32) };
        let (width, height) = (frame.right - frame.left, frame.bottom - frame.top);
        if width <= 0 || height <= 0 {
            return None;
        }
        let (small_width, small_height) = fit(width as u32, height as u32);
        unsafe {
            let screen = GetDC(None);
            let source = CreateCompatibleDC(Some(screen));
            let sink = CreateCompatibleDC(Some(screen));
            let full = dib(screen, whole.right - whole.left, whole.bottom - whole.top);
            let small = dib(screen, small_width as i32, small_height as i32);
            let mut pixels = None;
            if let (Some((full, _)), Some((small, bits))) = (full, small) {
                let old_source = SelectObject(source, full.into());
                let old_sink = SelectObject(sink, small.into());
                if PrintWindow(window, source, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT)).as_bool() {
                    SetStretchBltMode(sink, HALFTONE);
                    let _ = SetBrushOrgEx(sink, 0, 0, None);
                    let (left, top) = (frame.left - whole.left, frame.top - whole.top);
                    let _ = StretchBlt(sink, 0, 0, small_width as i32, small_height as i32, Some(source), left, top, width, height, SRCCOPY);
                    let _ = GdiFlush();
                    pixels = Some(std::slice::from_raw_parts(bits, (small_width * small_height * 4) as usize).to_vec());
                }
                SelectObject(source, old_source);
                SelectObject(sink, old_sink);
            }
            for (bitmap, _) in [full, small].into_iter().flatten() {
                let _ = DeleteObject(bitmap.into());
            }
            let _ = DeleteDC(source);
            let _ = DeleteDC(sink);
            ReleaseDC(None, screen);
            pixels.map(|pixels| (small_width, small_height, pixels))
        }
    }

    fn dib(screen: HDC, width: i32, height: i32) -> Option<(HBITMAP, *const u8)> {
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = std::ptr::null_mut();
        let bitmap = unsafe { CreateDIBSection(Some(screen), &info, DIB_RGB_COLORS, &mut bits, None, 0) }.ok()?;
        Some((bitmap, bits.cast_const().cast()))
    }

    fn shell(class: &str) -> bool {
        SHELL_CLASSES.contains(&class)
    }

    fn label(exe: Option<&str>, title: &str) -> Option<String> {
        exe.and_then(|exe| Path::new(exe).file_stem())
            .map(|stem| stem.to_string_lossy().into_owned())
            .filter(|stem| !stem.is_empty() && !stem.eq_ignore_ascii_case(UWP_HOST))
            .or_else(|| Some(title.trim().to_string()).filter(|title| !title.is_empty()))
    }

    fn fit(width: u32, height: u32) -> (u32, u32) {
        let scale = (f64::from(LONGEST_EDGE) / f64::from(width.max(height))).min(1.0);
        let side = |length: u32| ((f64::from(length) * scale).round() as u32).max(1);
        (side(width), side(height))
    }

    fn picture(width: u32, height: u32, bgra: &[u8]) -> Option<Shot> {
        let rgb: Vec<u8> = bgra.as_chunks::<4>().0.iter().flat_map(|&[blue, green, red, _]| [red, green, blue]).collect();
        if rgb.iter().all(|byte| *byte == 0) {
            return None;
        }
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&rgb).ok()?;
        writer.finish().ok()?;
        Some(Shot {
            media_type: "image/png",
            data: STANDARD.encode(bytes),
            width,
            height,
        })
    }

    fn decoded(wide: &[u16]) -> Option<String> {
        let end = wide.iter().position(|unit| *unit == 0).unwrap_or(wide.len());
        let text: String = char::decode_utf16(wide[..end].iter().copied())
            .map(|unit| unit.unwrap_or(char::REPLACEMENT_CHARACTER))
            .take(MOST_CHARS)
            .collect();
        Some(text).filter(|text| !text.trim().is_empty())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn wide(text: &str) -> Vec<u16> {
            text.encode_utf16().collect()
        }

        #[test]
        fn the_chip_names_the_program_without_its_extension_and_uwp_apps_by_their_title() {
            assert_eq!(label(Some(r"C:\Program Files\Microsoft VS Code\Code.exe"), "main.rs - Sens").as_deref(), Some("Code"));
            assert_eq!(label(Some(r"C:\Windows\System32\ApplicationFrameHost.exe"), " Calculadora ").as_deref(), Some("Calculadora"));
            assert_eq!(label(None, "Sin programa").as_deref(), Some("Sin programa"));
            assert_eq!(label(Some(r"C:\Windows\System32\applicationframehost.EXE"), "   "), None);
        }

        #[test]
        fn the_desktop_and_the_taskbar_are_never_the_window_in_front() {
            for class in ["Progman", "WorkerW", "Shell_TrayWnd", "Shell_SecondaryTrayWnd", "NotifyIconOverflowWindow"] {
                assert!(shell(class), "{class}");
            }
            assert!(!shell("Chrome_WidgetWin_1"));
            assert!(!shell("progman"));
        }

        #[test]
        fn a_capture_shrinks_to_1600_on_its_longest_edge_and_never_grows() {
            assert_eq!(fit(3840, 2160), (1600, 900));
            assert_eq!(fit(1000, 3000), (533, 1600));
            assert_eq!(fit(800, 600), (800, 600));
            assert_eq!(fit(1600, 1), (1600, 1));
            assert_eq!(fit(20000, 2), (1600, 1));
        }

        #[test]
        fn a_black_or_empty_capture_is_no_picture() {
            assert!(picture(2, 1, &[0, 0, 0, 255, 0, 0, 0, 0]).is_none());
            assert!(picture(0, 0, &[]).is_none());
        }

        #[test]
        fn a_capture_becomes_a_png_without_alpha_in_base64() {
            let shot = picture(2, 1, &[255, 0, 0, 7, 0, 0, 255, 9]).expect("picture");

            let bytes = STANDARD.decode(&shot.data).expect("base64");
            let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().expect("png");
            let mut pixels = vec![0; reader.output_buffer_size().expect("size")];
            let info = reader.next_frame(&mut pixels).expect("frame");
            assert_eq!((shot.media_type, shot.width, shot.height), ("image/png", 2, 1));
            assert_eq!((info.width, info.height, info.color_type), (2, 1, png::ColorType::Rgb));
            assert_eq!(&pixels[..info.buffer_size()], &[0, 0, 255, 255, 0, 0]);
        }

        #[test]
        fn clipboard_text_stops_at_its_end_and_at_twenty_thousand_characters() {
            let mut ended = wide("hola");
            ended.extend([0, 'x' as u16]);
            assert_eq!(decoded(&ended).as_deref(), Some("hola"));
            assert_eq!(decoded(&wide("sin final")).as_deref(), Some("sin final"));
            assert_eq!(decoded(&wide(&"é🙂".repeat(15_000))).map(|text| text.chars().count()), Some(MOST_CHARS));
            assert_eq!(decoded(&wide(" \n\t")), None);
            assert_eq!(decoded(&[]), None);
        }
    }
}

#[cfg(not(windows))]
mod native {
    use super::{Seen, Shot};

    pub fn foreground() -> Option<Seen> {
        None
    }

    pub fn shot(_window: isize) -> Option<Shot> {
        None
    }

    pub fn clipboard() -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_copy_is_recent_for_thirty_seconds() {
        let copied = Instant::now();
        let later = |seconds| copied + Duration::from_secs(seconds);

        assert!(fresh(Some(copied), later(0)));
        assert!(fresh(Some(copied), later(29)));
        assert!(!fresh(Some(copied), later(30)));
        assert!(!fresh(Some(copied), later(120)));
        assert!(!fresh(None, later(0)));
        assert!(fresh(Some(later(1)), copied));
    }

    #[test]
    fn the_clip_chip_shows_one_line_and_counts_every_character() {
        let clip = summary("  primera línea\n\tsegunda   línea  ");
        assert_eq!(
            clip,
            Clip {
                preview: "primera línea segunda línea".into(),
                chars: 34,
            }
        );
        assert_eq!(summary(&"ñ".repeat(200)).preview.chars().count(), PREVIEW_CHARS);
        assert_eq!(summary(&"ñ".repeat(200)).chars, 200);
    }
}
