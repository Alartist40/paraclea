# Gates: Paraclea Candle Offline Speech-to-Text & Workspace Verification Suite

- [x] G1: Candle Audio Preprocessor & Mel Spectrogram calculation (`crates/paraclea-core/src/candle_stt.rs`)
  CHECK: cargo test -p paraclea-core --lib candle_stt::tests::test_candle_audio_preprocessing
  EXPECT: test result: ok.
  EVIDENCE: test candle_stt::tests::test_candle_audio_preprocessing ... ok (exit code 0)

- [x] G2: Candle Transcriber inference pipeline & WAV parsing
  CHECK: cargo test -p paraclea-core --lib candle_stt::tests::test_candle_transcription_pipeline
  EXPECT: test result: ok.
  EVIDENCE: test candle_stt::tests::test_candle_transcription_pipeline ... ok (exit code 0)

- [x] G3: TUI Voice Command integration in command palette and dispatcher
  CHECK: cargo test -p paraclea-tui --lib tests::test_process_commands
  EXPECT: test result: ok.
  EVIDENCE: test tests::test_process_commands ... ok (exit code 0)

- [x] G4: Full workspace test & zero clippy warnings verification
  CHECK: cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: test result: ok.
  EVIDENCE: 46/46 tests passed across 5 workspace crates, 0 clippy warnings (exit code 0)
