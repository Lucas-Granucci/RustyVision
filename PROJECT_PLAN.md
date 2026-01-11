# FRC Vision System - Project Plan

## Project Overview

A high-performance object detection vision system for FIRST Robotics Competition, written in Rust, designed to run on a coprocessor (Raspberry Pi, Jetson Nano, etc.) and communicate with the robot's main controller via NetworkTables.

## Final Product Vision

### Core Capabilities
- Real-time object detection for FRC game pieces (AprilTags, notes, cones, cubes, etc.)
- Sub-50ms latency from camera to NetworkTables
- Multi-camera support (up to 4 cameras)
- Automatic camera calibration and distortion correction
- Robust performance in varying lighting conditions
- Web-based configuration dashboard
- Live video streaming for driver station
- Detailed telemetry and performance metrics

### Target Performance
- **Frame Rate**: 30+ FPS per camera
- **Detection Latency**: <50ms end-to-end
- **CPU Usage**: <70% on Raspberry Pi 4
- **Memory**: <500MB RAM usage
- **Startup Time**: <5 seconds to first detection

## Feature Set

### Phase 1: MVP (Minimum Viable Product)
- [x] Single camera capture
- [x] AprilTag detection
- [x] Basic NetworkTables publishing (x, y, distance, angle)
- [x] Simple CLI configuration
- [x] Basic error handling and logging

### Phase 2: Enhanced Detection
- [ ] Multi-camera support
- [ ] Color-based object detection (game pieces)
- [ ] Camera calibration system
- [ ] 3D pose estimation
- [ ] Target filtering and tracking
- [ ] Confidence scoring

### Phase 3: Robustness & Performance
- [ ] Multi-threaded pipeline (capture, process, publish)
- [ ] Adaptive exposure control
- [ ] Performance monitoring and auto-tuning
- [ ] Graceful degradation on overload
- [ ] Reconnection handling for NetworkTables
- [ ] Watchdog and health monitoring

### Phase 4: User Experience
- [ ] Web-based dashboard (configuration UI)
- [ ] Live video streaming to dashboard
- [ ] Real-time telemetry visualization
- [ ] Camera calibration wizard
- [ ] Detection zone configuration (ROI)
- [ ] Pre-built detection profiles per game

### Phase 5: Advanced Features
- [ ] Machine learning model support (ONNX)
- [ ] Multi-target tracking with Kalman filtering
- [ ] Automatic game piece type classification
- [ ] Recording and replay for analysis
- [ ] Integration with PathPlanner/Choreo
- [ ] Field-relative coordinate transformation

## Repository Structure

```
frc-vision-rs/
├── Cargo.toml                 # Workspace root
├── README.md
├── LICENSE
├── .gitignore
├── rustfmt.toml
├── Cross.toml                 # Cross-compilation config
│
├── crates/
│   ├── vision-core/           # Main application binary
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── config.rs      # Configuration management
│   │   │   ├── camera.rs      # Camera interface
│   │   │   ├── pipeline.rs    # Processing pipeline
│   │   │   └── telemetry.rs   # Metrics collection
│   │   └── tests/
│   │
│   ├── vision-detection/      # Detection algorithms library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── apriltag.rs    # AprilTag detection
│   │   │   ├── color.rs       # Color-based detection
│   │   │   ├── ml.rs          # ML model inference
│   │   │   └── tracking.rs    # Target tracking
│   │   └── tests/
│   │
│   ├── vision-nt/             # NetworkTables client library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs      # NT4 client
│   │   │   ├── publisher.rs   # Data publishing
│   │   │   └── types.rs       # Custom NT types
│   │   └── tests/
│   │
│   ├── vision-calibration/    # Camera calibration tools
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── intrinsic.rs   # Intrinsic calibration
│   │   │   └── pose.rs        # Pose estimation
│   │   └── tests/
│   │
│   └── vision-web/            # Web dashboard (optional)
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── server.rs      # HTTP server
│       │   └── api.rs         # REST API
│       └── static/            # HTML/CSS/JS files
│
├── config/                    # Configuration files
│   ├── default.toml
│   ├── camera-profiles/
│   └── detection-profiles/
│
├── scripts/                   # Utility scripts
│   ├── build-pi.sh           # Cross-compile for Pi
│   ├── deploy.sh             # Deploy to coprocessor
│   └── calibrate.sh          # Run calibration
│
├── docs/                      # Documentation
│   ├── ARCHITECTURE.md
│   ├── QUICKSTART.md
│   ├── CALIBRATION.md
│   └── API.md
│
└── examples/                  # Example code
    ├── simple-detection.rs
    ├── multi-camera.rs
    └── custom-pipeline.rs
```

## Technology Stack

### Core Dependencies
- **opencv** or **image-rs**: Camera capture and image processing
- **apriltag** or **apriltag-rs**: AprilTag detection
- **network-tables**: NetworkTables 4 protocol client
- **tokio**: Async runtime for networking
- **serde**: Configuration serialization
- **tracing**: Structured logging
- **axum**: Web server (for dashboard)

### Optional Dependencies
- **tract** or **onnxruntime**: ML model inference
- **nalgebra**: Linear algebra for pose estimation
- **rerun**: Visualization and debugging

## Development Roadmap

### Stage 1: Foundation (Week 1-2)
**Goal**: Get a single camera working with AprilTag detection

1. Set up Rust project structure
2. Implement basic camera capture (start with webcam)
3. Integrate AprilTag detection library
4. Display detected tags with bounding boxes
5. Calculate basic metrics (distance, angle)

**Success Criteria**: 
- Can detect AprilTags in real-time on dev machine
- Prints detection data to console
- Runs at 20+ FPS

### Stage 2: NetworkTables Integration (Week 3)
**Goal**: Publish detection data to NetworkTables

1. Add NetworkTables client library
2. Connect to robot NetworkTables server
3. Publish detection data (ID, x, y, distance, angle)
4. Add configuration file support (camera settings, NT server IP)
5. Implement basic error handling

**Success Criteria**:
- Data appears in NetworkTables on robot code
- Reconnects automatically if connection drops
- Can configure via TOML file

### Stage 3: Multi-Camera Support (Week 4)
**Goal**: Support multiple cameras simultaneously

1. Refactor camera capture to support multiple instances
2. Add per-camera configuration
3. Publish data from each camera to separate NT topics
4. Implement basic camera identification/naming

**Success Criteria**:
- Can run 2+ cameras simultaneously
- Each camera maintains 20+ FPS
- Data from each camera is clearly identified

### Stage 4: Performance Optimization (Week 5-6)
**Goal**: Achieve target performance metrics

1. Profile the application to identify bottlenecks
2. Implement multi-threaded pipeline (capture → process → publish)
3. Add frame dropping when overloaded
4. Optimize memory allocations
5. Add performance metrics logging

**Success Criteria**:
- Achieves <50ms latency
- Maintains 30+ FPS under load
- CPU usage <70% on target hardware

### Stage 5: Web Dashboard (Week 7-8)
**Goal**: Add user-friendly configuration interface

1. Create basic web server
2. Implement live video streaming
3. Add configuration UI (camera settings, detection params)
4. Display real-time telemetry
5. Add calibration wizard

**Success Criteria**:
- Can configure all settings via web interface
- Live video stream viewable in browser
- Telemetry updates in real-time

### Stage 6: Advanced Detection (Week 9-10)
**Goal**: Add game-piece detection

1. Implement color-based detection (HSV filtering)
2. Add contour detection and filtering
3. Implement shape recognition
4. Add detection confidence scoring
5. Create detection profiles for different game pieces

**Success Criteria**:
- Can reliably detect game pieces in various lighting
- False positive rate <5%
- Works with different game piece types



## Testing Strategy

### Unit Tests
- Test each detection algorithm independently
- Mock camera input with test images
- Verify NetworkTables message formatting

### Integration Tests
- End-to-end pipeline with recorded video
- NetworkTables communication with mock server
- Multi-camera coordination

### Field Testing
- Test on actual robot at varying distances
- Different lighting conditions (outdoor, arena)
- Test with moving robot
- Verify latency under competition conditions

## Deployment Process

1. Cross-compile for target architecture (ARM for Pi)
2. Copy binary and config to coprocessor
3. Set up systemd service for auto-start
4. Configure network settings
5. Run calibration routine
6. Verify NetworkTables connection
7. Test detection with real game pieces

## Success Metrics

- **Reliability**: Runs for entire match (2.5 min) without crashes
- **Accuracy**: >95% detection rate for visible targets
- **Latency**: <50ms from capture to NetworkTables
- **Uptime**: Boots and connects within 30 seconds of power-on
- **Usability**: Non-programmers can configure via web UI

## Resources

- Rust Book: https://doc.rust-lang.org/book/
- FRC NetworkTables: https://docs.wpilib.org/en/stable/docs/software/networktables/
- AprilTag specification: https://april.eecs.umich.edu/software/apriltag
- OpenCV tutorials: https://docs.opencv.org/4.x/d9/df8/tutorial_root.html
---
