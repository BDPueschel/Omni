use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

static ICON_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<String, String>> {
    ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[tauri::command]
pub fn get_icon(path: String) -> String {
    // Check cache first
    {
        let cache = get_cache().lock().unwrap();
        if let Some(cached) = cache.get(&path) {
            return cached.clone();
        }
    }

    let result = extract_icon(&path).unwrap_or_default();

    // Only cache successful results — failed extractions may succeed on retry
    if !result.is_empty() {
        let mut cache = get_cache().lock().unwrap();
        cache.insert(path, result.clone());
    }

    result
}

fn extract_icon(path: &str) -> Option<String> {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, SelectObject, BITMAP,
        BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, RGBQUAD,
    };
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_USEFILEATTRIBUTES};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    // Ensure COM is initialized on this thread (required for shell icon extraction)
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );
    }

    let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

    let mut shfi = SHFILEINFOW::default();
    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };

    if result == 0 || shfi.hIcon.is_invalid() {
        // Fallback: use file attributes (doesn't need file access, gives generic icon by type)
        shfi = SHFILEINFOW::default();
        let fallback = unsafe {
            SHGetFileInfoW(
                PCWSTR(wide_path.as_ptr()),
                windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
                Some(&mut shfi),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES,
            )
        };
        if fallback == 0 || shfi.hIcon.is_invalid() {
            return None;
        }
    }

    let icon = shfi.hIcon;

    // Get icon info to access the bitmap
    let mut icon_info = ICONINFO::default();
    let ok = unsafe { GetIconInfo(icon, &mut icon_info) };
    if ok.is_err() {
        unsafe {
            let _ = DestroyIcon(icon);
        }
        return None;
    }

    // Get bitmap dimensions
    let mut bmp = BITMAP::default();
    unsafe {
        GetObjectW(
            icon_info.hbmColor.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bmp as *mut _ as *mut _),
        );
    }

    let width = bmp.bmWidth as u32;
    let height = bmp.bmHeight as u32;

    if width == 0 || height == 0 {
        unsafe {
            let _ = DestroyIcon(icon);
            let _ = DeleteObject(icon_info.hbmColor.into());
            let _ = DeleteObject(icon_info.hbmMask.into());
        }
        return None;
    }

    // Extract pixel data
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32), // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0, // BI_RGB
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD::default()],
    };

    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let hdc = unsafe { CreateCompatibleDC(None) };
    let old = unsafe { SelectObject(hdc, icon_info.hbmColor.into()) };

    unsafe {
        GetDIBits(
            hdc,
            icon_info.hbmColor,
            0,
            height,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );
    }

    unsafe {
        SelectObject(hdc, old);
        let _ = DeleteDC(hdc);
        let _ = DeleteObject(icon_info.hbmColor.into());
        let _ = DeleteObject(icon_info.hbmMask.into());
        let _ = DestroyIcon(icon);
    }

    // Convert BGRA to RGBA
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2); // B <-> R
    }

    // Encode as PNG using the `image` crate
    let img = image::RgbaImage::from_raw(width, height, pixels)?;
    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    use image::ImageEncoder;
    encoder
        .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
        .ok()?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    Some(format!("data:image/png;base64,{}", b64))
}
