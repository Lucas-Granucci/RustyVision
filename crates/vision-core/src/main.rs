mod camera;
mod config;
mod detection;

use config::Config;
use minifb::{Key, Window, WindowOptions};
use std::time::{Duration, Instant};

use crate::{
    camera::{capture_frame_into, get_camera},
    detection::run_ball_detection,
    detection::run_color_mask,
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
    let mut accum_cont = Duration::ZERO;

    // Create buffers
    let pixel_count = (config.camera.width * config.camera.height) as usize;
    let mut window_buf: Vec<u32> = vec![0; pixel_count];

    let mut rgb_buf: Vec<u8> = vec![0; pixel_count * 3];
    let mut mask_buf: Vec<u8> = vec![0; pixel_count];
    let mut contour_buf: Vec<u8> = vec![0; pixel_count];
    let mut circle_buf: Vec<u8> = vec![0; pixel_count];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Camera capture into RGB buf
        let (_, _) = capture_frame_into(&mut camera, &mut rgb_buf)?;

        run_color_mask(&rgb_buf, &config.detection, &mut mask_buf);

        let t_cont = Instant::now();
        contour_buf.fill(0);
        circle_buf.fill(0);
        run_ball_detection(
            &mask_buf,
            config.camera.width,
            config.camera.height,
            &mut contour_buf,
            &mut circle_buf,
        );
        let cont_dt = t_cont.elapsed();

        for (dst, &gray) in window_buf.iter_mut().zip(circle_buf.iter()) {
            let g = gray as u32;
            *dst = (g << 16) | (g << 8) | g;
        }
        window.update_with_buffer(
            &window_buf,
            config.camera.width as usize,
            config.camera.height as usize,
        )?;

        frames += 1;
        accum_cont += cont_dt;

        let elapsed = last_log.elapsed();
        if elapsed >= Duration::from_secs(1) && frames > 0 {
            let fps = frames as f64 / elapsed.as_secs_f64();
            let denom = frames as f64;
            let avg_cont_ms = accum_cont.as_secs_f64() * 1000.0 / denom;

            // tracing::info!(
            //     fps = fps,
            //     frames_in_window = frames,
            //     avg_cont_ms = avg_cont_ms,
            // );

            frames = 0;
            accum_cont = Duration::ZERO;
            last_log = Instant::now();
        }
    }

    Ok(())
}
