# Gates: Paraclea TUI UI/UX Audit Fixes & Verification

- [x] G1: Chat auto-scroll pin on send/streaming/done + user scroll preservation
  CHECK: cargo test -p paraclea-tui --lib tests::test_chat_auto_scroll
  EXPECT: test result: ok.
  EVIDENCE: test tests::test_chat_auto_scroll ... ok (exit code 0)

- [x] G2: 1-7 numeric keys switch decks from MainViewport/Sidebar; boots into MainViewport so they work on launch; digits type freely in PromptInput; Alt+1-7 switches from anywhere
  CHECK: cargo test -p paraclea-tui --lib tests::test_numeric_tab_switch_on_empty_prompt
  EXPECT: test result: ok.
  EVIDENCE: test tests::test_numeric_tab_switch_on_empty_prompt ... ok (exit code 0)

- [x] G3: Real stream cancellation with AbortHandle stops background Ollama task and ignores trailing tokens
  CHECK: cargo test -p paraclea-tui --lib tests::test_real_stream_cancellation
  EXPECT: test result: ok.
  EVIDENCE: test tests::test_real_stream_cancellation ... ok (exit code 0)

- [x] G4: Galaxy view panel_bg + Color::DarkGray elimination across codebase
  CHECK: grep -rn "Color::DarkGray" crates/paraclea-tui/src/
  EXPECT: NO_DARKGRAY_FOUND
  EVIDENCE: NO_DARKGRAY_FOUND (0 occurrences across entire crate)

- [x] G5: Chapter clamp on all navigation paths + hardcoded counts dynamic + error toast
  CHECK: cargo test -p paraclea-tui --lib tests::test_chapter_clamping_all_paths
  EXPECT: test result: ok.
  EVIDENCE: test tests::test_chapter_clamping_all_paths ... ok (exit code 0)

- [x] G6: Full workspace test suite passes with 100% green tests & zero clippy warnings
  CHECK: cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: test result: ok.
  EVIDENCE: 35/35 tests passed across workspace (exit code 0); 0 clippy warnings on --all-targets.

- [x] G7: Binary installed and runnable via system PATH
  CHECK: cargo build --release -p paraclea-cli && cp target/release/paraclea ~/.local/bin/ && cp target/release/paraclea ~/.cargo/bin/
  EXPECT: Both install paths hold the same fresh binary
  EVIDENCE: ~/.local/bin/paraclea and ~/.cargo/bin/paraclea updated from the same release build — md5 30a079af7e6ea5071e733cd863a68b5e for both



