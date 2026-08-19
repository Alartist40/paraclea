package main

import (
	"encoding/json"
	"log"
	"sync"
	"time"
)

// Hub is the central orchestrator.
type Hub struct {
	config Config

	// Subscribers receive real-time events
	clients map[*Client]bool
	mu      sync.RWMutex

	// Pipeline state (mirrored from Rust core)
	state PipelineState

	// Persona manager
	persona *PersonaManager

	// Vision service (Groove Vision AI via ESP32 serial)
	vision *VisionService

	// Local vision service (Rust OpenCV DNN via file polling)
	localVision *LocalVisionService

	// Birdwatch service
	birdwatch *BirdWatchService
}

// PipelineState mirrors the Rust PipelineState for the dashboard.
type PipelineState struct {
	IsSpeaking       bool            `json:"is_speaking"`
	LastUserText     string          `json:"last_user_text"`
	LastAiText       string          `json:"last_ai_text"`
	Emotion          Emotion         `json:"emotion"`
	VisionDetections []Detection     `json:"vision_detections,omitempty"`
	VisionFPS        float64         `json:"vision_fps,omitempty"`
	GrovePresent     bool            `json:"grove_present,omitempty"`
	LocalDetections  []LocalDetection `json:"local_detections,omitempty"`
	LocalFPS         float32         `json:"local_fps,omitempty"`
	BirdTracks       []BirdTrack     `json:"bird_tracks,omitempty"`
}

type Emotion struct {
	Joy        float64 `json:"joy"`
	Sadness    float64 `json:"sadness"`
	Anger      float64 `json:"anger"`
	Fear       float64 `json:"fear"`
	Surprise   float64 `json:"surprise"`
	Trust      float64 `json:"trust"`
	MouthSmile float64 `json:"mouth_smile"`
}

type Client struct {
	hub  *Hub
	send chan []byte
}

func NewHub(cfg Config) *Hub {
	return &Hub{
		config:      cfg,
		clients:     make(map[*Client]bool),
		state:       PipelineState{},
		persona:     NewPersonaManager(cfg.DataDir),
		vision:      NewVisionService(""),
		localVision: NewLocalVisionService(""),
	}
}

// Run starts background goroutines.
func (h *Hub) Run() {
	if err := h.vision.Start(); err != nil {
		log.Printf("[hub] vision service failed to start: %v", err)
	}
	if err := h.localVision.Start(); err != nil {
		log.Printf("[hub] local vision service failed to start: %v", err)
	}
	if h.birdwatch != nil {
		if err := h.birdwatch.Start(); err != nil {
			log.Printf("[hub] birdwatch service failed to start: %v", err)
		}
	}
	log.Println("Hub running")
}

// Stop shuts down all services.
func (h *Hub) Stop() {
	h.vision.Stop()
	h.localVision.Stop()
	if h.birdwatch != nil {
		h.birdwatch.Stop()
	}
}

// Broadcast sends a message to all connected WebSocket clients.
func (h *Hub) Broadcast(msg []byte) {
	h.mu.RLock()
	defer h.mu.RUnlock()
	for client := range h.clients {
		select {
		case client.send <- msg:
		default:
		}
	}
}

func (h *Hub) RegisterClient(c *Client) {
	h.mu.Lock()
	h.clients[c] = true
	h.mu.Unlock()
	log.Println("Client connected")
}

func (h *Hub) UnregisterClient(c *Client) {
	h.mu.Lock()
	delete(h.clients, c)
	h.mu.Unlock()
	close(c.send)
	log.Println("Client disconnected")
}

// SyncVisionState copies the latest vision frames into pipeline state.
func (h *Hub) SyncVisionState() {
	// ESP32 vision
	frame := h.vision.LatestFrame()
	h.mu.Lock()
	h.state.VisionDetections = frame.Detections
	h.state.VisionFPS = frame.ESP32FPS
	h.state.GrovePresent = frame.GrovePresent
	h.mu.Unlock()

	// Local vision
	localFrame := h.localVision.LatestFrame()
	h.mu.Lock()
	h.state.LocalDetections = localFrame.Detections
	h.state.LocalFPS = localFrame.FPS
	h.mu.Unlock()

	// Birdwatch tracking
	if h.birdwatch != nil {
		tracks := h.birdwatch.ProcessDetections(localFrame.Detections)
		h.mu.Lock()
		h.state.BirdTracks = tracks
		h.mu.Unlock()
	}
}

// StartBroadcastLoop sends pipeline state to all clients periodically.
func (h *Hub) StartBroadcastLoop() {
	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()
	for range ticker.C {
		h.SyncVisionState()

		h.mu.RLock()
		state := h.state
		h.mu.RUnlock()

		data, err := json.Marshal(map[string]interface{}{
			"type": "state",
			"data": state,
		})
		if err != nil {
			continue
		}
		h.Broadcast(data)
	}
}
