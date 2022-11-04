use std::mem;
use std::sync::mpsc::sync_channel;
use std::time::Instant;

use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM};
use windows::core::{IInspectable, Interface};
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapEncoder, BitmapPixelFormat};
use windows::Storage::FileAccessMode;
use windows::Storage::Streams::FileRandomAccessStream;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Direct3D::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11Texture2D, D3D11_BIND_FLAG, D3D11_BIND_SHADER_RESOURCE,
    D3D11_CPU_ACCESS_FLAG, D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAP_READ,
    D3D11_RESOURCE_MISC_FLAG, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
    D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextLengthW};

fn main() -> windows::core::Result<()> {
    let device = get_device()?;

    let monitors = get_current_monitors();
    // let windows = get_current_windows();

    let monitor_item = create_capture_item_for_monitor(*monitors.get(0).unwrap())?;
    // let _window_item = create_capture_item_for_window(*windows.get(0).unwrap())?;

    let item = monitor_item;

    let now = Instant::now();

    // Initialize the encoder
    let bitmap_encoder = get_encoder()?;

    // Take the snapshot
    let texture = take_snapshot(device, item, DirectXPixelFormat::B8G8R8A8UIntNormalized)
        .expect("cannot take snapshot");

    // Encode the image
    encode_image(texture, bitmap_encoder)?;

    println!("{}", now.elapsed().as_millis());

    Ok(())
}

fn encode_image(
    texture: ID3D11Texture2D,
    bitmap_encoder: BitmapEncoder,
) -> windows::core::Result<()> {
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    unsafe {
        texture.GetDesc(&mut desc);
    }
    let pixels = get_bytes_from_texture(texture);
    bitmap_encoder.SetPixelData(
        BitmapPixelFormat::Bgra8,
        BitmapAlphaMode::Premultiplied,
        desc.Width,
        desc.Height,
        1.0,
        1.0,
        &pixels,
    ).expect("cannot encode");
    bitmap_encoder.FlushAsync().expect("cannot flush").get()
}

fn take_snapshot(
    device: IDirect3DDevice,
    item: GraphicsCaptureItem,
    format: DirectXPixelFormat,
) -> windows::core::Result<ID3D11Texture2D> {
    // let d3d_device = get_dxgi_interface_from_object();
    // let mut context = None;
    // unsafe {
    //     d3d_device.GetImmediateContext(&mut context);
    // }
    let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(&device, format, 2, item.Size()?)?;
    let session = pool.CreateCaptureSession(&item)?;
    let (sender, reciver) = sync_channel(1);
    let handler =
        TypedEventHandler::new(move |frame_pool: &Option<Direct3D11CaptureFramePool>, _| {
            let frame_pool = frame_pool.as_ref().unwrap();
            let frame = frame_pool.TryGetNextFrame()?;
            sender.send(frame).unwrap();
            Ok(())
        });
    pool.FrameArrived(&handler)?;
    session.StartCapture()?;
    let frame = reciver.recv().unwrap();
    session.Close()?;
    pool.Close()?;
    get_dxgi_interface_from_object::<ID3D11Texture2D>(IInspectable::from(frame.Surface()?))
}

fn get_encoder() -> windows::core::Result<BitmapEncoder> {
    let path = windows::core::HSTRING::from(r"C:\Users\dedas\Pictures\screenshot.jpg");
    let random_access_stream =
        FileRandomAccessStream::OpenAsync(&path, FileAccessMode::ReadWrite)?.get()?;
    BitmapEncoder::CreateAsync(BitmapEncoder::JpegEncoderId()?, &random_access_stream)?.get()
}

fn get_bytes_from_texture(texture: ID3D11Texture2D) -> Vec<u8> {
    let mut ppdevice = None;
    unsafe {
        texture.GetDevice(&mut ppdevice);
    }
    let ppdevice = ppdevice.unwrap();

    let mut ppimmediatecontext = None;
    unsafe {
        ppdevice.GetImmediateContext(&mut ppimmediatecontext);
    }
    let ppimmediatecontext = ppimmediatecontext.unwrap();

    let staging_texture = prepare_staging_texture(texture).expect("cannot prepare staging texture");

    let mut desc = D3D11_TEXTURE2D_DESC::default();
    unsafe {
        staging_texture.GetDesc(&mut desc);
    }

    // let bytes_per_pixel = match desc.Format {
    //     DXGI_FORMAT_B8G8R8A8_UNORM => 4,
    //     _ => panic!("Unsupported format! {:?}", desc.Format),
    // };
    let mapped_resource = unsafe {
        ppimmediatecontext
            .Map(&staging_texture, 0, D3D11_MAP_READ, 0)
            .expect("cannot get mapped resource")
    };
    let bytes = unsafe {
        std::slice::from_raw_parts(mapped_resource.pData as *const u8, (desc.Height * mapped_resource.RowPitch) as usize)
    };
    //  // Make a copy of the data
    //  let mut data = vec![0u8; ((desc.Width * desc.Height) * bytes_per_pixel) as usize];
    //  for row in 0..desc.Height {
    //      let data_begin = (row * (desc.Width * bytes_per_pixel)) as usize;
    //      let data_end = ((row + 1) * (desc.Width * bytes_per_pixel)) as usize;
    //      let slice_begin = (row * mapped_resource.RowPitch) as usize;
    //      let slice_end = slice_begin + (desc.Width * bytes_per_pixel) as usize;
    //      data[data_begin..data_end].copy_from_slice(&bytes[slice_begin..slice_end]);
    //  }
    
    let ret = bytes.to_vec();

    unsafe { ppimmediatecontext.Unmap(&staging_texture, 0) }

    ret
}

fn prepare_staging_texture(texture: ID3D11Texture2D) -> windows::core::Result<ID3D11Texture2D> {
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    unsafe {
        texture.GetDesc(&mut desc);
    }
    if desc.Usage == D3D11_USAGE_STAGING && (desc.CPUAccessFlags & D3D11_CPU_ACCESS_READ).0 != 0 {
        windows::core::Result::Ok(texture)
    } else {
        copy_d3d_texture(texture, true)
    }
}

fn copy_d3d_texture(
    texture: ID3D11Texture2D,
    as_staging_texture: bool,
) -> windows::core::Result<ID3D11Texture2D> {
    let mut ppdevice = None;
    unsafe {
        texture.GetDevice(&mut ppdevice);
    }
    let ppdevice = ppdevice.unwrap();
    let mut ppimmediatecontext = None;
    unsafe {
        ppdevice.GetImmediateContext(&mut ppimmediatecontext);
    }
    let ppimmediatecontext = ppimmediatecontext.unwrap();
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    unsafe {
        texture.GetDesc(&mut desc);
    }
    // Clear flags that we don't need
    desc.Usage = if as_staging_texture {
        D3D11_USAGE_STAGING
    } else {
        D3D11_USAGE_DEFAULT
    };
    desc.BindFlags = if as_staging_texture {
        D3D11_BIND_FLAG(0)
    } else {
        D3D11_BIND_SHADER_RESOURCE
    };
    desc.CPUAccessFlags = if as_staging_texture {
        D3D11_CPU_ACCESS_READ
    } else {
        D3D11_CPU_ACCESS_FLAG(0)
    };
    desc.MiscFlags = D3D11_RESOURCE_MISC_FLAG(0);

    // Create and fill the texture copy
    unsafe {
        let texture_copy = ppdevice.CreateTexture2D(&desc, None)?;
        ppimmediatecontext.CopyResource(&texture_copy, &texture);
        windows::core::Result::Ok(texture_copy)
    }
}

fn get_dxgi_interface_from_object<T>(object: IInspectable) -> windows::core::Result<T>
where
    T: Interface,
{
    let dxgi_interface = object.cast::<IDirect3DDxgiInterfaceAccess>()?;
    unsafe { dxgi_interface.GetInterface() }
}

fn get_device() -> windows::core::Result<IDirect3DDevice> {
    let device = create_device_with_type(D3D_DRIVER_TYPE_HARDWARE)?.cast::<IDXGIDevice>()?;
    unsafe { CreateDirect3D11DeviceFromDXGIDevice(&device)?.cast::<IDirect3DDevice>() }
}

fn create_device_with_type(drive_type: D3D_DRIVER_TYPE) -> windows::core::Result<ID3D11Device> {
    let mut device = None;
    unsafe {
        D3D11CreateDevice(
            None,
            drive_type,
            None,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )?;
        Ok(device.unwrap())
    }
}

fn get_current_monitors() -> Vec<HMONITOR> {
    let mut monitors = Vec::<HMONITOR>::new();
    let monitors_ptr = &mut monitors as *mut Vec<HMONITOR> as isize;
    unsafe extern "system" fn enumerate_monitors_callback(
        monitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        userdata: LPARAM,
    ) -> BOOL {
        let monitors: &mut Vec<HMONITOR> = mem::transmute(userdata);
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

fn create_capture_item_for_monitor(hmon: HMONITOR) -> windows::core::Result<GraphicsCaptureItem> {
    let activation_factory =
        windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    unsafe { activation_factory.CreateForMonitor(hmon) }
}

fn _get_current_windows() -> Vec<HWND> {
    let mut windows = Vec::<HWND>::new();
    let windows_ptr = &mut windows as *mut Vec<HWND> as isize;
    unsafe extern "system" fn enumerate_windows_callback(window: HWND, userdata: LPARAM) -> BOOL {
        let windows: &mut Vec<HWND> = mem::transmute(userdata);
        if GetWindowTextLengthW(window) > 0 {
            if !_is_capturable_window(window) {
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

fn _create_capture_item_for_window(hwnd: HWND) -> windows::core::Result<GraphicsCaptureItem> {
    let activation_factory =
        windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()?;
    unsafe { activation_factory.CreateForWindow(hwnd) }
}

fn _is_capturable_window(_hwnd: HWND) -> bool {
    todo!()
}
