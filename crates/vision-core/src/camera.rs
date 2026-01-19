use ndarray::{Array2, ArrayView2};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

pub fn get_camera(device_id: u32) -> anyhow::Result<Camera> {
    let index = CameraIndex::Index(device_id);
    let format_type = RequestedFormatType::AbsoluteHighestFrameRate;
    let requested = RequestedFormat::new::<RgbFormat>(format_type);

    let camera = Camera::new(index, requested)?;
    Ok(camera)
}

pub fn capture_frame(camera: &mut Camera, rgb_frame: &mut Array2<[u8; 3]>) -> anyhow::Result<()> {
    let frame = camera.frame()?;
    let decoded = frame.decode_image::<RgbFormat>()?;

    let width = decoded.width() as usize;
    let height = decoded.height() as usize;

    if rgb_frame.shape() != [height, width] {
        *rgb_frame = Array2::from_elem((height, width), [0u8; 3]);
    }

    let raw_data = decoded.as_raw();
    for (i, chunk) in raw_data.chunks_exact(3).enumerate() {
        let row = i / width;
        let col = i % width;
        rgb_frame[(row, col)] = [chunk[0], chunk[1], chunk[2]];
    }

    Ok(())
}

pub fn resize_array<T: Copy>(
    arr: ArrayView2<T>,
    resized: &mut Array2<T>,
    new_height: usize,
    new_width: usize,
) {
    let (old_height, old_width) = arr.dim();

    let y_ratio = old_height as f32 / new_height as f32;
    let x_ratio = old_width as f32 / new_width as f32;

    for y in 0..new_height {
        for x in 0..new_width {
            let src_y = (y as f32 * y_ratio) as usize;
            let src_x = (x as f32 * x_ratio) as usize;

            let src_y = src_y.min(old_height - 1);
            let src_x = src_x.min(old_width - 1);
            resized[(y, x)] = arr[(src_y, src_x)]
        }
    }
}
