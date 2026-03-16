/// Hue-shift an RGBA icon from its base blue to a target accent color.

/// Convert RGB [0..255] to HSL. Returns (h: 0..360, s: 0..1, l: 0..1).
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let r = r as f64 / 255.0;
    let g = g as f64 / 255.0;
    let b = b as f64 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;

    if (max - min).abs() < 1e-10 {
        return (0.0, 0.0, l);
    }

    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };

    let h = if (max - r).abs() < 1e-10 {
        let mut h = (g - b) / d;
        if g < b {
            h += 6.0;
        }
        h
    } else if (max - g).abs() < 1e-10 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };

    (h * 60.0, s, l)
}

/// Convert HSL back to RGB.
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s.abs() < 1e-10 {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let h = h / 360.0;

    let hue_to_rgb = |t: f64| -> f64 {
        let mut t = t;
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 1.0 / 2.0 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };

    let r = (hue_to_rgb(h + 1.0 / 3.0) * 255.0).round() as u8;
    let g = (hue_to_rgb(h) * 255.0).round() as u8;
    let b = (hue_to_rgb(h - 1.0 / 3.0) * 255.0).round() as u8;
    (r, g, b)
}

/// The base icon's dominant hue (blue, ~220°).
const BASE_HUE: f64 = 220.0;

/// Recolor RGBA pixel data by hue-shifting from the base blue to the target accent color.
/// Pixels that are near-grey (saturation < 0.05) or fully transparent are left unchanged.
pub fn recolor_icon_pixels(rgba: &mut [u8], accent_r: u8, accent_g: u8, accent_b: u8) {
    let (target_hue, target_sat, _) = rgb_to_hsl(accent_r, accent_g, accent_b);
    let hue_shift = target_hue - BASE_HUE;
    // Ratio to scale saturation (so a red accent doesn't stay blue-saturated)
    let sat_ratio = if target_sat > 0.01 { target_sat / 0.7 } else { 1.0 }; // base sat ~0.7

    for pixel in rgba.chunks_exact_mut(4) {
        let a = pixel[3];
        if a == 0 {
            continue;
        }

        let (h, s, l) = rgb_to_hsl(pixel[0], pixel[1], pixel[2]);

        // Skip near-grey pixels (highlights, shadows) — they should stay neutral
        if s < 0.05 {
            continue;
        }

        let mut new_h = h + hue_shift;
        if new_h < 0.0 { new_h += 360.0; }
        if new_h >= 360.0 { new_h -= 360.0; }

        let new_s = (s * sat_ratio).clamp(0.0, 1.0);
        let (r, g, b) = hsl_to_rgb(new_h, new_s, l);
        pixel[0] = r;
        pixel[1] = g;
        pixel[2] = b;
    }
}

/// Build a recolored Tauri Image from the app's default icon.
pub fn recolored_tray_icon(
    original: &tauri::image::Image<'_>,
    accent: (u8, u8, u8),
) -> tauri::image::Image<'static> {
    let mut rgba = original.rgba().to_vec();
    recolor_icon_pixels(&mut rgba, accent.0, accent.1, accent.2);
    tauri::image::Image::new_owned(rgba, original.width(), original.height())
}
