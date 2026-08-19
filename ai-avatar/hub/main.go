package main

import (
	"context"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
)

// Hub Service — Go-based orchestrator for the ai-avatar pipeline.

func main() {
	cfg := DefaultConfig()
	hub := NewHub(cfg)

	// Initialize birdwatch service
	birdwatch, err := NewBirdWatchService(cfg.DataDir)
	if err != nil {
		log.Printf("[hub] birdwatch init failed: %v", err)
	} else {
		hub.birdwatch = birdwatch
	}

	hub.Run()
	defer hub.Stop()

	// Start the broadcast loop in a background goroutine.
	go hub.StartBroadcastLoop()

	mux := http.NewServeMux()

	// REST API routes
	mux.HandleFunc("/api/state", hub.handleGetState)
	mux.HandleFunc("/api/vision", hub.handleGetVision)
	mux.HandleFunc("/api/local-vision", hub.handleGetLocalVision)
	mux.HandleFunc("/api/birdwatch/tracks", hub.handleGetBirdTracks)
	mux.HandleFunc("/api/birdwatch/sightings", hub.handleGetBirdSightings)
	mux.HandleFunc("/api/birdwatch/species", hub.handleGetBirdSpecies)
	mux.HandleFunc("/api/persona", func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			hub.handleGetPersona(w, r)
		case http.MethodPost:
			hub.handleSetPersona(w, r)
		default:
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		}
	})
	mux.HandleFunc("/api/command", hub.handleSendCommand)

	// WebSocket route
	mux.HandleFunc("/ws", hub.handleWebSocket)

	// Static file server for dashboard (production build)
	mux.Handle("/", http.FileServer(http.Dir(cfg.DashboardDir)))

	addr := cfg.Host + ":" + cfg.Port
	server := &http.Server{
		Addr:         addr,
		Handler:      mux,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	// Graceful shutdown on SIGINT/SIGTERM.
	done := make(chan os.Signal, 1)
	signal.Notify(done, os.Interrupt, syscall.SIGTERM)

	go func() {
		log.Printf("Hub listening on http://%s", addr)
		if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("server error: %v", err)
		}
	}()

	<-done
	log.Println("Shutting down hub...")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := server.Shutdown(ctx); err != nil {
		log.Printf("shutdown error: %v", err)
	}
	log.Println("Hub stopped")
}
