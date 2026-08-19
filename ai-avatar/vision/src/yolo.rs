use opencv::{
    prelude::*,
    core,
    dnn,
    imgproc,
};
use serde::Serialize;

const CONF_THRESH: f32 = 0.5;
const NMS_THRESH: f32 = 0.45;
const INPUT_SIZE: i32 = 640;

#[derive(Serialize, Debug, Clone)]
pub struct Detection {
    pub class_id: i32,
    pub class_name: String,
    pub confidence: f32,
    pub bbox: [i32; 4], // [x, y, width, height]
}

pub struct YoloDetector {
    net: dnn::Net,
    class_names: Vec<String>,
}

impl YoloDetector {
    pub fn new(model_path: &str) -> anyhow::Result<Self> {
        let mut net = dnn::read_net_from_onnx(model_path)?;
        net.set_preferable_target(dnn::DNN_TARGET_CPU)?;
        net.set_preferable_backend(dnn::DNN_BACKEND_OPENCV)?;

        let class_names = vec![
            "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck", "boat", "traffic light",
            "fire hydrant", "stop sign", "parking meter", "bench", "bird", "cat", "dog", "horse", "sheep", "cow",
            "elephant", "bear", "zebra", "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee",
            "skis", "snowboard", "sports ball", "kite", "baseball bat", "baseball glove", "skateboard", "surfboard", "tennis racket", "bottle",
            "wine glass", "cup", "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange",
            "broccoli", "carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch", "potted plant", "bed",
            "dining table", "toilet", "tv", "laptop", "mouse", "remote", "keyboard", "cell phone", "microwave", "oven",
            "toaster", "sink", "refrigerator", "book", "clock", "vase", "scissors", "teddy bear", "hair drier", "toothbrush"
        ].into_iter().map(|s| s.to_string()).collect();

        eprintln!("YOLOv5 model loaded: {}", model_path);
        Ok(YoloDetector { net, class_names })
    }

    pub fn detect(&mut self, frame: &Mat) -> anyhow::Result<Vec<Detection>> {
        let (img_height, img_width) = (frame.rows(), frame.cols());

        let blob = dnn::blob_from_image(
            frame,
            1.0 / 255.0,
            core::Size::new(INPUT_SIZE, INPUT_SIZE),
            core::Scalar::new(0.0, 0.0, 0.0, 0.0),
            true,
            false,
            core::CV_32F,
        )?;

        self.net.set_input(&blob, "", 1.0, core::Scalar::default())?;

        let mut outputs = core::Vector::<Mat>::new();
        let out_names = self.net.get_unconnected_out_layers_names()?;
        self.net.forward(&mut outputs, &out_names)?;

        if outputs.is_empty() {
            return Ok(Vec::new());
        }

        let output = outputs.get(0)?;
        let rows = output.rows();
        let cols = output.cols();

        let mut boxes = core::Vector::<core::Rect>::new();
        let mut confidences = core::Vector::<f32>::new();
        let mut class_ids = core::Vector::<i32>::new();

        for i in 0..rows {
            let row = output.row(i)?;
            let obj_conf = *row.at_2d::<f32>(0, 4)?;

            if obj_conf < CONF_THRESH {
                continue;
            }

            let mut max_cls_conf = 0.0f32;
            let mut max_cls_id = 0i32;
            for j in 5..cols {
                let cls_conf = *row.at_2d::<f32>(0, j)?;
                if cls_conf > max_cls_conf {
                    max_cls_conf = cls_conf;
                    max_cls_id = (j - 5) as i32;
                }
            }

            let score = obj_conf * max_cls_conf;
            if score < CONF_THRESH {
                continue;
            }

            let cx = *row.at_2d::<f32>(0, 0)?;
            let cy = *row.at_2d::<f32>(0, 1)?;
            let w = *row.at_2d::<f32>(0, 2)?;
            let h = *row.at_2d::<f32>(0, 3)?;

            let x_ratio = img_width as f32 / INPUT_SIZE as f32;
            let y_ratio = img_height as f32 / INPUT_SIZE as f32;

            let x = (cx - w / 2.0) * x_ratio;
            let y = (cy - h / 2.0) * y_ratio;
            let w = w * x_ratio;
            let h = h * y_ratio;

            let rect = core::Rect::new(
                x.max(0.0) as i32,
                y.max(0.0) as i32,
                w.max(0.0) as i32,
                h.max(0.0) as i32,
            );

            boxes.push(rect);
            confidences.push(score);
            class_ids.push(max_cls_id);
        }

        let mut indices = core::Vector::<i32>::new();
        dnn::nms_boxes(&boxes, &confidences, CONF_THRESH, NMS_THRESH, &mut indices, 1.0, 0)?;

        let mut detections = Vec::new();
        for idx in indices {
            let idx = idx as usize;
            let class_id = class_ids.get(idx)?.clone();
            let class_name = self.class_names.get(class_id as usize)
                .map(|s| s.as_str())
                .unwrap_or("unknown");

            let rect = boxes.get(idx)?;
            detections.push(Detection {
                class_id,
                class_name: class_name.to_string(),
                confidence: confidences.get(idx)?.clone(),
                bbox: [rect.x, rect.y, rect.width, rect.height],
            });
        }

        Ok(detections)
    }
}
