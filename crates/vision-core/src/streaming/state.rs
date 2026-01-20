use crate::config::{Config, DetectionConfig};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub type Frame = Vec<u8>;

#[derive(Clone)]
pub struct FrameHub {
    tx: broadcast::Sender<Frame>,
}

impl FrameHub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(16);
        Self { tx }
    }
    pub fn subscribe(&self) -> broadcast::Receiver<Frame> {
        self.tx.subscribe()
    }
    pub fn publish(&self, frame: Frame) {
        let _ = self.tx.send(frame);
    }
}

#[derive(Clone)]
pub struct AppState {
    pub raw_frames: FrameHub,
    pub mask_frames: FrameHub,
    pub contour_frames: FrameHub,
    pub circle_frames: FrameHub,
    pub config: Arc<RwLock<Config>>,
}

impl AppState {
    pub fn new(
        raw_hub: FrameHub,
        mask_hub: FrameHub,
        contour_hub: FrameHub,
        circle_hub: FrameHub,
        config: Config,
    ) -> Self {
        Self {
            raw_frames: raw_hub,
            mask_frames: mask_hub,
            contour_frames: contour_hub,
            circle_frames: circle_hub,
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn get_detection(&self) -> DetectionConfig {
        self.config.read().await.detection.clone()
    }
}
