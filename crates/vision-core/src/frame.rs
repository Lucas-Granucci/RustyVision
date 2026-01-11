use thiserror::Error;

#[derive(Clone, Debug)]
// Represents an image frame with raw pixel data and dimensions.
pub struct Frame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}

#[derive(Clone, Copy, Debug, PartialEq)]
// Describes how pixels are laid out and how many bytes each uses.
pub enum PixelFormat {
    RGB8,  // 3 bytes per pixel (R, G, B)
    RGBA8, // 4 bytes per pixel (R, G, B, A)
    BGR8,  // 3 bytes per pixel (B, G, R)
    GRAY8, // 1 byte per pixel (grayscale)
    HSV,   // 3 bytes per pixel (H, S, V)
}

impl PixelFormat {
    // Returns how many bytes each pixel uses for this format.
    pub const fn bytes_per_pixel(&self) -> u32 {
        match self {
            PixelFormat::GRAY8 => 1,
            PixelFormat::RGB8 | PixelFormat::BGR8 | PixelFormat::HSV => 3,
            PixelFormat::RGBA8 => 4,
        }
    }
}

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("Buffer size doesn't match")]
    InvalidDimensions { expected: usize, actual: usize },

    #[error("Provided dimensions are zero")]
    ZeroDimensions,
}

pub struct FrameConfig {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
}

impl Frame {
    // Validates buffer size against dimensions and constructs a frame.
    pub fn new(config: FrameConfig) -> Result<Self, FrameError> {
        // Validate of parameters
        if config.width == 0 || config.height == 0 {
            return Err(FrameError::ZeroDimensions);
        }

        let expected = config.width as usize
            * config.height as usize
            * config.format.bytes_per_pixel() as usize;
        if config.data.len() != expected {
            return Err(FrameError::InvalidDimensions {
                expected,
                actual: config.data.len(),
            });
        }

        Ok(Self {
            data: config.data,
            width: config.width,
            height: config.height,
            format: config.format,
        })
    }

    // Returns the pixel bytes at (x, y) if inside bounds.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let bytes_per_pixel = self.format.bytes_per_pixel() as usize;
        let index = ((y * self.width + x) as usize) * bytes_per_pixel;
        self.data.get(index..index + bytes_per_pixel)
    }

    // Converts to 32-bit int buffer
    pub fn frame_to_u32(&self) -> Vec<u32> {
        match self.format {
            PixelFormat::RGB8 => {
                return self
                    .data
                    .chunks_exact(3)
                    .map(|p| ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32))
                    .collect()
            }
            _ => {
                let rgb_frame = self.to_rgb8();
                return rgb_frame
                    .data
                    .chunks_exact(3)
                    .map(|p| ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32))
                    .collect();
            }
        }
    }

    // Converts the frame into an 8-bit RGB frame.
    pub fn to_rgb8(&self) -> Frame {
        let capacity = (self.height * self.width * 3) as usize;
        let mut new_data = Vec::with_capacity(capacity);

        if self.format == PixelFormat::RGB8 {
            return self.clone();
        }

        match self.format {
            PixelFormat::HSV => {
                for pixel in self.data.chunks_exact(3) {
                    let (r, g, b) = Frame::hsv_to_rgb(pixel[0], pixel[1], pixel[2]);
                    new_data.extend([r, g, b]);
                }
            }
            PixelFormat::GRAY8 => {
                for pixel in self.data.chunks_exact(1) {
                    new_data.extend([pixel[0], pixel[0], pixel[0]]);
                }
            }
            _ => {
                let format_size = self.format.bytes_per_pixel() as usize;
                for pixel in self.data.chunks_exact(format_size) {
                    let (r, g, b) = self.extract_rgb(pixel);
                    new_data.extend([r, g, b]);
                }
            }
        }

        Frame {
            data: new_data,
            width: self.width,
            height: self.height,
            format: PixelFormat::RGB8,
        }
    }

    // Converts the frame into an 8-bit grayscale frame.
    pub fn to_grayscale(&self) -> Frame {
        let capacity = (self.height * self.width) as usize;
        let mut new_data = Vec::with_capacity(capacity);

        if self.format == PixelFormat::GRAY8 {
            return self.clone();
        }

        match self.format {
            PixelFormat::HSV => {
                for pixel in self.data.chunks_exact(3) {
                    new_data.push(pixel[2]) // V channel
                }
            }
            _ => {
                let format_size = self.format.bytes_per_pixel() as usize;
                for pixel in self.data.chunks_exact(format_size) {
                    let (r, g, b) = self.extract_rgb(pixel);
                    new_data.push(Frame::rgb_to_gray(r, g, b));
                }
            }
        }

        Frame {
            data: new_data,
            width: self.width,
            height: self.height,
            format: PixelFormat::GRAY8,
        }
    }

    // Converts the frame into HSV pixel format.
    pub fn to_hsv(&self) -> Frame {
        let capacity = (self.height * self.width * 3) as usize;
        let mut new_data = Vec::with_capacity(capacity);

        if self.format == PixelFormat::HSV {
            return self.clone();
        }

        match self.format {
            PixelFormat::GRAY8 => {
                for gray in &self.data {
                    new_data.extend([0, 0, *gray]);
                }
            }
            _ => {
                let format_size = self.format.bytes_per_pixel() as usize;
                for pixel in self.data.chunks_exact(format_size) {
                    let (r, g, b) = self.extract_rgb(pixel);
                    let (h, s, v) = Frame::rgb_to_hsv(r, g, b);
                    new_data.extend([h, s, v]);
                }
            }
        }

        Frame {
            data: new_data,
            width: self.width,
            height: self.height,
            format: PixelFormat::HSV,
        }
    }

    // Normalizes a pixel into (r, g, b) ordering regardless of source format.
    fn extract_rgb(&self, pixel: &[u8]) -> (u8, u8, u8) {
        match self.format {
            PixelFormat::RGB8 | PixelFormat::RGBA8 => (pixel[0], pixel[1], pixel[2]),
            PixelFormat::BGR8 => (pixel[2], pixel[1], pixel[0]),
            PixelFormat::GRAY8 => (pixel[0], pixel[0], pixel[0]),
            PixelFormat::HSV => panic!("Can't extract RGB from HSV"),
        }
    }

    // Converts an RGB triple to a single luminance value.
    fn rgb_to_gray(r: u8, g: u8, b: u8) -> u8 {
        (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8
    }

    // Converts HSV to RGB
    fn hsv_to_rgb(h_byte: u8, s_byte: u8, v_byte: u8) -> (u8, u8, u8) {
        let h = (h_byte as f32) * 360.0 / 255.0; // Scale to 0-360 degrees
        let s = s_byte as f32 / 255.0; // Scale to 0-1
        let v = v_byte as f32 / 255.0; // Scale to 0-1

        let c = v * s; // Chroma
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r_prime, g_prime, b_prime) = match (h as i32) / 60 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            5 => (c, 0.0, x),
            _ => (0.0, 0.0, 0.0),
        };

        // Add m to each component and scale back to 0-255
        let r = ((r_prime + m) * 255.0).round() as u8;
        let g = ((g_prime + m) * 255.0).round() as u8;
        let b = ((b_prime + m) * 255.0).round() as u8;

        (r, g, b)
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
}
