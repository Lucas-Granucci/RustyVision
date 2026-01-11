use crate::frame::{Frame, FrameConfig, FrameError, PixelFormat};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera as NokhwaCamera;

pub struct Camera {
    inner: NokhwaCamera,
    width: u32,
    height: u32,
    fps: u32,
}

pub enum CameraError {
    OpenFailed(String),
    CaptureFailed(String),
    FrameError(FrameError),
    NoFrameAvailable,
}

impl From<FrameError> for CameraError {
    fn from(err: FrameError) -> Self {
        CameraError::FrameError(err)
    }
}

pub fn open_camera(
    device_id: u32,
    width: u32,
    height: u32,
    fps: u32,
) -> Result<Camera, CameraError> {
    let index = CameraIndex::Index(device_id);
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

    let mut camera =
        NokhwaCamera::new(index, requested).map_err(|e| CameraError::OpenFailed(e.to_string()))?;
    camera
        .open_stream()
        .map_err(|e| CameraError::OpenFailed(e.to_string()))?;

    Ok(Camera {
        inner: camera,
        width: width,
        height: height,
        fps: fps,
    })
}

pub fn capture_frame(camera: &mut Camera) -> Result<Frame, CameraError> {
    let frame = camera
        .inner
        .frame()
        .map_err(|e| CameraError::CaptureFailed(e.to_string()))?;

    let decoded = frame
        .decode_image::<RgbFormat>()
        .map_err(|e| CameraError::CaptureFailed(e.to_string()))?;

    let data = decoded.to_vec();
    let width = decoded.width();
    let height = decoded.height();

    let frame = Frame::new(FrameConfig {
        data,
        width,
        height,
        format: PixelFormat::RGB8,
    })?;
    Ok(frame)
}
