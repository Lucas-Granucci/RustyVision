use super::state::{AppState, FrameHub};
use crate::config::DetectionConfig;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use bytes::BytesMut;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

pub async fn stream_raw(State(state): State<AppState>) -> impl IntoResponse {
    stream_mjpeg_internal(state.raw_frames).await
}

pub async fn stream_mask(State(state): State<AppState>) -> impl IntoResponse {
    stream_mjpeg_internal(state.mask_frames).await
}

pub async fn stream_contours(State(state): State<AppState>) -> impl IntoResponse {
    stream_mjpeg_internal(state.contour_frames).await
}

pub async fn stream_circles(State(state): State<AppState>) -> impl IntoResponse {
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

pub async fn get_config_handler(State(state): State<AppState>) -> Json<DetectionConfig> {
    tracing::info!("getting config");
    Json(state.get_detection().await)
}

pub async fn update_config_handler(
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
