# voice — Makefile
# Quick commands for building and installing the Go toolchain.

BINARY := voice
INSTALL_DIR := /usr/local/bin

.PHONY: all build install clean test segment transcribe metadata speak dataset

all: build

build:
	go build -o $(BINARY) .
	@echo "Built: ./$(BINARY)"

install: build
	cp $(BINARY) $(INSTALL_DIR)/$(BINARY)
	@echo "Installed to $(INSTALL_DIR)/$(BINARY)"

uninstall:
	rm -f $(INSTALL_DIR)/$(BINARY)
	@echo "Uninstalled"

clean:
	rm -f $(BINARY)
	go clean

test: build
	./$(BINARY) help

# Convenience targets for the ai-avatar project
segment: build
	./$(BINARY) segment \
		-input ../aliceinwonderland_01_gerstenberg_64kb.mp3 \
		-output ../dataset/training_data/wavs

transcribe: build
	./$(BINARY) transcribe \
		-input ../dataset/training_data/wavs \
		-model ../models/ggml-tiny.bin

metadata: build
	./$(BINARY) metadata \
		-input ../dataset/training_data/wavs \
		-output ../dataset/training_data/metadata.csv

speak: build
	./$(BINARY) speak \
		-text "Hello, I am Paraclea. How can I help you today?" \
		-output /tmp/go_voice_test.wav

dataset: build
	./$(BINARY) dataset \
		-input ../aliceinwonderland_01_gerstenberg_64kb.mp3 \
		-output ../dataset/training_data
