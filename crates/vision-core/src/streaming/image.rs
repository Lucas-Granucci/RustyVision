use image::{GrayImage, ImageBuffer};
use ndarray::ArrayView2;

// Convert grayscale ndarray to JPEG bytes
pub fn array_to_jpeg(arr: ArrayView2<u8>) -> Option<Vec<u8>> {
    let (height, width) = arr.dim();
    let img: GrayImage = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
        image::Luma([arr[[y as usize, x as usize]]])
    });
    let mut buf = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 60)
        .encode_image(&img)
        .ok()?;
    Some(buf)
}
