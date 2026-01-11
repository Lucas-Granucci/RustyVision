mod camera;
mod config;
mod detection;

use config::Config;
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};

use crate::{
    camera::{capture_frame_into, get_camera},
    detection::run_color_mask_into,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("RustyVision waking up...");

    // Load config
    let config = Config::load_default().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Using default configuration");
        Config::default()
    });

    // Camera stuff
    let mut camera = get_camera(config.camera.device_id)?;
    camera.open_stream()?;

    // Setup window
    let mut window = Window::new(
        "RustyVision",
        config.camera.width as usize,
        config.camera.height as usize,
        WindowOptions::default(),
    )?;
    window.set_target_fps(60);

    // Setup timing
    let mut frames: u64 = 0;
    let mut last_log = Instant::now();
    let mut accum_capture = Duration::ZERO;
    let mut accum_mask = Duration::ZERO;
    let mut accum_blit = Duration::ZERO;

    // Create buffers
    let pixel_count = (config.camera.width * config.camera.height) as usize;
    let mut window_buf: Vec<u32> = vec![0; pixel_count];

    let mut rgb_buf: Vec<u8> = vec![0; pixel_count * 3];
    let mut mask_buf: Vec<u8> = vec![0; pixel_count];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let t_capture = Instant::now();

        // Camera capture into RGB buf
        let (_, _) = capture_frame_into(&mut camera, &mut rgb_buf)?;
        let capture_dt = t_capture.elapsed();

        let t_mask = Instant::now();
        run_color_mask_into(&rgb_buf, &config.detection, &mut mask_buf);
        let mask_dt = t_mask.elapsed();

        let t_blit = Instant::now();
        for (dst, &gray) in window_buf.iter_mut().zip(mask_buf.iter()) {
            let g = gray as u32;
            *dst = (g << 16) | (g << 8) | g;
        }
        window.update_with_buffer(
            &window_buf,
            config.camera.width as usize,
            config.camera.height as usize,
        )?;
        let blit_dt = t_blit.elapsed();

        frames += 1;
        accum_capture += capture_dt;
        accum_mask += mask_dt;
        accum_blit += blit_dt;

        let elapsed = last_log.elapsed();
        if elapsed >= Duration::from_secs(1) && frames > 0 {
            let fps = frames as f64 / elapsed.as_secs_f64();
            let denom = frames as f64;
            let avg_capture_ms = accum_capture.as_secs_f64() * 1000.0 / denom;
            let avg_mask_ms = accum_mask.as_secs_f64() * 1000.0 / denom;
            let avg_blit_ms = accum_blit.as_secs_f64() * 1000.0 / denom;

            tracing::info!(
                fps = fps,
                frames_in_window = frames,
                avg_capture_ms = avg_capture_ms,
                avg_mask_ms = avg_mask_ms,
                avg_blit_ms = avg_blit_ms
            );

            frames = 0;
            accum_capture = Duration::ZERO;
            accum_mask = Duration::ZERO;
            accum_blit = Duration::ZERO;
            last_log = Instant::now();
        }
    }

    Ok(())
}
