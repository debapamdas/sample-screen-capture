use windows::{Win32::Graphics::Direct3D11::{ID3D11Texture2D, D3D11_TEXTURE2D_DESC}, Graphics::Imaging::{BitmapEncoder, BitmapPixelFormat, BitmapAlphaMode}, Storage::{Streams::FileRandomAccessStream, FileAccessMode}};

pub fn get_encoder() -> windows::core::Result<BitmapEncoder> {
    let path = windows::core::HSTRING::from(r"C:\Users\dedas\Pictures\screenshot.jpg");
    let random_access_stream =
        FileRandomAccessStream::OpenAsync(&path, FileAccessMode::ReadWrite)?.get()?;
    BitmapEncoder::CreateAsync(BitmapEncoder::JpegEncoderId()?, &random_access_stream)?.get()
}

pub fn encode_image(
    texture: ID3D11Texture2D,
    bitmap_encoder: BitmapEncoder,
) -> windows::core::Result<()> {
    let mut desc = D3D11_TEXTURE2D_DESC::default();
    unsafe {
        texture.GetDesc(&mut desc);
    }
    let pixels = crate::d3d::get_bytes_from_texture(texture);
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