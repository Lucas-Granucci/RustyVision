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

                .header-bar {
                    padding: 15px 20px;
                    border-bottom: 2px solid #000;
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
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

                .toggle-controls:hover {
                    background: #333;
                }

                .main {
                    flex: 1;
                    display: grid;
                    grid-template-columns: 320px 1fr;
                    transition: grid-template-columns 0.3s;
                }

                .main.controls-hidden {
                    grid-template-columns: 0 1fr;
                }

                .sidebar {
                    border-right: 2px solid #000;
                    overflow: hidden;
                    display: flex;
                    flex-direction: column;
                }

                .controls-area {
                    flex: 1;
                    overflow-y: auto;
                    padding: 20px;
                }

                .section {
                    margin-bottom: 30px;
                }

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

                .field {
                    display: flex;
                    flex-direction: column;
                }

                .field-label {
                    font-size: 0.65rem;
                    margin-bottom: 4px;
                    text-transform: uppercase;
                }

                .field input {
                    border: 1px solid #000;
                    background: #fff;
                    padding: 6px 8px;
                    width: 100%;
                    font-family: 'Space Mono', monospace;
                    font-size: 0.8rem;
                }

                .field input:focus {
                    outline: 2px solid #000;
                    outline-offset: -2px;
                }

                .save-area {
                    padding: 12px 20px 14px;
                    border-top: 2px solid #000;
                    display: flex;
                    flex-direction: column;
                    gap: 8px;
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

                .save-btn:active {
                    background: #333;
                }

                .save-status {
                    margin-top: 0;
                    padding: 8px;
                    background: #000;
                    color: #fff;
                    text-align: center;
                    font-size: 0.7rem;
                    opacity: 0;
                    transition: opacity 0.3s;
                }

                .save-status.visible {
                    opacity: 1;
                }

                .content-area {
                    display: grid;
                    grid-template-columns: repeat(2, 1fr);
                    grid-template-rows: repeat(2, 1fr);
                    gap: 2px;
                    background: #000;
                    padding: 2px;
                }

                .feed {
                    background: #fff;
                    position: relative;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                }

                .feed-title {
                    position: absolute;
                    top: 10px;
                    left: 10px;
                    font-size: 0.65rem;
                    font-weight: 700;
                    text-transform: uppercase;
                    background: #fff;
                    padding: 4px 8px;
                    border: 1px solid #000;
                    z-index: 10;
                }

                .feed img {
                    width: 100%;
                    height: 100%;
                    object-fit: contain;
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
                            <button id="save_btn" class="save-btn">Save</button>
                            <div id="status_msg" class="save-status">SAVED</div>
                        </div>
                    </div>

                    <div class="content-area">
                        <div class="feed">
                            <div class="feed-title">RAW</div>
                            <img src="/stream/raw" alt="Raw">
                        </div>
                        <div class="feed">
                            <div class="feed-title">MASK</div>
                            <img src="/stream/mask" alt="Mask">
                        </div>
                        <div class="feed">
                            <div class="feed-title">CONTOURS</div>
                            <img src="/stream/contours" alt="Contours">
                        </div>
                        <div class="feed">
                            <div class="feed-title">DETECT</div>
                            <img src="/stream/circles" alt="Circles">
                        </div>
                    </div>
                </div>
            </div>

            <script>
                const val = (id) => parseFloat(document.getElementById(id).value) || 0;
                const mainArea = document.getElementById('main_area');
                const toggleBtn = document.getElementById('toggle_btn');

                toggleBtn.addEventListener('click', () => {
                    mainArea.classList.toggle('controls-hidden');
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
