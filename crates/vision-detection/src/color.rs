pub struct ColorRange {
    pub lower: [u8; 3],
    pub upper: [u8; 3],
}

impl ColorRange {
    pub fn in_range(&self, h: u8, s: u8, v: u8) -> bool {
        h >= self.lower[0]
            && h <= self.upper[0]
            && s >= self.lower[1]
            && s <= self.upper[1]
            && v >= self.lower[2]
            && v <= self.upper[2]
    }
}

// Converts an RGB triple to HSV components scaled to bytes.
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
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
