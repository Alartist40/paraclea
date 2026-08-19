package main

import (
	"encoding/json"
	"log"
	"net/http"
	"strconv"
)

// API handlers for the hub REST endpoints.

func (h *Hub) handleGetState(w http.ResponseWriter, r *http.Request) {
	h.mu.RLock()
	state := h.state
	h.mu.RUnlock()
	respondJSON(w, state)
}

func (h *Hub) handleGetPersona(w http.ResponseWriter, r *http.Request) {
	botID := r.URL.Query().Get("bot")
	if botID == "" {
		http.Error(w, "missing bot param", http.StatusBadRequest)
		return
	}
	files, err := h.persona.List(botID)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	respondJSON(w, files)
}

func (h *Hub) handleSetPersona(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	var req struct {
		BotID string            `json:"bot_id"`
		Files map[string]string `json:"files"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	if err := h.persona.ReplaceAll(req.BotID, req.Files); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	respondJSON(w, map[string]string{"status": "ok"})
}

func (h *Hub) handleSendCommand(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	var req struct {
		Command string `json:"command"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	log.Printf("Received command: %s", req.Command)
	// TODO: Forward to Rust core
	respondJSON(w, map[string]string{"status": "received", "command": req.Command})
}

func (h *Hub) handleGetVision(w http.ResponseWriter, r *http.Request) {
	frame := h.vision.LatestFrame()
	respondJSON(w, frame)
}

func (h *Hub) handleGetLocalVision(w http.ResponseWriter, r *http.Request) {
	frame := h.localVision.LatestFrame()
	respondJSON(w, frame)
}

func (h *Hub) handleGetBirdTracks(w http.ResponseWriter, r *http.Request) {
	if h.birdwatch == nil {
		respondJSON(w, map[string]interface{}{"tracks": []BirdTrack{}, "enabled": false})
		return
	}
	tracks := h.birdwatch.getActiveTracks()
	respondJSON(w, map[string]interface{}{"tracks": tracks, "enabled": true})
}

func (h *Hub) handleGetBirdSightings(w http.ResponseWriter, r *http.Request) {
	if h.birdwatch == nil {
		http.Error(w, "birdwatch not enabled", http.StatusServiceUnavailable)
		return
	}
	limitStr := r.URL.Query().Get("limit")
	limit := 50
	if limitStr != "" {
		if n, err := strconv.Atoi(limitStr); err == nil && n > 0 {
			limit = n
		}
	}
	sightings, err := h.birdwatch.GetRecentSightings(limit)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	respondJSON(w, sightings)
}

func (h *Hub) handleGetBirdSpecies(w http.ResponseWriter, r *http.Request) {
	if h.birdwatch == nil {
		http.Error(w, "birdwatch not enabled", http.StatusServiceUnavailable)
		return
	}
	species, err := h.birdwatch.GetSpeciesList()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	respondJSON(w, species)
}

func respondJSON(w http.ResponseWriter, v interface{}) {
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(v); err != nil {
		log.Printf("json encode error: %v", err)
	}
}
