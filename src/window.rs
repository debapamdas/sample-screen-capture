use windows::Win32::{Foundation::{BOOL, HWND, LPARAM}, UI::WindowsAndMessaging::{GetWindowTextLengthW, EnumWindows}};

#[allow(dead_code)]
pub fn get_current_windows() -> Vec<HWND> {
    let mut windows = Vec::<HWND>::new();
    let windows_ptr = &mut windows as *mut Vec<HWND> as isize;
    unsafe extern "system" fn enumerate_windows_callback(window: HWND, userdata: LPARAM) -> BOOL {
        let windows: &mut Vec<HWND> = std::mem::transmute(userdata);
        if GetWindowTextLengthW(window) > 0 {
            if !is_capturable_window(window) {
                BOOL(1)
            } else {
                windows.push(window);
                BOOL(1)
            }
        } else {
            BOOL(1)
        }
    }
    unsafe {
        let _ = EnumWindows(Some(enumerate_windows_callback), LPARAM(windows_ptr));
        windows
    }
}

fn is_capturable_window(_hwnd: HWND) -> bool {
    todo!()
}
