mod camera;
mod config;
mod frame;

use camera::{open_camera, Camera};
use config::Config;

fn main() {
    println!("RustyVision waking up...");

    let config = match Config::load_default() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Could not find configuration file: {}", e);
            eprintln!("Using default configuration");
            Config::default()
        }
    };

    let camera = open_camera(
        config.camera.device_id,
        config.camera.width,
        config.camera.height,
        config.camera.fps,
    );
}
