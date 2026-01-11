pub fn rgb_to_hvs_into(rgb_buf: &[u8], hsv_buf: &mut [u8]) {
    for (i, pixel) in rgb_buf.chunks_exact(3).enumerate() {
        let (h, s, v) = rgb_to_hsv(pixel[0], pixel[1], pixel[2]);
        let base = i * 3;
        hsv_buf[base] = h;
        hsv_buf[base + 1] = s;
        hsv_buf[base + 2] = v;
    }
}

// Converts an RGB triple to HSV components scaled to bytes.
fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };
    let h_byte = (h * 255.0 / 360.0).round() as u8;

    let s = if max == 0.0 { 0.0 } else { delta / max };
    let s_byte = (s * 255.0).round() as u8;
    let v_byte = (max * 255.0).round() as u8;

    (h_byte, s_byte, v_byte)
}
