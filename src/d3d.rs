use windows::{
    core::{Interface, IInspectable},
    Graphics::DirectX::Direct3D11::IDirect3DDevice,
    Win32::{
        Graphics::{
            Direct3D::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE},
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                D3D11_SDK_VERSION, ID3D11Texture2D, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_RESOURCE_MISC_FLAG, D3D11_CPU_ACCESS_FLAG, D3D11_BIND_FLAG, D3D11_USAGE_DEFAULT, D3D11_BIND_SHADER_RESOURCE,
            },
            Dxgi::IDXGIDevice,
        },
        System::WinRT::Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess},
    },
};

pub fn get_device() -> windows::core::Result<IDirect3DDevice> {
    let device = create_device_with_type(D3D_DRIVER_TYPE_HARDWARE)?.cast::<IDXGIDevice>()?;
    unsafe { CreateDirect3D11DeviceFromDXGIDevice(&device)?.cast::<IDirect3DDevice>() }
}

pub fn get_dxgi_interface_from_object<T>(object: IInspectable) -> windows::core::Result<T>
where
    T: Interface,
{
    let dxgi_interface = object.cast::<IDirect3DDxgiInterfaceAccess>()?;
    unsafe { dxgi_interface.GetInterface() }
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

pub fn get_bytes_from_texture(texture: ID3D11Texture2D) -> Vec<u8> {
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
