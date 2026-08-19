#!/bin/bash
# Piper fine-tuning launcher
# Run this on a machine with GPU for reasonable training times.

set -e

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DATASET_DIR="$PROJECT_ROOT/dataset/training_data"
CHECKPOINT_DIR="$PROJECT_ROOT/dataset/checkpoints"

# Base model to fine-tune from
BASE_MODEL="en_US-lessac-medium"

# Training parameters
BATCH_SIZE=32
LEARNING_RATE=0.0001
EPOCHS=5000

mkdir -p "$CHECKPOINT_DIR"

echo "=========================================="
echo "Piper Voice Training Pipeline"
echo "=========================================="
echo "Dataset: $DATASET_DIR"
echo "Checkpoints: $CHECKPOINT_DIR"
echo "Base model: $BASE_MODEL"
echo "Epochs: $EPOCHS"
echo ""

# Check if piper-tts python package is available
if ! python3 -c "import piper" 2>/dev/null; then
    echo "ERROR: piper-tts Python package not installed."
    echo "Install: pip install piper-tts"
    exit 1
fi

# Check dataset
if [ ! -f "$DATASET_DIR/metadata.csv" ]; then
    echo "ERROR: metadata.csv not found. Run 01_segment_audiobook.py, 02_transcribe.py, 03_build_metadata.py first."
    exit 1
fi

LINE_COUNT=$(wc -l < "$DATASET_DIR/metadata.csv")
echo "Dataset size: $LINE_COUNT clips"

# Preprocess dataset into Piper format
echo ""
echo "Step 1: Preprocessing dataset..."
python3 -m piper_train.preprocess \
    --dataset-dir "$DATASET_DIR" \
    --dataset-format ljspeech \
    --sample-rate 22050 \
    --max-workers 4

# Start training
echo ""
echo "Step 2: Starting training..."
echo "This will take several hours on GPU, or days on CPU."
echo "Press Ctrl+C to pause (checkpoint is saved every 100 epochs)."
echo ""

python3 -m piper_train \
    --dataset-dir "$DATASET_DIR" \
    --accelerator gpu \
    --batch-size "$BATCH_SIZE" \
    --validation-split 0.05 \
    --num-test-examples 5 \
    --max-epochs "$EPOCHS" \
    --checkpoint-epochs 100 \
    --checkpoint-dir "$CHECKPOINT_DIR" \
    --resume_from_single_speaker_checkpoint "$PROJECT_ROOT/models/${BASE_MODEL}.ckpt" \
    --optimizer adamw \
    --lr "$LEARNING_RATE"

echo ""
echo "Training complete! Checkpoints saved to: $CHECKPOINT_DIR"
echo ""
echo "To export the best checkpoint to ONNX:"
echo "  python3 -m piper_train.export_onnx \\"
echo "      --checkpoint $CHECKPOINT_DIR/epoch=XXXX-step=YYYYY.ckpt \\"
echo "      --output $PROJECT_ROOT/models/paraclea_custom.onnx"
