# Gates: Paraclea Embedded Vector RAG, Sneakernet Appliance & Interlinear Suite

- [x] G1: In-process Embedded Vector Store (`EmbeddedVectorStore`) with cosine similarity & top-k search (zero daemon)
  CHECK: cargo test -p paraclea-core --lib vector::tests::test_embedded_vector_store
  EXPECT: test result: ok.
  EVIDENCE: test vector::tests::test_embedded_vector_store ... ok (exit code 0)

- [x] G2: USB Sneakernet Sync Engine (`crates/paraclea-core/src/sneakernet.rs`) with encrypted delta synchronization
  CHECK: cargo test -p paraclea-core --lib sneakernet::tests::test_sneakernet_sync_roundtrip
  EXPECT: test result: ok.
  EVIDENCE: test sneakernet::tests::test_sneakernet_sync_roundtrip ... ok (exit code 0)

- [x] G3: Cross-Lingual Greek/Hebrew Interlinear Mode in Bible view with Strong's tags
  CHECK: cargo test -p paraclea-tui --lib views::bible::tests::test_interlinear_rendering
  EXPECT: test result: ok.
  EVIDENCE: test views::bible::tests::test_interlinear_rendering ... ok (exit code 0)

- [x] G4: LoRa Packet Monitor Telemetry & Oscilloscope View (`/packets`)
  CHECK: cargo test -p paraclea-tui --lib views::packets::tests::test_packet_telemetry
  EXPECT: test result: ok.
  EVIDENCE: test views::packets::tests::test_packet_telemetry ... ok (exit code 0)

- [x] G5: Full workspace test & zero clippy warnings verification
  CHECK: cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: test result: ok.
  EVIDENCE: 44/44 tests passed across 5 workspace crates, 0 clippy warnings (exit code 0)
