# Gates: Mazzaroth Standalone Galaxy Engine Extraction

- [x] G1: Standalone mazzaroth crate builds and passes its own unit tests
  CHECK: cargo test -p mazzaroth
  EXPECT: test result: ok.
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] G2: Paraclea TUI integrates mazzaroth and passes all galaxy tests
  CHECK: cargo test -p paraclea-tui
  EXPECT: test result: ok.
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.65s

- [x] G3: Clippy workspace lint clean with no warnings
  CHECK: cargo clippy --workspace -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.84s (0 warnings)

- [x] G4: Full workspace test suite passes with 100% green tests
  CHECK: cargo test --workspace
  EXPECT: test result: ok.
  EVIDENCE: test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.58s

- [x] G5: Release CLI binary compiles and builds successfully
  CHECK: cargo build --release -p paraclea-cli
  EXPECT: Finished
  EVIDENCE: Finished `release` profile [optimized] target(s) in 14.64s
