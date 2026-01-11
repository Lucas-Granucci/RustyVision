use crate::{config::DetectionConfig, frame::rgb_to_hvs_into};
use vision_detection::color::{mask_hsv, ColorRange};

pub fn run_color_mask_into(
    rgb_buf: &[u8],
    cfg: &DetectionConfig,
    hsv_buf: &mut [u8],
    mask_buf: &mut Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    rgb_to_hvs_into(rgb_buf, hsv_buf);

    let range = ColorRange {
        lower: cfg.color_lower,
        upper: cfg.color_upper,
    };

    mask_buf.clear();
    mask_hsv(&hsv_buf, range, mask_buf);
    Ok(())
}
