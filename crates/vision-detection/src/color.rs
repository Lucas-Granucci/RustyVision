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

pub fn mask_hsv(hsv_data: &[u8], range: ColorRange, out_buf: &mut Vec<u8>) {
    out_buf.clear();
    out_buf.extend(hsv_data.chunks_exact(3).map(|p| {
        if range.in_range(p[0], p[1], p[2]) {
            255
        } else {
            0
        }
    }));
}
