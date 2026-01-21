use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};

pub async fn index_page() -> impl IntoResponse {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html")],
        r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>RustyVision</title>
            <link href="https://fonts.googleapis.com/css2?family=Space+Mono:wght@400;700&display=swap" rel="stylesheet">
            <style>
                * { margin: 0; padding: 0; box-sizing: border-box; }

                body {
                    background: #fff;
                    color: #000;
                    font-family: 'Space Mono', monospace;
                    height: 100vh;
                    overflow: hidden;
                }

                .wrapper {
                    height: 100vh;
                    display: flex;
                    flex-direction: column;
                }

                /* --- Header --- */
                .header-bar {
                    padding: 15px 20px;
                    border-bottom: 2px solid #000;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    flex-shrink: 0;
                    background: #fff;
                    z-index: 20;
                }

                .brand {
                    font-weight: 700;
                    font-size: 1.2rem;
                    letter-spacing: -1px;
                }

                .toggle-controls {
                    background: #000;
                    color: #fff;
                    border: none;
                    padding: 8px 16px;
                    font-family: 'Space Mono', monospace;
                    font-size: 0.8rem;
                    cursor: pointer;
                    transition: all 0.2s;
                }
                .toggle-controls:hover { background: #333; }

                /* --- Main Layout --- */
                .main {
                    flex: 1;
                    display: grid;
                    grid-template-columns: 320px 1fr;
                    transition: grid-template-columns 0.3s cubic-bezier(0.25, 1, 0.5, 1);
                    overflow: hidden;
                }

                .main.controls-hidden {
                    grid-template-columns: 0 1fr;
                }

                /* --- Sidebar --- */
                .sidebar {
                    border-right: 2px solid #000;
                    overflow: hidden;
                    display: flex;
                    flex-direction: column;
                    background: #f4f4f4;
                    min-width: 320px; /* Prevent squishing during transition */
                }

                .controls-area {
                    flex: 1;
                    overflow-y: auto;
                    padding: 20px;
                }

                .section { margin-bottom: 30px; }

                .section-head {
                    font-size: 0.7rem;
                    font-weight: 700;
                    text-transform: uppercase;
                    letter-spacing: 1px;
                    margin-bottom: 15px;
                    padding-bottom: 8px;
                    border-bottom: 1px solid #000;
                }

                .field-group {
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: 10px;
                    margin-bottom: 10px;
                }

                .field { display: flex; flex-direction: column; }
                .field-label { font-size: 0.65rem; margin-bottom: 4px; text-transform: uppercase; }
                .field input {
                    border: 1px solid #000;
                    background: #fff;
                    padding: 6px 8px;
                    width: 100%;
                    font-family: 'Space Mono', monospace;
                    font-size: 0.8rem;
                }
                .field input:focus { outline: 2px solid #000; outline-offset: -2px; }

                .checkbox-field {
                    display: flex;
                    align-items: center;
                    margin-bottom: 8px;
                    font-size: 0.8rem;
                    cursor: pointer;
                    user-select: none;
                }
                .checkbox-field input { margin-right: 10px; cursor: pointer; }

                .save-area {
                    padding: 12px 20px 14px;
                    border-top: 2px solid #000;
                    display: flex;
                    flex-direction: column;
                    gap: 8px;
                    background: #fff;
                }

                .save-btn {
                    width: 100%;
                    background: #000;
                    color: #fff;
                    border: none;
                    padding: 12px;
                    font-family: 'Space Mono', monospace;
                    font-size: 0.75rem;
                    font-weight: 700;
                    text-transform: uppercase;
                    cursor: pointer;
                    letter-spacing: 1px;
                }
                .save-btn:active { background: #333; }

                .save-status {
                    text-align: center;
                    font-size: 0.7rem;
                    opacity: 0;
                    height: 0;
                    overflow: hidden;
                    padding: 0 4px;
                    transition: opacity 0.3s, height 0.3s, padding 0.3s;
                    background: #000;
                    color: white;
                }
                .save-status.visible {
                    opacity: 1;
                    height: auto;
                    padding: 4px;
                }

                /* --- Dynamic Content Grid --- */
                .content-area {
                    background: #000;
                    padding: 2px;
                    display: grid;
                    gap: 2px;
                    overflow: hidden;
                }

                /* Grid Layout Modes
                   These are toggled via JS based on the count of active streams
                */

                /* Mode 3: Main (Detect) takes left half, Mask/Contours stack on right */
                .content-area.layout-3 {
                    grid-template-columns: 2fr 1fr;
                    grid-template-rows: 1fr 1fr;
                    grid-template-areas:
                        "main sub1"
                        "main sub2";
                }
                /* Need to map specific IDs to areas in Layout 3 */
                .layout-3 #feed_detect   { grid-area: main; }
                .layout-3 #feed_mask     { grid-area: sub1; }
                .layout-3 #feed_contours { grid-area: sub2; }

                /* Mode 2: Split screen (50/50 vertical) */
                .content-area.layout-2 {
                    grid-template-columns: 1fr 1fr;
                    grid-template-rows: 1fr;
                    /* Areas not strictly needed, flow will handle it, but for safety: */
                }

                /* Mode 1: Full screen */
                .content-area.layout-1 {
                    grid-template-columns: 1fr;
                    grid-template-rows: 1fr;
                }

                /* Mode 0: Nothing */
                .content-area.layout-0 {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }
                .content-area.layout-0::after {
                    content: "NO ACTIVE FEEDS";
                    color: #fff;
                }

                /* --- Feed Items --- */
                .feed {
                    background: #222;
                    position: relative;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    overflow: hidden;
                    width: 100%;
                    height: 100%;
                }

                /* When hidden, completely remove from layout flow */
                .feed.hidden {
                    display: none !important;
                }

                .feed-title {
                    position: absolute;
                    top: 10px;
                    left: 10px;
                    font-size: 0.65rem;
                    font-weight: 700;
                    text-transform: uppercase;
                    background: #fff;
                    color: #000;
                    padding: 4px 8px;
                    border: 1px solid #000;
                    z-index: 10;
                    pointer-events: none;
                }

                .feed img {
                    width: 100%;
                    height: 100%;
                    object-fit: contain; /* Ensures we see the whole frame */
                    display: block;
                }

            </style>
        </head>
        <body>
            <div class="wrapper">
                <div class="header-bar">
                    <div class="brand">RUSTYVISION</div>
                    <button class="toggle-controls" id="toggle_btn">CONTROLS</button>
                </div>

                <div class="main" id="main_area">
                    <div class="sidebar">
                        <div class="controls-area">
                            <div class="section">
                                <div class="section-head">Active Streams</div>
                                <div class="checkbox-field" onclick="toggleStream('detect')">
                                    <input type="checkbox" id="chk_detect" checked value="on"> <div>Main: Detect</div>
                                </div>
                                <div class="checkbox-field" onclick="toggleStream('mask')">
                                    <input type="checkbox" id="chk_mask" checked value="on"> <div>Sub: Mask</div>
                                </div>
                                <div class="checkbox-field" onclick="toggleStream('contours')">
                                    <input type="checkbox" id="chk_contours" checked value="on"> <div>Sub: Contours</div>
                                </div>
                            </div>

                            <div class="section">
                                <div class="section-head">HSV Range</div>
                                <div class="field-group">
                                    <div class="field"><div class="field-label">H Low</div><input type="number" id="h_low"></div>
                                    <div class="field"><div class="field-label">H High</div><input type="number" id="h_high"></div>
                                </div>
                                <div class="field-group">
                                    <div class="field"><div class="field-label">S Low</div><input type="number" id="s_low"></div>
                                    <div class="field"><div class="field-label">S High</div><input type="number" id="s_high"></div>
                                </div>
                                <div class="field-group">
                                    <div class="field"><div class="field-label">V Low</div><input type="number" id="v_low"></div>
                                    <div class="field"><div class="field-label">V High</div><input type="number" id="v_high"></div>
                                </div>
                            </div>

                            <div class="section">
                                <div class="section-head">Morph</div>
                                <div class="field-group">
                                    <div class="field"><div class="field-label">Area</div><input type="number" id="min_area"></div>
                                    <div class="field"><div class="field-label">Length</div><input type="number" id="min_length"></div>
                                </div>
                            </div>

                            <div class="section">
                                <div class="section-head">Circles</div>
                                <div class="field-group">
                                    <div class="field"><div class="field-label">Vote</div><input type="number" id="vote_thresh"></div>
                                    <div class="field"><div class="field-label">Step</div><input type="number" id="radius_step"></div>
                                </div>
                                <div class="field-group">
                                    <div class="field"><div class="field-label">R Min</div><input type="number" id="min_radius"></div>
                                    <div class="field"><div class="field-label">R Max</div><input type="number" id="max_radius"></div>
                                </div>
                            </div>
                        </div>

                        <div class="save-area">
                            <button id="save_btn" class="save-btn">Save Config</button>
                            <div id="status_msg" class="save-status">SAVED</div>
                        </div>
                    </div>

                    <div class="content-area layout-3" id="grid_container">
                        <div class="feed" id="feed_detect">
                            <div class="feed-title">DETECT (FINAL)</div>
                            <img src="/stream/circles" data-stream-url="/stream/circles" alt="Circles">
                        </div>

                        <div class="feed" id="feed_mask">
                            <div class="feed-title">MASK</div>
                            <img src="/stream/mask" data-stream-url="/stream/mask" alt="Mask">
                        </div>

                        <div class="feed" id="feed_contours">
                            <div class="feed-title">CONTOURS</div>
                            <img src="/stream/contours" data-stream-url="/stream/contours" alt="Contours">
                        </div>
                    </div>
                </div>
            </div>

            <script>
                const val = (id) => parseFloat(document.getElementById(id).value) || 0;
                const mainArea = document.getElementById('main_area');
                const toggleBtn = document.getElementById('toggle_btn');
                const gridContainer = document.getElementById('grid_container');

                // Sidebar Toggle
                toggleBtn.addEventListener('click', () => {
                    mainArea.classList.toggle('controls-hidden');
                });

                // --- Layout Logic ---

                function updateLayout() {
                    const feeds = ['detect', 'mask', 'contours'];
                    let activeCount = 0;

                    // 1. Calculate Active Streams
                    feeds.forEach(type => {
                        const isChecked = document.getElementById('chk_' + type).checked;
                        if(isChecked) activeCount++;
                    });

                    // 2. Apply Grid Class to Container
                    gridContainer.className = 'content-area layout-' + activeCount;
                }

                function toggleStream(type) {
                    const checkbox = document.getElementById('chk_' + type);
                    const container = document.getElementById('feed_' + type);
                    const img = container.querySelector('img');

                    if (checkbox.checked) {
                        // Enable:
                        // 1. Make visible in DOM
                        container.classList.remove('hidden');

                        // 2. Restore Source (reconnects socket/mjpeg)
                        if (!img.src) {
                            img.src = img.getAttribute('data-stream-url');
                        }
                    } else {
                        // Disable:
                        // 1. Remove Source (This is crucial for saving bandwidth)
                        img.removeAttribute('src');

                        // 2. Remove from DOM layout entirely
                        container.classList.add('hidden');
                    }

                    // Recalculate grid sizing
                    updateLayout();
                }

                // --- Config Logic ---

                async function loadConfig() {
                    try {
                        const res = await fetch('/config');
                        const cfg = await res.json();
                        // Mapping fields...
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
                    btn.innerHTML = "...";
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
