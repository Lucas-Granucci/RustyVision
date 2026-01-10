use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub system: SystemConfig,
    pub networktables: NetworkTablesConfig,
    pub camera: CameraConfig,
    pub detection: DetectionConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SystemConfig {
    pub log_level: String,
    pub telemetry_enabled: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NetworkTablesConfig {
    pub server: String,
    pub identity: String,
    pub publish_rate_hz: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CameraConfig {
    pub device_id: u32,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DetectionConfig {
    pub enabled: bool,
    pub color_lower: [u8; 3],
    pub color_upper: [u8; 3],
    pub min_area: i32,
}

impl Config {
    // Load config from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    // Load default config
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        Self::from_file("config/default.toml")
    }

    // Default config in memory if file doesn't exist
    pub fn default() -> Self {
        Config {
            system: SystemConfig {
                log_level: "info".to_string(),
                telemetry_enabled: true,
            },
            networktables: NetworkTablesConfig {
                server: "10.0.0.2".to_string(),
                identity: "vision-coprocessor".to_string(),
                publish_rate_hz: 50,
            },
            camera: CameraConfig {
                device_id: 0,
                width: 1280,
                height: 720,
                fps: 30,
            },
            detection: DetectionConfig {
                enabled: true,
                color_lower: [20, 100, 100],
                color_upper: [30, 255, 255],
                min_area: 100,
            },
        }
    }
}
