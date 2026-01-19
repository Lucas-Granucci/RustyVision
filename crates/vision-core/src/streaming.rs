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
        .route("/stream/mask", get(stream_mask))
        .route("/stream/contour", get(stream_contours))
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
