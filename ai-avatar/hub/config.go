package main

// Config holds hub-wide settings.
// Learning moment: Go structs with json tags are the standard pattern
// for configuration and API payloads. No classes, no inheritance —
// just data with tagged fields for automatic serialization.
type Config struct {
	Host         string `json:"host"`
	Port         string `json:"port"`
	DataDir      string `json:"data_dir"`
	RustCorePath string `json:"rust_core_path"`
	DashboardDir string `json:"dashboard_dir"`
}

func DefaultConfig() Config {
	return Config{
		Host:         "0.0.0.0",
		Port:         "8080",
		DataDir:      "./data",
		RustCorePath: "../target/release/ai-avatar",
		DashboardDir: "../dashboard/dist",
	}
}
