use std::sync::mpsc::sync_channel;

use windows::{
    core::IInspectable,
    Foundation::TypedEventHandler,
    Graphics::{
        Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem},
        DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
    },
    Win32::Graphics::Direct3D11::ID3D11Texture2D,
};

pub fn take_snapshot(
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
    let handler = TypedEventHandler::new({
        let session = session.clone();
        move |frame_pool: &Option<Direct3D11CaptureFramePool>, _| {
            let frame_pool = frame_pool.as_ref().unwrap();
            let frame = frame_pool.TryGetNextFrame()?;
            sender.send(frame).unwrap();

            // End the capture
            session.Close()?;
            frame_pool.Close()?;
            Ok(())
        }
    });

    // Start the capture
    pool.FrameArrived(&handler)?;
    session.StartCapture()?;

    // Wait for our texture to come
    let frame = reciver.recv().unwrap();
    crate::d3d::get_dxgi_interface_from_object::<ID3D11Texture2D>(IInspectable::from(
        frame.Surface()?,
    ))
}
