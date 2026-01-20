use image::{DynamicImage, GrayImage, ImageBuffer, Rgb, RgbImage};
use ndarray::ArrayView2;

pub trait ToDynamicImage {
    fn to_dynamic_image(&self) -> Option<DynamicImage>;
}

impl<'a> ToDynamicImage for ArrayView2<'a, u8> {
    fn to_dynamic_image(&self) -> Option<DynamicImage> {
        let (height, width) = self.dim();
        let img: GrayImage = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
            image::Luma([self[[y as usize, x as usize]]])
        });
        Some(DynamicImage::ImageLuma8(img))
    }
}

impl<'a> ToDynamicImage for ArrayView2<'a, [u8; 3]> {
    fn to_dynamic_image(&self) -> Option<DynamicImage> {
        let (height, width) = self.dim();
        let img: RgbImage = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
            // Convert [u8; 3] to Rgb pixel
            Rgb(self[[y as usize, x as usize]])
        });
        Some(DynamicImage::ImageRgb8(img))
    }
}

// Convert grayscale ndarray to JPEG bytes
pub fn array_to_jpeg(arr: impl ToDynamicImage) -> Option<Vec<u8>> {
    let img = arr.to_dynamic_image()?;
    let mut buf = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 60)
        .encode_image(&img)
        .ok()?;
    Some(buf)
}
