use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
};
use bytes::BytesMut;
use image::{DynamicImage, GrayImage, ImageBuffer, Rgb, RgbImage};
use ndarray::ArrayView2;
use std::net::SocketAddr;
use tokio::sync::broadcast;
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
struct AppState {
    raw_frames: FrameHub,
    mask_frames: FrameHub,
    contour_frames: FrameHub,
    circle_frames: FrameHub,
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
) -> anyhow::Result<()> {
    let state = AppState {
        raw_frames: raw_hub,
        mask_frames: mask_hub,
        contour_frames: contour_hub,
        circle_frames: circle_hub,
    };
    let app = axum::Router::new()
        .route("/", get(index_page))
        .route("/stream/raw", get(stream_raw))
        .route("/stream/mask", get(stream_mask))
        .route("/stream/contours", get(stream_contours))
        .route("/stream/circles", get(stream_circles))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:5800".parse()?;
    tracing::info!("dashboard listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
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

async fn index_page() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html")],
        r#"<!DOCTYPE html>
<html>
<head>
    <title>RustyVision Dashboard</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            background-color: #0f172a;
            color: #f8fafc;
            font-family: 'Courier New', Courier, monospace;
            height: 100vh;
            display: flex;
            overflow: hidden;
        }

        .sidebar {
            width: 340px;
            background: #1e293b;
            border-right: 1px solid #334155;
            display: flex;
            flex-direction: column;
        }

        .sidebar-header {
            padding: 15px;
            border-bottom: 1px solid #334155;
            background: #0f172a;
        }
        .sidebar-header h1 { color: #38bdf8; font-size: 1.2rem; text-transform: uppercase; letter-spacing: 2px; }
        .team-badge { background: #3b82f6; color: #fff; padding: 4px 10px; font-weight: bold; border-radius: 4px; display: inline-block; margin-top: 8px; font-size: 0.9rem; }

        .config-panel {
            flex: 1;
            overflow-y: auto;
            padding: 15px;
        }

        .config-section {
            background: #0f172a;
            border: 1px solid #334155;
            border-radius: 6px;
            margin-bottom: 15px;
        }

        .config-section-title {
            padding: 10px 12px;
            color: #38bdf8;
            font-weight: bold;
            font-size: 0.8rem;
            border-bottom: 1px solid #334155;
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        .config-section-controls {
            padding: 12px;
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 10px;
        }

        .input-group { display: flex; flex-direction: column; }
        .input-group label { font-size: 0.7rem; color: #64748b; margin-bottom: 4px; text-transform: uppercase; }
        .input-group input[type="number"] {
            background: #334155; border: none; color: white; padding: 6px; border-radius: 4px; font-size: 0.85rem; width: 100%;
        }

        .main-view {
            flex: 1;
            display: flex;
            flex-direction: column;
            background: #0f172a;
        }

        .main-header {
            padding: 15px 20px;
            border-bottom: 1px solid #334155;
        }
        .main-header h2 { color: #38bdf8; font-size: 1.1rem; text-transform: uppercase; letter-spacing: 1px; }

        .grid-container {
            flex: 1;
            padding: 15px;
            display: grid;
            grid-template-columns: 1fr 1fr;
            grid-template-rows: 1fr 1fr;
            gap: 15px;
        }

        .view-window {
            background: #1e293b;
            border: 1px solid #334155;
            border-radius: 8px;
            display: flex;
            flex-direction: column;
            min-height: 0;
        }

        .view-title {
            padding: 8px 12px;
            background: #0f172a;
            border-bottom: 1px solid #334155;
            color: #94a3b8;
            font-weight: bold;
            font-size: 0.75rem;
            text-transform: uppercase;
        }

        .view-display {
            flex: 1;
            background: #000;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 0;
        }
        .view-display img { max-width: 100%; max-height: 100%; display: block; object-fit: contain; }
    </style>
</head>
<body>
    <div class="sidebar">
        <div class="sidebar-header">
            <h1>RustyVision</h1>
            <div class="team-badge">3082</div>
        </div>
        <div class="config-panel">
            <div class="config-section">
                <div class="config-section-title">HSV Threshold</div>
                <div class="config-section-controls">
                    <div class="input-group">
                        <label>H Lower</label>
                        <input type="number" value="30">
                    </div>
                    <div class="input-group">
                        <label>H Upper</label>
                        <input type="number" value="90">
                    </div>
                    <div class="input-group">
                        <label>S Lower</label>
                        <input type="number" value="100">
                    </div>
                    <div class="input-group">
                        <label>S Upper</label>
                        <input type="number" value="255">
                    </div>
                    <div class="input-group">
                        <label>V Lower</label>
                        <input type="number" value="100">
                    </div>
                    <div class="input-group">
                        <label>V Upper</label>
                        <input type="number" value="255">
                    </div>
                </div>
            </div>

            <div class="config-section">
                <div class="config-section-title">Contour Filter</div>
                <div class="config-section-controls">
                    <div class="input-group">
                        <label>Min Area</label>
                        <input type="number" value="500">
                    </div>
                    <div class="input-group">
                        <label>Min Length</label>
                        <input type="number" value="50">
                    </div>
                </div>
            </div>

            <div class="config-section">
                <div class="config-section-title">Circle Hough</div>
                <div class="config-section-controls">
                    <div class="input-group">
                        <label>Vote Thresh</label>
                        <input type="number" value="25">
                    </div>
                    <div class="input-group">
                        <label>Min Radius</label>
                        <input type="number" value="10">
                    </div>
                </div>
            </div>
        </div>
    </div>

    <div class="main-view">
        <div class="main-header">
            <h2>Pipeline Views</h2>
        </div>
        <div class="grid-container">
            <div class="view-window">
                <div class="view-title">Raw Footage</div>
                <div class="view-display">
                    <img src="/stream/raw" />
                </div>
            </div>

            <div class="view-window">
                <div class="view-title">HSV Threshold</div>
                <div class="view-display">
                    <img src="/stream/mask" />
                </div>
            </div>

            <div class="view-window">
                <div class="view-title">Contour Filter</div>
                <div class="view-display">
                    <img src="/stream/contours" />
                </div>
            </div>

            <div class="view-window">
                <div class="view-title">Circle Hough</div>
                <div class="view-display">
                    <img src="/stream/circles" />
                </div>
            </div>
        </div>
    </div>
</body>
</html>
"#,
    )
}
