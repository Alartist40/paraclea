use opencv::{
    prelude::*,
    core,
    imgproc,
    objdetect,
};
use crate::yolo::Detection;

pub struct FaceDetector {
    cascade: objdetect::CascadeClassifier,
}

impl FaceDetector {
    pub fn new(cascade_path: &str) -> anyhow::Result<Self> {
        let mut cascade = objdetect::CascadeClassifier::default()?;
        if !cascade.load(cascade_path)? {
            return Err(anyhow::anyhow!("Failed to load face cascade XML from {}", cascade_path));
        }
        eprintln!("Face cascade loaded: {}", cascade_path);
        Ok(FaceDetector { cascade })
    }

    pub fn detect(&mut self, frame: &Mat) -> anyhow::Result<Vec<Detection>> {
        let mut gray = Mat::default();
        imgproc::cvt_color(frame, &mut gray, imgproc::COLOR_BGR2GRAY, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT)?;

        let mut faces = core::Vector::<core::Rect>::new();
        self.cascade.detect_multi_scale(
            &gray,
            &mut faces,
            1.1,
            3,
            objdetect::CASCADE_SCALE_IMAGE,
            core::Size::new(30, 30),
            core::Size::new(0, 0),
        )?;

        let mut detections = Vec::new();
        for face in faces {
            detections.push(Detection {
                class_id: -1,
                class_name: "face".to_string(),
                confidence: 1.0,
                bbox: [face.x, face.y, face.width, face.height],
            });
        }

        Ok(detections)
    }
}
