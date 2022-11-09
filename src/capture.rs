use windows::{
    Graphics::Capture::GraphicsCaptureItem,
    Win32::{
        Foundation::HWND, Graphics::Gdi::HMONITOR,
        System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop,
    },
};

pub fn create_capture_item_for_monitor(
    hmon: HMONITOR,
) -> windows::core::Result<GraphicsCaptureItem> {
    let activation_factory =
        windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    unsafe { activation_factory.CreateForMonitor(hmon) }
}

pub fn create_capture_item_for_window(hwnd: HWND) -> windows::core::Result<GraphicsCaptureItem> {
    let activation_factory =
        windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    unsafe { activation_factory.CreateForWindow(hwnd) }
}
