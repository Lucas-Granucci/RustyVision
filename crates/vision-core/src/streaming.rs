use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
};
use bytes::BytesMut;
use image::{GrayImage, ImageBuffer};
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
    mask_frames: FrameHub,
    contour_frames: FrameHub,
    circle_frames: FrameHub,
}

// Convert grayscale ndarray to JPEG bytes
pub fn array_to_jpeg(arr: ArrayView2<u8>) -> Option<Vec<u8>> {
    let (height, width) = arr.dim();
    let img: GrayImage = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
        image::Luma([arr[[y as usize, x as usize]]])
    });
    let mut buf = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 60)
        .encode_image(&img)
        .ok()?;
    Some(buf)
}

pub async fn run_dashboard_server(
    mask_hub: FrameHub,
    contour_hub: FrameHub,
    circle_hub: FrameHub,
) -> anyhow::Result<()> {
    let state = AppState {
        mask_frames: mask_hub,
        contour_frames: contour_hub,
        circle_frames: circle_hub,
    };
    let app = axum::Router::new()
        .route("/", get(index_page))
        .route("/stream/mask", get(stream_mask))
        .route("/stream/contours", get(stream_contours))
        .route("/stream/circles", get(stream_circles))
        .with_state(state);

    let addr: SocketAddr = "0.0.0.0:5800".parse()?;
    tracing::info!("dashboard listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
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
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>RustyVision Streams</title>
    <style>
        body {
            margin: 0;
            padding: 20px;
            background: #1a1a1a;
            font-family: monospace;
            color: #fff;
        }
        h1 {
            text-align: center;
            color: #4a9eff;
            margin-bottom: 30px;
        }
        .streams {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
            max-width: 1400px;
            margin: 0 auto;
        }
        .stream-box {
            background: #2a2a2a;
            border: 2px solid #4a9eff;
            border-radius: 8px;
            padding: 15px;
            box-shadow: 0 4px 12px rgba(74, 158, 255, 0.3);
        }
        .stream-box h2 {
            margin: 0 0 10px 0;
            color: #4a9eff;
            font-size: 18px;
        }
        .stream-box img {
            width: 100%;
            height: auto;
            display: block;
            border: 1px solid #333;
        }
    </style>
</head>
<body>
    <h1>RustyVision - Multi-Stream Dashboard</h1>
    <div class="streams">
        <div class="stream-box">
            <h2>Mask (HSV Color Filter)</h2>
            <img src="/stream/mask" alt="Mask stream" />
        </div>
        <div class="stream-box">
            <h2>Contours</h2>
            <img src="/stream/contours" alt="Contours stream" />
        </div>
        <div class="stream-box">
            <h2>Detected Circles</h2>
            <img src="/stream/circles" alt="Circles stream" />
        </div>
    </div>
</body>
</html>
"#,
    )
}
