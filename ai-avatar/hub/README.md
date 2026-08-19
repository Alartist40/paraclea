# AI Avatar Hub

Go-based orchestrator for the Paraclea AI avatar. Replaces OmniBot's Python
FastAPI backend with a minimal-dependency Go service.

## Architecture

```
                    WebSocket / REST
    ┌─────────────┐ ◄──────────────► ┌─────┐
    │  Dashboard  │                  │ Hub │
    │  (React)    │                  │(Go) │
    └─────────────┘                  └──┬──┘
         ▲                              │
         │ file poll                    │ serial
         └──────────────────────────────┘
              ┌─────────────┐      ┌──────────────┐
              │ Local Vision│      │ Groove Vision│
              │ (Rust+ONNX) │      │ AI (ESP32)   │
              └─────────────┘      └──────────────┘
```

## Features

- **REST API** — State, vision, birdwatch, persona, commands
- **WebSocket** — Real-time pipeline state broadcast at 10 Hz
- **Persona Framework** — Markdown files per bot (SOUL, IDENTITY, MEMORY, etc.)
- **Vision Integration**
  - **ESP32 Serial** — Groove Vision AI at `/dev/ttyACM0` (9600 baud JSON)
  - **Local Vision** — Rust OpenCV DNN running YOLOv5 + Haar face detection
- **Birdwatching** — Multi-frame centroid tracking + SQLite logging
- **Graceful Degradation** — Services retry automatically if hardware disconnects

## Build

```bash
cd hub/
go build -o hub .
```

## Run

```bash
./hub
# Listening on http://0.0.0.0:8080
```

## Local Vision (Rust)

The Rust vision binary runs independently and writes to `/tmp/local_detections.json`.

```bash
cd ../vision
cargo run
# Or for release build:
cargo build --release
./target/release/ai-avatar-vision
```

### Requirements
- OpenCV 4.x (system package)
- YOLOv5 ONNX model at `../models/yolov5s.onnx`
- Haar cascade at `/usr/share/opencv4/haarcascades/haarcascade_frontalface_default.xml`

### Capabilities
- **Object Detection** — 80 COCO classes via YOLOv5s (OpenCV DNN)
- **Face Detection** — Haar Cascade frontal face detector
- **Fallback Mode** — If no webcam available, writes empty frames every second

## Dashboard

The React dashboard is served statically from `../dashboard/dist`. Build it first:

```bash
cd ../dashboard
npm install
npm run build
```

## API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/state` | GET | Full pipeline state (emotion, vision, birds) |
| `/api/vision` | GET | Latest ESP32 vision frame |
| `/api/local-vision` | GET | Latest local webcam/YOLO frame |
| `/api/birdwatch/tracks` | GET | Active bird tracks |
| `/api/birdwatch/sightings` | GET | Recent sightings (query: `limit`) |
| `/api/birdwatch/species` | GET | Unique species list |
| `/api/persona?bot=default` | GET | All persona markdown files |
| `/api/persona` | POST | Update persona files (`{bot_id, files}`) |
| `/api/command` | POST | Send command (`{command}`) |
| `/ws` | WS | Real-time state stream |

## Vision Protocols

### ESP32 Serial (Groove Vision AI)
Newline-delimited JSON at 9600 baud:
```json
{"timestamp":1884231,"frame_number":191,"esp32_fps":15.15,"grove_present":true,"detections":[]}
```

### Local Vision File (`/tmp/local_detections.json`)
Atomically-written JSON by Rust vision engine:
```json
{"timestamp":"2026-05-24T20:36:40Z","frame_number":42,"detections":[
  {"class_id":14,"class_name":"bird","confidence":0.87,"bbox":[100,200,50,30]}
],"fps":15.0,"inference_time_ms":45,"source":"local"}
```

## Persona Files

Stored in `data/persona/<botID>/`:

- `SOUL.md` — Personality, tone, quirks
- `IDENTITY.md` — Name, nature, origin
- `USER.md` — Human user profile
- `MEMORY.md` — Long-term memory
- `TOOLS.md` — Available capabilities
- `HEARTBEAT.md` — Maintenance instructions
- `AGENTS.md` — Behavior guides
