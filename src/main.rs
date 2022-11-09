use std::time::Instant;

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

    // Initialize the encoder
    let bitmap_encoder = encoder::get_encoder()?;

    // Take the snapshot
    let texture = snapshot::take_snapshot(device, item, DirectXPixelFormat::B8G8R8A8UIntNormalized)
        .expect("cannot take snapshot");

    // Encode the image
    encoder::encode_image(texture, bitmap_encoder)?;

    println!("{}", now.elapsed().as_millis());

    Ok(())
}


