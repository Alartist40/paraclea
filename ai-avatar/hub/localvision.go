package main

import (
	"encoding/json"
	"log"
	"os"
	"sync"
	"time"
)

// LocalDetection matches the Rust vision JSON output.
type LocalDetection struct {
	ClassID    int32   `json:"class_id"`
	ClassName  string  `json:"class_name"`
	Confidence float32 `json:"confidence"`
	Bbox       [4]int  `json:"bbox"` // [x, y, width, height]
}

// LocalVisionFrame is the JSON written by ai-avatar-vision.
type LocalVisionFrame struct {
	Timestamp       string           `json:"timestamp"`
	FrameNumber     uint64           `json:"frame_number"`
	Detections      []LocalDetection `json:"detections"`
	FPS             float32          `json:"fps"`
	InferenceTimeMs uint64           `json:"inference_time_ms"`
	Source          string           `json:"source"`
}

// LocalVisionService polls the Rust vision output file.
type LocalVisionService struct {
	jsonPath string
	latest   LocalVisionFrame
	running  bool
	mu       sync.RWMutex
	stopCh   chan struct{}
}

func NewLocalVisionService(jsonPath string) *LocalVisionService {
	if jsonPath == "" {
		jsonPath = "/tmp/local_detections.json"
	}
	return &LocalVisionService{
		jsonPath: jsonPath,
		stopCh:   make(chan struct{}),
	}
}

func (l *LocalVisionService) Start() error {
	l.running = true
	go l.pollLoop()
	log.Printf("[local-vision] started polling %s", l.jsonPath)
	return nil
}

func (l *LocalVisionService) Stop() {
	l.mu.Lock()
	defer l.mu.Unlock()
	if !l.running {
		return
	}
	l.running = false
	close(l.stopCh)
	log.Println("[local-vision] stopped")
}

func (l *LocalVisionService) LatestFrame() LocalVisionFrame {
	l.mu.RLock()
	defer l.mu.RUnlock()
	return l.latest
}

func (l *LocalVisionService) IsRunning() bool {
	l.mu.RLock()
	defer l.mu.RUnlock()
	return l.running
}

func (l *LocalVisionService) pollLoop() {
	ticker := time.NewTicker(200 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-l.stopCh:
			return
		case <-ticker.C:
		}

		data, err := os.ReadFile(l.jsonPath)
		if err != nil {
			continue // File may not exist yet
		}

		var frame LocalVisionFrame
		if err := json.Unmarshal(data, &frame); err != nil {
			continue
		}

		l.mu.Lock()
		l.latest = frame
		l.mu.Unlock()
	}
}
