use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

pub fn get_camera(device_id: u32) -> Result<Camera, Box<dyn std::error::Error>> {
    let index = CameraIndex::Index(device_id);
    let format_type = RequestedFormatType::AbsoluteHighestFrameRate;
    let requested = RequestedFormat::new::<RgbFormat>(format_type);

    let camera = Camera::new(index, requested)?;
    Ok(camera)
}

pub fn capture_frame_into(
    camera: &mut Camera,
    rgb_buf: &mut Vec<u8>,
) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let frame = camera.frame()?;
    let decoded = frame.decode_image::<RgbFormat>()?;

    let width = decoded.width();
    let height = decoded.height();
    let expected = (width * height * 3) as usize;

    rgb_buf.resize(expected, 0);
    rgb_buf.copy_from_slice(decoded.as_raw());
    Ok((width, height))
}
