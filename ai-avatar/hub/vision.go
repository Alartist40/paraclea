package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"sync"
	"time"

	"github.com/tarm/serial"
)

// VisionFrame is one JSON line from the ESP32 serial stream.
type VisionFrame struct {
	Timestamp    int64       `json:"timestamp"`
	FrameNumber  int         `json:"frame_number"`
	ESP32FPS     float64     `json:"esp32_fps"`
	GrovePresent bool        `json:"grove_present"`
	Detections   []Detection `json:"detections"`
}

// Detection represents a single object detected by Groove Vision AI.
type Detection struct {
	Label  string  `json:"label"`
	Score  float64 `json:"score"`
	X      int     `json:"x"`
	Y      int     `json:"y"`
	Width  int     `json:"width"`
	Height int     `json:"height"`
}

// VisionService reads JSON frames from the Groove Vision AI serial port.
type VisionService struct {
	portPath string
	port     *serial.Port
	latest   VisionFrame
	running  bool
	mu       sync.RWMutex
	stopCh   chan struct{}
}

// NewVisionService creates a vision reader for the given serial device.
func NewVisionService(portPath string) *VisionService {
	if portPath == "" {
		portPath = "/dev/ttyACM0"
	}
	return &VisionService{
		portPath: portPath,
		stopCh:   make(chan struct{}),
	}
}

// Start begins reading frames, retrying if the serial port is not yet available.
func (v *VisionService) Start() error {
	v.running = true
	go v.connectLoop()
	return nil
}

// Stop closes the serial port and ends the read loop.
func (v *VisionService) Stop() {
	v.mu.Lock()
	defer v.mu.Unlock()
	if !v.running {
		return
	}
	v.running = false
	close(v.stopCh)
	if v.port != nil {
		v.port.Close()
	}
	log.Println("[vision] stopped")
}

// LatestFrame returns the most recent vision frame.
func (v *VisionService) LatestFrame() VisionFrame {
	v.mu.RLock()
	defer v.mu.RUnlock()
	return v.latest
}

// IsRunning reports whether the service is active.
func (v *VisionService) IsRunning() bool {
	v.mu.RLock()
	defer v.mu.RUnlock()
	return v.running
}

func (v *VisionService) connectLoop() {
	for {
		select {
		case <-v.stopCh:
			return
		default:
		}

		if err := v.tryConnect(); err != nil {
			log.Printf("[vision] %v — retrying in 3s", err)
			select {
			case <-v.stopCh:
				return
			case <-time.After(3 * time.Second):
				continue
			}
		}

		// Connected — read until error or stop.
		v.readLoop()

		// Clean up port before retry.
		v.mu.Lock()
		if v.port != nil {
			v.port.Close()
			v.port = nil
		}
		v.mu.Unlock()
	}
}

func (v *VisionService) tryConnect() error {
	cfg := &serial.Config{
		Name:        v.portPath,
		Baud:        9600,
		ReadTimeout: time.Second * 2,
	}
	port, err := serial.OpenPort(cfg)
	if err != nil {
		return fmt.Errorf("open serial %s: %w", v.portPath, err)
	}
	v.port = port
	log.Printf("[vision] connected to %s", v.portPath)
	return nil
}

func (v *VisionService) readLoop() {
	reader := bufio.NewReader(v.port)
	for {
		select {
		case <-v.stopCh:
			return
		default:
		}

		line, err := reader.ReadBytes('\n')
		if err != nil {
			if os.IsTimeout(err) {
				continue
			}
			log.Printf("[vision] serial read error: %v", err)
			return
		}

		var frame VisionFrame
		if err := json.Unmarshal(line, &frame); err != nil {
			// Some lines might be partial; skip them.
			continue
		}

		v.mu.Lock()
		v.latest = frame
		v.mu.Unlock()
	}
}
