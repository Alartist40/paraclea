# Gates: Paraclea Universal Elevation & Optimization

- [x] G1: Context ring buffer pruning and FlashAttention/num_gpu options
  CHECK: cargo test -p paraclea-core -- test_context_ring_buffer_pruning
  EXPECT: test persona::tests::test_context_ring_buffer_pruning ... ok
  CWD: .
  EVIDENCE: passed (test result: ok. 1 passed in 0.00s)

- [x] G2: Strong's lexicon theological terms cache pre-warming
  CHECK: cargo test -p paraclea-core -- test_strongs_cache_prewarming
  EXPECT: test strongs::tests::test_strongs_cache_prewarming ... ok
  CWD: .
  EVIDENCE: passed (test result: ok. 1 passed in 0.00s)

- [x] G3: TUI Split-Screen Study Mode (`Ctrl+B`) & Quick-Jump Palette modal
  CHECK: cargo test -p paraclea-tui -- test_split_screen_and_quick_jump
  EXPECT: test tests::test_split_screen_and_quick_jump ... ok
  CWD: .
  EVIDENCE: passed (test result: ok. 1 passed in 0.01s)

- [x] G4: Full workspace compiles with zero clippy warnings
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished `dev` profile
  CWD: .
  EVIDENCE: passed (Finished dev profile with 0 warnings)

- [x] G5: Full workspace test suite passes cleanly
  CHECK: cargo test --workspace
  EXPECT: test result: ok.
  CWD: .
  EVIDENCE: passed (76 tests passed, 0 failed across all crates)
