use std::{time::Instant, path::Path};

use windows::Graphics::DirectX::DirectXPixelFormat;

mod d3d;
mod display;
mod window;
mod encoder;
mod snapshot;
mod capture;

fn main() -> windows::core::Result<()> {
    let device = d3d::get_device()?;

    let monitors = display::get_current_monitors();

    let capture_item = capture::create_capture_item_for_monitor(*monitors.get(0).unwrap())?;

    let now = Instant::now();

    // Take the snapshot
    let (height, width, pixels) = snapshot::take_snapshot(device, capture_item, DirectXPixelFormat::B8G8R8A8UIntNormalized)
        .expect("cannot take snapshot");
    // get encoder
    let encoder = encoder::get_encoder(Path::new(r"C:\Users\dedas\Pictures\screenshot.jpg"))?;
    // Encode the image
    encoder::encode_image(height, width, pixels, encoder)?;

    println!("screen captured in {} ms", now.elapsed().as_millis());

    Ok(())
}


