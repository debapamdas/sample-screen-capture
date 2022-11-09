use windows::Win32::{
    Foundation::{BOOL, LPARAM, RECT},
    Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
};

pub fn get_current_monitors() -> Vec<HMONITOR> {
    let mut monitors = Vec::<HMONITOR>::new();
    let monitors_ptr = &mut monitors as *mut Vec<HMONITOR> as isize;
    unsafe extern "system" fn enumerate_monitors_callback(
        monitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        userdata: LPARAM,
    ) -> BOOL {
        let monitors: &mut Vec<HMONITOR> = std::mem::transmute(userdata);
        monitors.push(monitor);
        BOOL(1)
    }
    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(enumerate_monitors_callback),
            LPARAM(monitors_ptr),
        );
        monitors
    }
}
