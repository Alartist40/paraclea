# Gates: Paraclea Architecture & Memory Elevation

- [x] G1: Workspace tests pass with Dendrite topic-keys and progressive disclosure
  CHECK: cargo test -p paraclea-core -- test_topic_key_upsert_and_progressive_disclosure
  EXPECT: test dendrite::store::tests::test_topic_key_upsert_and_progressive_disclosure ... ok
  CWD: .
  EVIDENCE: test result: ok. 1 passed; 0 failed

- [x] G2: SQLite schema migration preserves existing dendrite nodes and adds new engram columns
  CHECK: cargo test -p paraclea-core -- test_dendrite_schema_migration
  EXPECT: test dendrite::store::tests::test_dendrite_schema_migration ... ok
  CWD: .
  EVIDENCE: test result: ok. 1 passed; 0 failed

- [x] G3: Two-zone prompt split outputs byte-stable Zone A prefix and dynamic Zone B tail
  CHECK: cargo test -p paraclea-core -- test_two_zone_prompt_compilation
  EXPECT: test persona::tests::test_two_zone_prompt_compilation ... ok
  CWD: .
  EVIDENCE: test result: ok. 1 passed; 0 failed

- [x] G4: Full workspace compiles cleanly and passes all test suites
  CHECK: cargo test --workspace
  EXPECT: test result: ok.
  CWD: .
  EVIDENCE: 73 passed across all crates (mazzaroth, paraclea-core, paraclea-tui, paraclea-cli)

- [x] G5: Release binary builds cleanly
  CHECK: cargo build --release -p paraclea-cli
  EXPECT: Finished `release` profile
  CWD: .
  EVIDENCE: Finished release profile [optimized] in 12.98s and copied to ~/.local/bin/paraclea
