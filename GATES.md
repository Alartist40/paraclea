# Gates: Cross-Platform Portability Overhaul (Windows, macOS, ARM64 & Linux)

- [x] G1: Add dirs crate and cross-platform helpers (home_dir, temp_dir, shell_command, shell_arg) in paraclea-core
  CHECK: cargo test -p paraclea-core
  EXPECT: test result: ok.
  EVIDENCE: test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.74s

- [x] G2: Eliminate all hardcoded HOME env vars in favor of paraclea_core::home_dir()
  CHECK: cargo test --workspace
  EXPECT: test result: ok.
  EVIDENCE: Grep confirmed 0 occurrences of env::var("HOME"); 28/28 tests passed

- [x] G3: Eliminate all hardcoded /tmp/ strings in favor of paraclea_core::temp_dir()
  CHECK: cargo test --workspace
  EXPECT: test result: ok.
  EVIDENCE: Grep confirmed 0 occurrences of hardcoded "/tmp"; 28/28 tests passed

- [x] G4: Cross-platform shell spawning and daemon execution implemented
  CHECK: cargo test -p paraclea-cli
  EXPECT: test result: ok.
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] G5: Audio playback fallbacks and USB backup drive scanners implemented for macOS & Windows
  CHECK: cargo clippy --workspace -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.69s (0 warnings)

- [x] G6: Full workspace test suite passes with 100% green tests
  CHECK: cargo test --workspace
  EXPECT: test result: ok.
  EVIDENCE: test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.73s

- [x] G7: Release binary compiles and installs cleanly
  CHECK: cargo build --release -p paraclea-cli
  EXPECT: Finished
  EVIDENCE: Finished `release` profile [optimized] target(s) in 17.06s; installed to ~/.local/bin/paraclea and ~/.cargo/bin/paraclea
