mod camera;
mod config;
mod frame;

use camera::Camera;
use config::Config;
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("RustyVision waking up...");

    let config = Config::load_default().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Using default configuration");
        Config::default()
    });

    let mut camera = Camera::new(
        config.camera.device_id,
        config.camera.width,
        config.camera.height,
        config.camera.fps,
    )?;
    camera.open()?;

    let mut window = Window::new(
        "RustyVision",
        config.camera.width as usize,
        config.camera.height as usize,
        WindowOptions::default(),
    )?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame = camera.capture_frame()?;
        let buf = frame.frame_to_u32();
        window
            .update_with_buffer(&buf, frame.width as usize, frame.height as usize)
            .expect("could not update window");
    }

    Ok(())

    // let mut last_log = Instant::now();
    // let mut frames = 0u64;
    // loop {
    //     let frame = camera.capture_frame()?;
    //     frames += 1;
    //     if last_log.elapsed() > Duration::from_secs(1) {
    //         tracing::info!(fps = frames);
    //         frames = 0;
    //         last_log = Instant::now();
    //     }

    //     image::save_buffer(
    //         "frame.png",
    //         &frame.data,
    //         frame.width,
    //         frame.height,
    //         image::ColorType::Rgb8,
    //     )?;
    // }
}
