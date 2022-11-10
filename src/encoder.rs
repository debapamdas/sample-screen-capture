use std::path::Path;

use windows::{
    Graphics::Imaging::{BitmapAlphaMode, BitmapEncoder, BitmapPixelFormat},
    Storage::{FileAccessMode, Streams::FileRandomAccessStream},
};

pub fn get_encoder(path: &Path) -> windows::core::Result<BitmapEncoder> {
    let path = windows::core::HSTRING::from(path.to_str().unwrap());
    let random_access_stream =
        FileRandomAccessStream::OpenAsync(&path, FileAccessMode::ReadWrite)?.get()?;
    BitmapEncoder::CreateAsync(BitmapEncoder::JpegEncoderId()?, &random_access_stream)?.get()
}

pub fn encode_image(
    height: u32,
    width: u32,
    pixels: Vec<u8>,
    bitmap_encoder: BitmapEncoder,
) -> windows::core::Result<()> {
    bitmap_encoder
        .SetPixelData(
            BitmapPixelFormat::Bgra8,
            BitmapAlphaMode::Premultiplied,
            width,
            height,
            1.0,
            1.0,
            &pixels,
        )
        .expect("cannot encode");
    bitmap_encoder.FlushAsync().expect("cannot flush").get()
}
