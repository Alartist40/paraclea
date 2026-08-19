package main

import (
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/gorilla/websocket"
)

// WebSocket upgrade configuration.
// Learning moment: upgrader is stored as a package-level var because
// its zero value is usable (defaults are sensible). No constructor needed.
var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		// Allow all origins for development.
		// In production, restrict to known dashboard origins.
		return true
	},
}

// handleWebSocket upgrades HTTP to WebSocket and manages the client lifecycle.
func (h *Hub) handleWebSocket(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("websocket upgrade failed: %v", err)
		return
	}
	defer conn.Close()

	client := &Client{
		hub:  h,
		send: make(chan []byte, 256),
	}
	h.RegisterClient(client)
	defer h.UnregisterClient(client)

	// Start writer goroutine — pumps messages from the send channel to the WebSocket.
	// Learning moment: One goroutine per direction (read/write) is the Go idiom
	// for WebSocket handling. This prevents blocking and allows clean shutdown.
	done := make(chan struct{})
	go func() {
		defer close(done)
		for msg := range client.send {
			conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := conn.WriteMessage(websocket.TextMessage, msg); err != nil {
				log.Printf("websocket write error: %v", err)
				return
			}
		}
	}()

	// Reader loop — processes incoming messages from the client.
	for {
		conn.SetReadDeadline(time.Now().Add(60 * time.Second))
		_, msg, err := conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("websocket read error: %v", err)
			}
			break
		}
		h.handleClientMessage(client, msg)
	}

	// Wait for writer to finish before returning.
	<-done
}

// handleClientMessage processes messages from the dashboard.
func (h *Hub) handleClientMessage(c *Client, msg []byte) {
	var req struct {
		Type string `json:"type"`
		Data map[string]interface{} `json:"data"`
	}
	if err := json.Unmarshal(msg, &req); err != nil {
		log.Printf("invalid client message: %v", err)
		return
	}

	switch req.Type {
	case "command":
		cmd, _ := req.Data["text"].(string)
		log.Printf("WebSocket command: %s", cmd)
		// TODO: Forward to Rust core
	case "ping":
		c.send <- []byte(`{"type":"pong"}`)
	default:
		log.Printf("unknown message type: %s", req.Type)
	}
}


