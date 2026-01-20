use super::routes::{
    get_config_handler, stream_circles, stream_contours, stream_mask, stream_raw,
    update_config_handler,
};
use super::state::{AppState, FrameHub};
use super::ui::index_page;
use crate::Config;
use axum::routing::get;
use std::net::SocketAddr;

pub async fn run_dashboard_server(
    raw_hub: FrameHub,
    mask_hub: FrameHub,
    contour_hub: FrameHub,
    circle_hub: FrameHub,
    config: Config,
) -> anyhow::Result<AppState> {
    let port = config.web.port;
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

    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    tracing::info!("Dashboard listening on http://{}", addr);

    tokio::spawn(async move {
        if let Err(e) = axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await {
            tracing::error!("Serving error: {}", e)
        }
    });

    Ok(state)
}
