use crate::frame::{Frame, FrameConfig, FrameError, PixelFormat};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera as NokhwaCamera;
use thiserror::Error;

pub struct Camera {
    pub inner: NokhwaCamera,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

#[derive(Debug, Error)]
pub enum CameraError {
    #[error("Failed to open camera: {0}")]
    OpenFailed(String),

    #[error("Failed to capture frame: {0}")]
    CaptureFailed(String),

    #[error("Frame error: {0}")]
    FrameError(#[from] FrameError),

    #[error("No frame available from camera")]
    NoFrameAvailable,
}

impl Camera {
    pub fn new(device_id: u32, width: u32, height: u32, fps: u32) -> Result<Self, CameraError> {
        let camera = Camera::get_camera(device_id)?;
        Ok(Self {
            inner: camera,
            width: width,
            height: height,
            fps: fps,
        })
    }

    fn get_camera(device_id: u32) -> Result<NokhwaCamera, CameraError> {
        let index = CameraIndex::Index(device_id);
        let format_type = RequestedFormatType::AbsoluteHighestFrameRate;
        let requested = RequestedFormat::new::<RgbFormat>(format_type);

        let camera = NokhwaCamera::new(index, requested)
            .map_err(|e| CameraError::OpenFailed(e.to_string()))?;
        Ok(camera)
    }

    pub fn open(&mut self) -> Result<(), CameraError> {
        self.inner
            .open_stream()
            .map_err(|e| CameraError::OpenFailed(e.to_string()))
    }

    pub fn capture_frame(&mut self) -> Result<Frame, CameraError> {
        let frame = self
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
}
