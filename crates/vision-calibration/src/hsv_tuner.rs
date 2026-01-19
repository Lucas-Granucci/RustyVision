fn main() {}
// use minifb::{Key, KeyRepeat, Window, WindowOptions};
// use ndarray::Array2;
// use std::time::{Duration, Instant};

// use vision_core::{
//     camera::{capture_frame, get_camera, resize_array},
//     config::Config,
//     detection::run_color_mask,
// };

// struct HsvTunerState {
//     lower: [u8; 3],
//     upper: [u8; 3],
//     step: u8,
// }

// fn handle_key_input(window: &Window, state: &mut HsvTunerState) {
//     let step = state.step;

//     let keys = [
//         (Key::A, false, 0, true), // (key, is_upper, channel, is_add)
//         (Key::Z, false, 0, false),
//         (Key::S, false, 1, true),
//         (Key::X, false, 1, false),
//         (Key::D, false, 2, true),
//         (Key::C, false, 2, false),
//         (Key::F, true, 0, true),
//         (Key::V, true, 0, false),
//         (Key::G, true, 1, true),
//         (Key::B, true, 1, false),
//         (Key::H, true, 2, true),
//         (Key::N, true, 2, false),
//     ];

//     for (key, is_upper, ch, is_add) in keys {
//         if window.is_key_pressed(key, KeyRepeat::No) {
//             let bounds = if is_upper {
//                 &mut state.upper
//             } else {
//                 &mut state.lower
//             };
//             bounds[ch] = if is_add {
//                 bounds[ch].saturating_add(step)
//             } else {
//                 bounds[ch].saturating_sub(step)
//             };
//         }
//     }
// }

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     tracing_subscriber::fmt::init();
//     tracing::info!("RustyVision waking up...");

//     // Load config
//     let config = Config::load_default().unwrap_or_else(|e| {
//         tracing::warn!(error = %e, "Using default configuration");
//         Config::default()
//     });

//     // Read constants from config
//     let camera_device_id = config.camera.device_id;
//     let width = config.camera.width as usize;
//     let height = config.camera.height as usize;

//     let resize_factor: usize = 2;
//     let proc_width = width / resize_factor;
//     let proc_height = height / resize_factor;

//     let mut state = HsvTunerState {
//         lower: config.detection.color_lower,
//         upper: config.detection.color_upper,
//         step: 1,
//     };

//     // Camera stuff
//     let mut camera = get_camera(camera_device_id)?;
//     camera.open_stream()?;

//     // Setup window
//     let mut window = Window::new(
//         "RustyVision",
//         proc_width,
//         proc_height,
//         WindowOptions::default(),
//     )?;
//     window.set_target_fps(60);

//     // Create buffers
//     let pixel_count = proc_width * proc_height;
//     let mut window_buf: Vec<u32> = vec![0; pixel_count];

//     let mut rgb_frame: Array2<[u8; 3]> = Array2::from_elem((height, width), [0u8; 3]);
//     let mut rgb_resized: Array2<[u8; 3]> = Array2::from_elem((proc_height, proc_width), [0u8; 3]);
//     let mut mask_arr: Array2<u8> = Array2::zeros((proc_height, proc_width));

//     // Setup timing
//     let mut frames: u64 = 0;
//     let mut last_log = Instant::now();

//     while window.is_open() && !window.is_key_down(Key::Escape) {
//         // Handle input for HSV adjustments
//         handle_key_input(&window, &mut state);

//         // Camera capture into RGB buf
//         capture_frame(&mut camera, &mut rgb_frame)?;
//         resize_array(rgb_frame.view(), &mut rgb_resized, proc_height, proc_width);

//         // Run HSV color mask
//         run_color_mask(rgb_resized.view(), &mut mask_arr, state.lower, state.upper);

//         // Convert to RGB for display
//         for (dst, &gray) in window_buf.iter_mut().zip(mask_arr.iter()) {
//             let g = gray as u32;
//             *dst = (g << 16) | (g << 8) | g;
//         }

//         window.update_with_buffer(&window_buf, proc_width, proc_height)?;

//         // Timing
//         frames += 1;

//         let elapsed = last_log.elapsed();
//         if elapsed >= Duration::from_secs(1) && frames > 0 {
//             tracing::info!(
//                 frames_in_window = frames,
//                 hsv_lower = ?state.lower,
//                 hsv_upper = ?state.upper,
//                 step = state.step
//             );
//             frames = 0;
//             last_log = Instant::now();
//         }
//     }

//     Ok(())
// }
