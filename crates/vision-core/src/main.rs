mod camera;
mod config;
mod detection;
mod streaming;

use config::Config;
use ndarray::Array2;
use std::time::Duration;
use tokio::time::Instant;
use vision_detection::circle::precompute_circle_points;

use crate::{
    camera::{capture_frame, get_camera, resize_array},
    detection::{detect_circles, detect_contours, run_color_mask},
    streaming::{array_to_jpeg, run_dashboard_server, FrameHub},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
    tracing::info!("RustyVision waking up...");

    // Create FrameHub for streaming
    let frame_hub = FrameHub::new();
    let hub_clone = frame_hub.clone();

    tokio::spawn(async move {
        if let Err(e) = run_dashboard_server(hub_clone).await {
            tracing::error!("Dashboard server error: {}", e);
        }
    });

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

    // Create buffers
    let mut rgb_frame: Array2<[u8; 3]> = Array2::from_elem((height, width), [0u8; 3]);
    let mut rgb_resized: Array2<[u8; 3]> = Array2::from_elem((proc_height, proc_width), [0u8; 3]);
    let mut mask_arr: Array2<u8> = Array2::zeros((proc_height, proc_width));
    let mut contour_arr: Array2<u8> = Array2::zeros((proc_height, proc_width));
    let mut circle_arr: Array2<u8> = Array2::zeros((proc_height, proc_width));

    // Circle cache
    let circle_cache = precompute_circle_points(min_radius, max_radius, radius_step);

    // Run vision processing in blocking task
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut camera = get_camera(camera_device_id)?;
        camera.open_stream()?;

        let mut frame_counter = 0u32;
        let mut last_log = Instant::now();

        loop {
            // Camera capture into RGB buf
            capture_frame(&mut camera, &mut rgb_frame)?;
            resize_array(rgb_frame.view(), &mut rgb_resized, proc_height, proc_width);
            run_color_mask(rgb_resized.view(), &mut mask_arr, color_lower, color_upper);
            detect_contours(mask_arr.view(), &mut contour_arr, min_length, min_area);
            detect_circles(contour_arr.view(), &mut circle_arr, &circle_cache);

            if let Some(jpeg_data) = array_to_jpeg(circle_arr.view()) {
                frame_hub.publish(jpeg_data);
            }

            frame_counter += 1;
            if last_log.elapsed() >= Duration::from_secs(1) {
                let fps = frame_counter as f64 / last_log.elapsed().as_secs_f64();
                tracing::info!("Stream FPS: {:.1}", fps);
                frame_counter = 0;
                last_log = Instant::now();
            }
        }
    })
    .await??;
    Ok(())
}
