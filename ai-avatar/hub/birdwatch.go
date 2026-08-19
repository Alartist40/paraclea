package main

import (
	"database/sql"
	"fmt"
	"log"
	"math"
	"os"
	"path/filepath"
	"sync"
	"time"

	_ "github.com/mattn/go-sqlite3"
)

// BirdSighting represents a logged bird observation.
type BirdSighting struct {
	ID         string    `json:"id"`
	Species    string    `json:"species"`
	Confidence float32   `json:"confidence"`
	Timestamp  time.Time `json:"timestamp"`
	BBoxX      int       `json:"bbox_x"`
	BBoxY      int       `json:"bbox_y"`
	BBoxW      int       `json:"bbox_w"`
	BBoxH      int       `json:"bbox_h"`
}

// BirdTrack represents a tracked bird across frames.
type BirdTrack struct {
	TrackID      int64     `json:"track_id"`
	Species      string    `json:"species"`
	Confidence   float32   `json:"confidence"`
	CentroidX    int       `json:"centroid_x"`
	CentroidY    int       `json:"centroid_y"`
	X            int       `json:"x"`
	Y            int       `json:"y"`
	W            int       `json:"w"`
	H            int       `json:"h"`
	FrameCount   int       `json:"frame_count"`
	TotalDist    int       `json:"total_dist"`
	LastDetected time.Time `json:"last_detected"`
	Status       string    `json:"status"`
}

// BirdWatchService tracks birds detected by vision systems.
type BirdWatchService struct {
	db            *sql.DB
	activeTracks  map[int64]*BirdTrack
	nextTrackID   int64
	distThreshold int
	maxMissedAge  time.Duration
	mu            sync.Mutex
	running       bool
	stopCh        chan struct{}
}

func NewBirdWatchService(dataDir string) (*BirdWatchService, error) {
	dbPath := filepath.Join(dataDir, "birdwatch.sqlite")
	os.MkdirAll(dataDir, 0755)

	db, err := sql.Open("sqlite3", dbPath+"?_journal=WAL")
	if err != nil {
		return nil, err
	}

	schema := `
	CREATE TABLE IF NOT EXISTS bird_sightings (
		id         TEXT PRIMARY KEY,
		species    TEXT NOT NULL,
		confidence REAL NOT NULL,
		timestamp  INTEGER NOT NULL,
		bbox_x     INTEGER,
		bbox_y     INTEGER,
		bbox_w     INTEGER,
		bbox_h     INTEGER
	);
	CREATE TABLE IF NOT EXISTS bird_tracks (
		track_id    INTEGER PRIMARY KEY,
		species     TEXT,
		confidence  REAL,
		centroid_x  INTEGER,
		centroid_y  INTEGER,
		frame_count INTEGER,
		total_dist  INTEGER,
		last_seen   INTEGER,
		status      TEXT
	);
	CREATE INDEX IF NOT EXISTS idx_sightings_timestamp ON bird_sightings(timestamp);
	CREATE INDEX IF NOT EXISTS idx_sightings_species ON bird_sightings(species);
	CREATE INDEX IF NOT EXISTS idx_tracks_status ON bird_tracks(status);
	`
	if _, err := db.Exec(schema); err != nil {
		return nil, err
	}

	return &BirdWatchService{
		db:            db,
		activeTracks:  make(map[int64]*BirdTrack),
		nextTrackID:   1,
		distThreshold: 100,
		maxMissedAge:  time.Second * 2,
		stopCh:        make(chan struct{}),
	}, nil
}

func (bw *BirdWatchService) Start() error {
	bw.running = true
	log.Println("[birdwatch] started")
	return nil
}

func (bw *BirdWatchService) Stop() {
	bw.mu.Lock()
	defer bw.mu.Unlock()
	if !bw.running {
		return
	}
	bw.running = false
	close(bw.stopCh)
	bw.db.Close()
	log.Println("[birdwatch] stopped")
}

// ProcessDetections filters for bird class and updates tracks.
func (bw *BirdWatchService) ProcessDetections(detections []LocalDetection) []BirdTrack {
	var birds []LocalDetection
	for _, d := range detections {
		if d.ClassName == "bird" {
			birds = append(birds, d)
		}
	}
	if len(birds) == 0 {
		bw.pruneLostTracks()
		return bw.getActiveTracks()
	}

	bw.mu.Lock()
	defer bw.mu.Unlock()

	matched := make(map[int]bool)
	now := time.Now()

	// Match existing tracks
	for _, track := range bw.activeTracks {
		bestMatch := -1
		bestDistance := bw.distThreshold

		for i, det := range birds {
			if matched[i] {
				continue
			}
			cx := det.Bbox[0] + det.Bbox[2]/2
			cy := det.Bbox[1] + det.Bbox[3]/2
			dx := float64(cx - track.CentroidX)
			dy := float64(cy - track.CentroidY)
			dist := int(math.Sqrt(dx*dx + dy*dy))

			if dist < bestDistance {
				bestDistance = dist
				bestMatch = i
			}
		}

		if bestMatch >= 0 {
			det := birds[bestMatch]
			track.X = det.Bbox[0]
			track.Y = det.Bbox[1]
			track.W = det.Bbox[2]
			track.H = det.Bbox[3]
			track.CentroidX = det.Bbox[0] + det.Bbox[2]/2
			track.CentroidY = det.Bbox[1] + det.Bbox[3]/2
			track.Species = det.ClassName
			track.Confidence = det.Confidence
			track.LastDetected = now
			track.FrameCount++
			track.TotalDist += bestDistance
			track.Status = "active"
			matched[bestMatch] = true
		} else {
			track.Status = "lost"
		}
	}

	// Create new tracks for unmatched detections
	for i, det := range birds {
		if !matched[i] {
			trackID := bw.nextTrackID
			bw.nextTrackID++
			track := &BirdTrack{
				TrackID:      trackID,
				Species:      det.ClassName,
				Confidence:   det.Confidence,
				X:            det.Bbox[0],
				Y:            det.Bbox[1],
				W:            det.Bbox[2],
				H:            det.Bbox[3],
				CentroidX:    det.Bbox[0] + det.Bbox[2]/2,
				CentroidY:    det.Bbox[1] + det.Bbox[3]/2,
				LastDetected: now,
				FrameCount:   1,
				Status:       "active",
			}
			bw.activeTracks[trackID] = track

			// Log sighting to DB
			bw.logSightingLocked(det)
		}
	}

	// Prune lost tracks
	for id, track := range bw.activeTracks {
		if track.Status == "lost" && now.Sub(track.LastDetected) > bw.maxMissedAge {
			delete(bw.activeTracks, id)
		}
	}

	return bw.getActiveTracksLocked()
}

func (bw *BirdWatchService) logSightingLocked(det LocalDetection) {
	id := fmt.Sprintf("%d-%d", time.Now().UnixNano(), det.Bbox[0])
	_, err := bw.db.Exec(
		`INSERT INTO bird_sightings (id, species, confidence, timestamp, bbox_x, bbox_y, bbox_w, bbox_h)
		 VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
		id, det.ClassName, det.Confidence, time.Now().Unix(),
		det.Bbox[0], det.Bbox[1], det.Bbox[2], det.Bbox[3],
	)
	if err != nil {
		log.Printf("[birdwatch] failed to log sighting: %v", err)
	}
}

func (bw *BirdWatchService) pruneLostTracks() {
	bw.mu.Lock()
	defer bw.mu.Unlock()
	now := time.Now()
	for id, track := range bw.activeTracks {
		if track.Status == "lost" && now.Sub(track.LastDetected) > bw.maxMissedAge {
			delete(bw.activeTracks, id)
		}
	}
}

func (bw *BirdWatchService) getActiveTracks() []BirdTrack {
	bw.mu.Lock()
	defer bw.mu.Unlock()
	return bw.getActiveTracksLocked()
}

func (bw *BirdWatchService) getActiveTracksLocked() []BirdTrack {
	tracks := make([]BirdTrack, 0, len(bw.activeTracks))
	for _, t := range bw.activeTracks {
		tracks = append(tracks, *t)
	}
	return tracks
}

func (bw *BirdWatchService) GetSpeciesList() ([]string, error) {
	rows, err := bw.db.Query("SELECT DISTINCT species FROM bird_sightings")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var species []string
	for rows.Next() {
		var s string
		if err := rows.Scan(&s); err != nil {
			continue
		}
		species = append(species, s)
	}
	return species, nil
}

func (bw *BirdWatchService) GetRecentSightings(limit int) ([]BirdSighting, error) {
	rows, err := bw.db.Query(
		`SELECT id, species, confidence, timestamp, bbox_x, bbox_y, bbox_w, bbox_h
		 FROM bird_sightings ORDER BY timestamp DESC LIMIT ?`, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var sightings []BirdSighting
	for rows.Next() {
		var s BirdSighting
		var ts int64
		if err := rows.Scan(&s.ID, &s.Species, &s.Confidence, &ts, &s.BBoxX, &s.BBoxY, &s.BBoxW, &s.BBoxH); err != nil {
			continue
		}
		s.Timestamp = time.Unix(ts, 0)
		sightings = append(sightings, s)
	}
	return sightings, nil
}
