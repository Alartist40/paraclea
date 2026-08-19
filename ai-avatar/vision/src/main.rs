mod yolo;
mod face;

use opencv::{
    prelude::*,
    videoio,
    imgcodecs,
    core,
};
use std::time::Instant;
use chrono::Local;
use yolo::{YoloDetector, Detection};
use face::FaceDetector;

const OUTPUT_IMAGE_PATH: &str = "/tmp/local_vision_feed.jpg";
const OUTPUT_JSON_PATH: &str = "/tmp/local_detections.json";
const TARGET_FPS: f64 = 15.0;

#[derive(serde::Serialize)]
struct DetectionFrame {
    timestamp: String,
    frame_number: u64,
    detections: Vec<Detection>,
    fps: f32,
    inference_time_ms: u64,
    source: String,
}

fn main() -> anyhow::Result<()> {
    eprintln!("🚀 Starting AI Avatar Local Vision Engine...");

    let model_path = std::env::var("YOLO_MODEL")
        .unwrap_or_else(|_| "../models/yolov5s.onnx".to_string());
    let cascade_path = std::env::var("FACE_CASCADE")
        .unwrap_or_else(|_| "/usr/share/opencv4/haarcascades/haarcascade_frontalface_default.xml".to_string());

    let mut yolo = YoloDetector::new(&model_path)?;
    let mut face_detector = FaceDetector::new(&cascade_path)?;

    // Try to open camera and verify it works
    let mut cam = match videoio::VideoCapture::new(0, videoio::CAP_ANY) {
        Ok(mut c) => {
            let _ = c.set(videoio::CAP_PROP_FRAME_WIDTH, 640.0);
            let _ = c.set(videoio::CAP_PROP_FRAME_HEIGHT, 480.0);
            // Test read
            let mut test_frame = Mat::default();
            if !c.read(&mut test_frame).unwrap_or(false) || test_frame.empty() {
                eprintln!("⚠️ Camera opened but cannot read frames");
                None
            } else {
                eprintln!("📷 Camera opened and verified");
                Some(c)
            }
        }
        Err(e) => {
            eprintln!("⚠️ No camera available: {}", e);
            None
        }
    };

    if cam.is_none() {
        eprintln!("📝 Running in stub mode (empty detections)");
        return run_stub_mode();
    }

    let mut cam = cam.unwrap();
    let frame_duration = std::time::Duration::from_millis((1000.0 / TARGET_FPS) as u64);
    let mut frame_count: u64 = 0;

    loop {
        let loop_start = Instant::now();
        let mut frame = Mat::default();

        if !cam.read(&mut frame)? || frame.empty() {
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }

        // Run YOLO object detection
        let mut all_detections = yolo.detect(&frame).unwrap_or_default();

        // Run face detection
        let face_dets = face_detector.detect(&frame).unwrap_or_default();
        all_detections.extend(face_dets);

        let detection_frame = DetectionFrame {
            timestamp: Local::now().to_rfc3339(),
            frame_number: frame_count,
            detections: all_detections,
            fps: TARGET_FPS as f32,
            inference_time_ms: loop_start.elapsed().as_millis() as u64,
            source: "local".to_string(),
        };

        let json_data = serde_json::to_string(&detection_frame)?;
        write_atomic(OUTPUT_JSON_PATH, &json_data)?;

        let _ = imgcodecs::imwrite(OUTPUT_IMAGE_PATH, &frame, &core::Vector::default());

        frame_count += 1;

        let elapsed = loop_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}

fn run_stub_mode() -> anyhow::Result<()> {
    let frame_duration = std::time::Duration::from_secs(1);
    let mut frame_count: u64 = 0;
    loop {
        let detection_frame = DetectionFrame {
            timestamp: Local::now().to_rfc3339(),
            frame_number: frame_count,
            detections: vec![],
            fps: 0.0,
            inference_time_ms: 0,
            source: "local-stub".to_string(),
        };
        let json_data = serde_json::to_string(&detection_frame)?;
        write_atomic(OUTPUT_JSON_PATH, &json_data)?;
        frame_count += 1;
        std::thread::sleep(frame_duration);
    }
}

fn write_atomic(path: &str, content: &str) -> std::io::Result<()> {
    let temp_path = format!("{}.tmp", path);
    std::fs::write(&temp_path, content)?;
    std::fs::rename(&temp_path, path)
}
