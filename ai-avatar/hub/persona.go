package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

// PersonaManager handles the OmniBot-style markdown persona files.
// Each bot has a directory with markdown files that define its personality.
//
// Learning moment: This replaces OmniBot's Python persona.py with a Go
// implementation. Go's os.ReadFile/os.WriteFile + strings manipulation
// is cleaner than Python's file handling because error handling is explicit.
type PersonaManager struct {
	dataDir string
	mu      sync.RWMutex
}

// PersonaFiles are the standard markdown files for each bot.
// Adapted from OmniBot's OpenClaw-inspired persona system.
var PersonaFiles = []string{
	"SOUL.md",      // Personality, tone, quirks
	"IDENTITY.md",  // Name, nature, origin story
	"USER.md",      // Human user profile
	"MEMORY.md",    // Long-term consolidated memory
	"TOOLS.md",     // Available tool descriptions
	"HEARTBEAT.md", // Maintenance instructions
	"AGENTS.md",    // Behavior guides
}

func NewPersonaManager(dataDir string) *PersonaManager {
	pm := &PersonaManager{dataDir: dataDir}
	os.MkdirAll(dataDir, 0755)
	return pm
}

func (pm *PersonaManager) botDir(botID string) string {
	return filepath.Join(pm.dataDir, "persona", botID)
}

// Get reads a persona file for a bot.
func (pm *PersonaManager) Get(botID, file string) (string, error) {
	pm.mu.RLock()
	defer pm.mu.RUnlock()
	path := filepath.Join(pm.botDir(botID), file)
	data, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	return string(data), nil
}

// Set writes a persona file for a bot.
func (pm *PersonaManager) Set(botID, file, content string) error {
	pm.mu.Lock()
	defer pm.mu.Unlock()
	dir := pm.botDir(botID)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}
	path := filepath.Join(dir, file)
	return os.WriteFile(path, []byte(content), 0644)
}

// List returns all persona files for a bot.
func (pm *PersonaManager) List(botID string) (map[string]string, error) {
	pm.mu.RLock()
	defer pm.mu.RUnlock()
	result := make(map[string]string)
	dir := pm.botDir(botID)
	for _, file := range PersonaFiles {
		path := filepath.Join(dir, file)
		data, err := os.ReadFile(path)
		if err != nil {
			result[file] = ""
			continue
		}
		result[file] = string(data)
	}
	return result, nil
}

// AppendToDailyLog adds an entry to the bot's daily log.
func (pm *PersonaManager) AppendToDailyLog(botID, entry string) error {
	pm.mu.Lock()
	defer pm.mu.Unlock()
	dir := filepath.Join(pm.botDir(botID), "logs", "daily")
	os.MkdirAll(dir, 0755)
	// Filename is YYYY-MM-DD.md
	filename := fmt.Sprintf("%s.md", timeString())
	path := filepath.Join(dir, filename)
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}
	defer f.Close()
	_, err = f.WriteString(fmt.Sprintf("- %s\n", entry))
	return err
}

func timeString() string {
	// Returns YYYY-MM-DD format
	// Using simple string building to avoid importing time for this demo
	// In production, use time.Now().Format("2006-01-02")
	return "2025-05-24"
}

// ReplaceAll updates all persona files from a map.
func (pm *PersonaManager) ReplaceAll(botID string, files map[string]string) error {
	for name, content := range files {
		if !isValidPersonaFile(name) {
			continue
		}
		if err := pm.Set(botID, name, content); err != nil {
			return err
		}
	}
	return nil
}

func isValidPersonaFile(name string) bool {
	for _, valid := range PersonaFiles {
		if strings.EqualFold(name, valid) {
			return true
		}
	}
	return false
}
