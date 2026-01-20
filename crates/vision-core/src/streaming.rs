use crate::config::{Config, DetectionConfig};
use axum::Json;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
};
use bytes::BytesMut;
use image::{DynamicImage, GrayImage, ImageBuffer, Rgb, RgbImage};
use ndarray::ArrayView2;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

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

pub trait ToDynamicImage {
    fn to_dynamic_image(&self) -> Option<DynamicImage>;
}

impl<'a> ToDynamicImage for ArrayView2<'a, u8> {
    fn to_dynamic_image(&self) -> Option<DynamicImage> {
        let (height, width) = self.dim();
        let img: GrayImage = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
            image::Luma([self[[y as usize, x as usize]]])
        });
        Some(DynamicImage::ImageLuma8(img))
    }
}

impl<'a> ToDynamicImage for ArrayView2<'a, [u8; 3]> {
    fn to_dynamic_image(&self) -> Option<DynamicImage> {
        let (height, width) = self.dim();
        let img: RgbImage = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
            // Convert [u8; 3] to Rgb pixel
            Rgb(self[[y as usize, x as usize]])
        });
        Some(DynamicImage::ImageRgb8(img))
    }
}

// Convert grayscale ndarray to JPEG bytes
pub fn array_to_jpeg(arr: impl ToDynamicImage) -> Option<Vec<u8>> {
    let img = arr.to_dynamic_image()?;
    let mut buf = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 60)
        .encode_image(&img)
        .ok()?;
    Some(buf)
}

pub async fn run_dashboard_server(
    raw_hub: FrameHub,
    mask_hub: FrameHub,
    contour_hub: FrameHub,
    circle_hub: FrameHub,
    config: Config,
) -> anyhow::Result<AppState> {
    let state = AppState::new(raw_hub, mask_hub, contour_hub, circle_hub, config);
    let state_for_axum = state.clone();

    let app = axum::Router::new()
        .route("/", get(index_page))
        .route(
            "/config",
            get(get_config_handler).post(update_config_handler),
        )
        .route("/stream/raw", get(stream_raw))
        .route("/stream/mask", get(stream_mask))
        .route("/stream/contours", get(stream_contours))
        .route("/stream/circles", get(stream_circles))
        .with_state(state_for_axum);

    let addr: SocketAddr = "0.0.0.0:5800".parse()?;
    tracing::info!("Dashboard listening on http://{}", addr);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await {
            tracing::error!("Serving error: {}", e)
        }
    });

    Ok(state)
}

async fn stream_raw(State(state): State<AppState>) -> impl IntoResponse {
    stream_mjpeg_internal(state.raw_frames).await
}

async fn stream_mask(State(state): State<AppState>) -> impl IntoResponse {
    stream_mjpeg_internal(state.mask_frames).await
}

async fn stream_contours(State(state): State<AppState>) -> impl IntoResponse {
    stream_mjpeg_internal(state.contour_frames).await
}

async fn stream_circles(State(state): State<AppState>) -> impl IntoResponse {
    stream_mjpeg_internal(state.circle_frames).await
}

async fn stream_mjpeg_internal(hub: FrameHub) -> impl IntoResponse {
    let rx = hub.subscribe();
    let stream = BroadcastStream::new(rx)
        .filter_map(|result| result.ok())
        .map(|frame| {
            let mut buf = BytesMut::new();
            buf.extend_from_slice(b"--frame\r\n");
            buf.extend_from_slice(b"Content-Type: image/jpeg\r\n");
            buf.extend_from_slice(format!("Content-Length: {}\r\n\r\n", frame.len()).as_bytes());
            buf.extend_from_slice(&frame);
            buf.extend_from_slice(b"\r\n");
            Ok::<_, std::io::Error>(buf.freeze())
        });

    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "multipart/x-mixed-replace; boundary=frame",
        )],
        axum::body::Body::from_stream(stream),
    )
}

async fn get_config_handler(State(state): State<AppState>) -> Json<DetectionConfig> {
    tracing::info!("getting config");
    Json(state.get_detection().await)
}

async fn update_config_handler(
    State(state): State<AppState>,
    Json(new_detection_cfg): Json<DetectionConfig>,
) -> impl IntoResponse {
    tracing::info!("Received configuration update request");
    let mut config = state.config.write().await;
    tracing::debug!("New Config Values: {:?}", new_detection_cfg);
    config.detection = new_detection_cfg;
    tracing::info!("Configuration successfully updated in AppState");
    StatusCode::OK
}

async fn index_page() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html")],
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>RustyVision Studio</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600&display=swap" rel="stylesheet">
    <style>
        :root {
            --primary-blue: #2563eb;
            --primary-blue-hover: #1d4ed8;
            --bg-gray: #f1f5f9;
            --bg-white: #ffffff;
            --border: #cbd5e1;
            --text-primary: #0f172a;
            --text-secondary: #64748b;
            --sidebar-width: 300px;
        }

        * { margin: 0; padding: 0; box-sizing: border-box; }

        body {
            background: var(--bg-gray);
            color: var(--text-primary);
            font-family: 'Inter', sans-serif;
            height: 100vh;
            display: flex;
            overflow: hidden;
        }

        .sidebar {
            width: var(--sidebar-width);
            background: var(--bg-white);
            border-right: 1px solid var(--border);
            display: flex;
            flex-direction: column;
            transition: width 0.2s;
            overflow: hidden;
        }

        .sidebar.collapsed {
            width: 0;
        }

        .sidebar-header {
            padding: 20px;
            border-bottom: 1px solid var(--border);
            background: var(--primary-blue);
            color: white;
        }

        .brand-text {
            font-weight: 600;
            font-size: 1.1rem;
        }

        .sidebar-content {
            flex: 1;
            overflow-y: auto;
            overflow-x: hidden;
            padding: 20px;
        }

        .control-group {
            margin-bottom: 24px;
        }

        .group-label {
            font-size: 0.85rem;
            font-weight: 600;
            color: var(--text-primary);
            margin-bottom: 12px;
            padding-bottom: 6px;
            border-bottom: 2px solid var(--primary-blue);
        }

        .input-grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 12px;
        }

        .field {
            display: flex;
            flex-direction: column;
            gap: 4px;
        }

        .field label {
            font-size: 0.8rem;
            color: var(--text-secondary);
            font-weight: 500;
        }

        .field input {
            padding: 8px 10px;
            border: 1px solid var(--border);
            border-radius: 4px;
            font-size: 0.9rem;
            font-family: 'Inter', sans-serif;
            width: 100%;
            min-width: 0;
        }

        .field input:focus {
            outline: none;
            border-color: var(--primary-blue);
        }

        .sidebar-footer {
            padding: 20px;
            border-top: 1px solid var(--border);
        }

        .btn-primary {
            width: 100%;
            background: var(--primary-blue);
            color: white;
            border: none;
            padding: 12px;
            border-radius: 4px;
            font-weight: 600;
            cursor: pointer;
            font-size: 0.9rem;
        }

        .btn-primary:hover {
            background: var(--primary-blue-hover);
        }

        .btn-primary:disabled {
            opacity: 0.6;
            cursor: not-allowed;
        }

        .main-stage {
            flex: 1;
            display: flex;
            flex-direction: column;
        }

        .top-bar {
            background: var(--bg-white);
            border-bottom: 1px solid var(--border);
            padding: 16px 24px;
            display: flex;
            align-items: center;
            gap: 16px;
        }

        .toggle-btn {
            background: var(--bg-white);
            border: 1px solid var(--border);
            color: var(--text-secondary);
            width: 36px;
            height: 36px;
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
        }

        .toggle-btn:hover {
            background: var(--bg-gray);
            border-color: var(--primary-blue);
        }

        .toggle-btn svg {
            width: 20px;
            height: 20px;
        }

        .page-title {
            font-weight: 600;
            font-size: 1.1rem;
            color: var(--text-primary);
        }

        .video-grid {
            flex: 1;
            padding: 20px;
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            grid-template-rows: repeat(2, 1fr);
            gap: 20px;
            overflow: hidden;
        }

        .video-card {
            background: var(--bg-white);
            border: 1px solid var(--border);
            border-radius: 4px;
            display: flex;
            flex-direction: column;
            overflow: hidden;
        }

        .card-header {
            padding: 10px 16px;
            font-size: 0.85rem;
            font-weight: 600;
            color: var(--text-primary);
            border-bottom: 1px solid var(--border);
            background: var(--bg-gray);
        }

        .card-view {
            flex: 1;
            background: #000;
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .card-view img {
            max-width: 100%;
            max-height: 100%;
            object-fit: contain;
        }

        .toast {
            font-size: 0.85rem;
            color: white;
            background: #10b981;
            text-align: center;
            margin-top: 10px;
            font-weight: 500;
            opacity: 0;
            transition: opacity 0.3s;
            padding: 8px;
            border-radius: 4px;
            display: none;
        }

        .toast.visible {
            opacity: 1;
            display: block;
        }
    </style>
</head>
<body>

    <div class="sidebar" id="sidebar">
        <div class="sidebar-header">
            <div class="brand-text">RustyVision</div>
        </div>

        <div class="sidebar-content">
            <div class="control-group">
                <div class="group-label">HSV Thresholds</div>
                <div class="input-grid">
                    <div class="field"><label>H Min</label><input type="number" id="h_low"></div>
                    <div class="field"><label>H Max</label><input type="number" id="h_high"></div>
                    <div class="field"><label>S Min</label><input type="number" id="s_low"></div>
                    <div class="field"><label>S Max</label><input type="number" id="s_high"></div>
                    <div class="field"><label>V Min</label><input type="number" id="v_low"></div>
                    <div class="field"><label>V Max</label><input type="number" id="v_high"></div>
                </div>
            </div>

            <div class="control-group">
                <div class="group-label">Morphology</div>
                <div class="input-grid">
                    <div class="field"><label>Min Area</label><input type="number" id="min_area"></div>
                    <div class="field"><label>Min Length</label><input type="number" id="min_length"></div>
                </div>
            </div>

            <div class="control-group">
                <div class="group-label">Hough Circles</div>
                <div class="input-grid">
                    <div class="field"><label>Vote</label><input type="number" id="vote_thresh"></div>
                    <div class="field"><label>Step</label><input type="number" id="radius_step"></div>
                    <div class="field"><label>Rad Min</label><input type="number" id="min_radius"></div>
                    <div class="field"><label>Rad Max</label><input type="number" id="max_radius"></div>
                </div>
            </div>
        </div>

        <div class="sidebar-footer">
            <button id="save_btn" class="btn-primary">Save Config</button>
            <div id="status_msg" class="toast">✓ Changes Saved</div>
        </div>
    </div>

    <div class="main-stage">
        <div class="top-bar">
            <button class="toggle-btn" id="sidebar_toggle" title="Toggle Sidebar">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"><line x1="3" y1="12" x2="21" y2="12"></line><line x1="3" y1="6" x2="21" y2="6"></line><line x1="3" y1="18" x2="21" y2="18"></line></svg>
            </button>
            <div class="page-title">Live Pipeline</div>
        </div>

        <div class="video-grid">
            <div class="video-card">
                <div class="card-header">Raw Input</div>
                <div class="card-view"><img src="/stream/raw" /></div>
            </div>
            <div class="video-card">
                <div class="card-header">HSV Mask</div>
                <div class="card-view"><img src="/stream/mask" /></div>
            </div>
            <div class="video-card">
                <div class="card-header">Contours</div>
                <div class="card-view"><img src="/stream/contours" /></div>
            </div>
            <div class="video-card">
                <div class="card-header">Detection</div>
                <div class="card-view"><img src="/stream/circles" /></div>
            </div>
        </div>
    </div>

    <script>
        const val = (id) => parseFloat(document.getElementById(id).value) || 0;

        const sidebar = document.getElementById('sidebar');
        const toggleBtn = document.getElementById('sidebar_toggle');

        toggleBtn.addEventListener('click', () => {
            sidebar.classList.toggle('collapsed');
        });

        async function loadConfig() {
            try {
                const res = await fetch('/config');
                const cfg = await res.json();

                document.getElementById('h_low').value = cfg.color_lower[0];
                document.getElementById('s_low').value = cfg.color_lower[1];
                document.getElementById('v_low').value = cfg.color_lower[2];
                document.getElementById('h_high').value = cfg.color_upper[0];
                document.getElementById('s_high').value = cfg.color_upper[1];
                document.getElementById('v_high').value = cfg.color_upper[2];
                document.getElementById('min_area').value = cfg.min_area;
                document.getElementById('min_length').value = cfg.min_contour_length;
                document.getElementById('min_radius').value = cfg.min_radius;
                document.getElementById('max_radius').value = cfg.max_radius;
                document.getElementById('radius_step').value = cfg.radius_step;
                document.getElementById('vote_thresh').value = cfg.vote_thresh;

            } catch (e) { console.error("Config load error", e); }
        }

        async function updateConfig() {
            const btn = document.getElementById('save_btn');
            const status = document.getElementById('status_msg');
            const originalText = btn.innerHTML;

            btn.innerHTML = "Saving...";
            btn.disabled = true;

            const data = {
                color_lower: [val('h_low'), val('s_low'), val('v_low')],
                color_upper: [val('h_high'), val('s_high'), val('v_high')],
                min_area: val('min_area'),
                min_contour_length: Math.floor(val('min_length')),
                min_radius: Math.floor(val('min_radius')),
                max_radius: Math.floor(val('max_radius')),
                radius_step: Math.floor(val('radius_step')),
                vote_thresh: Math.floor(val('vote_thresh'))
            };

            try {
                const response = await fetch('/config', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(data)
                });

                if (response.ok) {
                    status.classList.add('visible');
                    setTimeout(() => status.classList.remove('visible'), 2000);
                }
            } catch (e) {
                alert("Failed to save");
            } finally {
                btn.innerHTML = originalText;
                btn.disabled = false;
            }
        }

        document.getElementById('save_btn').addEventListener('click', updateConfig);
        loadConfig();
    </script>
</body>
</html>
"#,
    )
}
