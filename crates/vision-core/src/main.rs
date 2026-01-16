mod camera;
mod config;
mod detection;

use config::Config;
use minifb::{Key, Window, WindowOptions};
use ndarray::Array2;
use std::time::{Duration, Instant};
use vision_detection::circle::precompute_circle_points;

use crate::{
    camera::{capture_frame, get_camera, resize_array},
    detection::{detect_circles, detect_contours, run_color_mask},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("RustyVision waking up...");

    // Load config
    let config = Config::load_default().unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Using default configuration");
        Config::default()
    });

    // Read constants from config
    let camera_device_id = config.camera.device_id;
    let width = config.camera.width as usize;
    let height = config.camera.height as usize;

    let resize_factor: usize = 2;
    let proc_width = width / resize_factor;
    let proc_height = height / resize_factor;

    let min_radius = config.detection.min_radius / resize_factor as u32;
    let max_radius = config.detection.max_radius / resize_factor as u32;
    let radius_step = config.detection.radius_step;

    let min_length = config.detection.min_contour_length / resize_factor as u32;
    let min_area = config.detection.min_area / (resize_factor * resize_factor) as f32;
    let color_lower = config.detection.color_lower;
    let color_upper = config.detection.color_upper;

    // Camera stuff
    let mut camera = get_camera(camera_device_id)?;
    camera.open_stream()?;

    // Setup window
    let mut window = Window::new(
        "RustyVision",
        proc_width,
        proc_height,
        WindowOptions::default(),
    )?;
    window.set_target_fps(60);

    // Create buffers
    let pixel_count = proc_width * proc_height;
    let mut window_buf: Vec<u32> = vec![0; pixel_count];

    let mut rgb_frame: Array2<[u8; 3]> = Array2::from_elem((height, width), [0u8; 3]);

    let mut rgb_resized: Array2<[u8; 3]> = Array2::from_elem((proc_height, proc_width), [0u8; 3]);
    let mut mask_arr: Array2<u8> = Array2::zeros((proc_height, proc_width));
    let mut contour_arr: Array2<u8> = Array2::zeros((proc_height, proc_width));
    let mut circle_arr: Array2<u8> = Array2::zeros((proc_height, proc_width));

    // Circle cache
    let circle_cache = precompute_circle_points(min_radius, max_radius, radius_step);

    // Setup timing
    let mut frames: u64 = 0;
    let mut last_log = Instant::now();
    let mut accum_cont = Duration::ZERO;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Camera capture into RGB buf
        capture_frame(&mut camera, &mut rgb_frame)?;
        resize_array(rgb_frame.view(), &mut rgb_resized, proc_height, proc_width);

        // Run HSV color mask
        run_color_mask(rgb_resized.view(), &mut mask_arr, color_lower, color_upper);

        let t_cont = Instant::now();

        // Extract contours from mask
        detect_contours(mask_arr.view(), &mut contour_arr, min_length, min_area);

        // Run Houghs circle detection
        detect_circles(contour_arr.view(), &mut circle_arr, &circle_cache);
        let cont_dt = t_cont.elapsed();

        // Convert to RGB for display
        for ((dst, &gray), &mask) in window_buf
            .iter_mut()
            .zip(circle_arr.iter())
            .zip(contour_arr.iter())
        {
            let g = (gray + mask) as u32;
            *dst = (g << 16) | (g << 8) | g;
        }
        // for (dst, &gray) in window_buf.iter_mut().zip(circle_arr.iter()) {
        //     let g = gray as u32;
        //     *dst = (g << 16) | (g << 8) | g;
        // }

        window.update_with_buffer(&window_buf, proc_width, proc_height)?;

        // Timing
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
