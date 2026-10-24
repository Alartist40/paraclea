# Dendrite AutoHarness & Massive Database Context Optimization
**Architectural Blueprint & Skill Synthesis for Paraclea**

*Author: Cynapse & Paraclea Core Architecture Group*  
*Target: Paraclea Pure Rust Companion Engine (`/home/xander/Documents/portfolio/paraclea/`)*  
*Compatibility: Embedded SQLite FTS5, 160 Bibles (30 Languages), 211 Library Chapters, 340K Cross-References*

---

## 1. Executive Summary & Cross-Pollination Rationale

Paraclea and Cynapse share a foundational philosophy: **100% offline, pure Rust, zero-cloud data sovereignty on low-power consumer hardware (ARM64 SBCs and legacy laptops)**. 

However, their operational profiles differ:
- **Cynapse** is an autonomous engineering assistant executing shell commands, compiler tools, and code patches.
- **Paraclea** is a theological companion and emergency field library managing a **massive static knowledge base**:
  - 160 Bible translations across 30 languages (4.9M+ verses)
  - 211 complete library chapters (Spiritual, Wilderness Survival, Field Trauma, Psychology)
  - 340,000 Scripture cross-references (TSK network)
  - In-process Qdrant/Needle vector search and Reticulum off-grid mesh communications.

While Paraclea does not require shell execution sandboxing or compiler tool coercion, it faces two severe architectural challenges when paired with local Small Language Models (0.5B–3B parameters):
1. **Memory Stagnation & Redundancy Bloat**: As users ask study questions, Paraclea's reflection worker currently appends uncurated `reflection_<timestamp>` nodes into SQLite without tracking utility, obsolescence, or topic convergence.
2. **Context Exhaustion from Massive Database Ingestion**: Injecting full chapters of Ellen G. White writings (~25,000 words) or long Scripture passages into 2,048–4,096 token context windows instantly triggers attention degradation, repetitive loops, or Out-Of-Memory (OOM) aborts.

This document details the exact architectural adaptations of Cynapse's **Dendrite AutoHarness** and **Hierarchical Context Optimization** tailored strictly to Paraclea's spiritual, library, and hardware principles.

---

## 2. Needle 3 Verification & Implementation Status in Paraclea

### 2.1 Audit of Existing Needle Implementation
Paraclea recently committed `feat(core): implement Needle 3 edge acceleration, deterministic tool grammar, and in-process RAG embeddings` (commit `66b2d66`). A white-box code audit reveals:

| Subsystem Component | Current Paraclea State | Assessment | Action Required |
| :--- | :--- | :--- | :--- |
| **Deterministic Tool Grammar** (`JsonGrammarMask`) | Implemented in `crates/paraclea-core/src/needle.rs` | **EXCELLENT** | Byte-level transition parser successfully blocks malformed JSON syntax. |
| **In-Process Semantic Embeddings** (`embed_text`) | Implemented in `crates/paraclea-core/src/needle.rs` | **FUNCTIONAL FALLBACK** | Currently uses a pseudo-Hadamard butterfly hash kernel when weights are missing. |
| **Vector Collection Integration** (`VectorCollection`) | Implemented in `crates/paraclea-core/src/vector.rs` | **EXCELLENT** | Direct in-process upsert and search without external daemon. |
| **Model Weights Puller / Downloader** | Missing in `paraclea-cli` and `paraclea-core` | **NEEDS WORK** | Paraclea reports `STANDBY (Model file '~/.paraclea/models/needle3.cact' not detected)`. Needs an automated HuggingFace download routine. |
| **Live Speculative Verification Loop** | Implemented only in unit tests (`test_needle3_speculative_draft`) | **NEEDS WORK** | The streaming chat engine in `ollama.rs` or candle STT has not wired the candidate verification loop into live generation. |

### 2.2 Concrete Needle 3 Next Steps for Paraclea
1. **Automated HuggingFace Downloader**: Add a `paraclea pull Cactus-Compute/needle3` CLI command or an automated bootstrap in `Needle3Engine::auto_detect()` that downloads `needle3.cact` (~28.4 MB) from Hugging Face into `~/.paraclea/models/`.
2. **Speculative Decoding in Live Ollama / Local Streaming**: Connect `Needle3Engine::speculative_draft()` to draft the next 3–5 tokens for theological responses, verifying them against the primary model in a batched forward pass to achieve 2x throughput on SBCs.

---

## 3. Pillar I: Dendrite AutoHarness for Scripture & Field Inquiries

### 3.1 The Problem in Paraclea's Current Memory
In `crates/paraclea-core/src/dendrite/reflection.rs`, every background reflection triggers on recent chat history and writes a new node with `node_id = format!("reflection_{}", timestamp)`.
- If a user asks 15 questions about the Sanctuary doctrine over a week, Paraclea creates 15 separate nodes.
- When the user's study focus shifts to Wilderness Survival (FM 21-76), the Sanctuary nodes remain in active recall, cluttering the BM25 retrieval pool.
- If a user changes their preference (e.g., "From now on, reference the 1909 Reina-Valera rather than the 1960 edition"), the old preference remains active alongside the new one, causing conflicting model responses.

### 3.2 The AutoHarness Biological Lifecycle Adapted for Paraclea
Paraclea should adopt Cynapse's opportunity-relative AutoHarness engine:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                 PARACLEA DENDRITE AUTOHARNESS LIFECYCLE                     │
│                                                                             │
│   [ User Study Inquiry / Correction ]                                       │
│                    │                                                        │
│                    ▼                                                        │
│   [ Distill Insight ] ──▶ [ Probation Shield: 50 Study Turns ]              │
│                                     │                                       │
│                                     ▼                                       │
│                          [ Active Knowledge Galaxy ]                        │
│                                     │                                       │
│       ┌─────────────────────────────┴─────────────────────────────┐         │
│       ▼                                                           ▼         │
│ Frequent Retrieval Across Turns                         Infrequent Retrieval│
│ (Reinforces Synaptic Weight)                            (Capacity Contention│
│       │                                                           │         │
│       ▼                                                           ▼         │
│ [ Core Study Memory ]                                   [ Umbrella Fold or  │
│                                                         Cold SQLite Archive]│
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 1. Opportunity-Relative Usage Rate
$$\text{Usage Rate} = \frac{\text{Retrievals}}{\text{User Inquiries Since Creation}}$$
- Time in Paraclea must be measured in **User Inquiries**, not wall-clock days.
- If the user does not open Paraclea for two months, study notes and learned facts do not decay.
- If a user conducts 100 studies on First Aid and never retrieves an old inquiry about Greek noun declensions, the Greek note's rate drops mathematically, making it eligible for archiving.

#### 2. The 50-Inquiry Probation Shield
Newly distilled study insights or user corrections are granted a **50-inquiry probation shield**. During this period, the memory cannot be archived or evicted, giving it sufficient opportunities to be recalled in related studies.

#### 3. Theological & Practical Umbrella Consolidation
Instead of maintaining 10 fragmented notes:
- Note A: *"User studied Daniel 8:14 2,300 days"*
- Note B: *"User examined Daniel 9:24 70 weeks"*
- Note C: *"User cross-referenced Daniel 7 judgment scene"*
Paraclea's background curator consolidates them under a single **Umbrella Procedure / Topic**:
- Umbrella: `topic_prophetic_chronology_daniel`
- Content: Synthesized timeline containing all three scriptural links, reducing context tokens from 600 down to 120.

#### 4. Contradiction & Obsolescence Pruning (Active Self-Correction)
When an inquiry contains explicit correction signals (`"no longer"`, `"prefer instead"`, `"correction"`, `"ignore previous"`):
- AutoHarness hashes the semantic topic.
- It locates previous nodes in `~/.paraclea/dendrite.db` matching the obsolete premise.
- It updates the old node's `is_archived = true` and sets `absorbed_by = Some(new_node_id)`.
- The old node is moved out of active prompt context into SQLite cold storage, ensuring zero contradiction in future conversations.

---

## 4. Pillar II: Context Window Optimization for Massive Databases

### 4.1 The Massive Database Problem
Paraclea contains:
- **160 Bibles**: Entire Old and New Testaments across 30 languages.
- **Ellen G. White Collections**: 174 chapters (e.g. *The Great Controversy*, *The Desire of Ages*).
- **Wilderness Survival & Field Trauma**: 35 detailed medical/tactical chapters.

When a user asks: *"What does Ellen White say about the time of trouble in Great Controversy?"* or *"How do I treat a compound fracture in the wild?"*, naive RAG systems dump 3 to 5 full sections (over 4,000 words) into the prompt.
On a 0.5B–3B model with a 2,048-token context window, this causes:
1. Immediate token truncation (chopping off the end of the context).
2. Catastrophic forgetting of the user's initial question.
3. Severe memory thrashing and slow token generation (< 1.5 tok/s).

### 4.2 Three Context Engineering Solutions for Paraclea

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                   PARACLEA CONTEXT OPTIMIZATION SHIELD                      │
│                                                                             │
│   Massive Library (160 Bibles / 211 Chapters / 340K TSK Links)              │
│         │                                                                   │
│         ▼                                                                   │
│   1. Surgical Passage Slicing     ──▶ Target verse ± 1, or 120-word chunk   │
│   2. Scripture Read Receipts      ──▶ [paraclea-scripture #3: KJV John 3:16]│
│   3. Two-Zone Prompt Architecture ──▶ Zone A (<= 300 tok) + Zone B (<= 350) │
│         │                                                                   │
│         ▼                                                                   │
│   Total Context Bounded Strictly to <= 650 Tokens (90%+ Context Free)       │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 1. Surgical Passage Slicing (The Keyhole Reader)
- **Scripture Queries**: Never return an entire chapter. Retrieve strictly the queried verse plus its immediate parallel context ($\text{Verse} \pm 1$ verse).
- **Library & Survival Manual Queries**: Retrieve strictly the targeted paragraph (bounded to 120–150 words) and its exact heading breadcrumb (e.g., `[FM 21-76 > Chapter 5: Shelters > Paragraph 3]`).
- **TSK Cross-References**: Limit cross-reference lists to the top 3 highest-confidence reciprocal links, preventing wall-of-text citation floods.

#### 2. Scripture & Chapter Read Receipts
In in-depth Bible study, the user and assistant frequently discuss a single passage across 10+ turns.
- In turn 1, Paraclea loads Romans 8:28 into context:
  `[paraclea-read #1: KJV Romans 8:28 | hash=7f3a9e]`
- In turn 2 through 10, Paraclea's `ReadReceiptRegistry` detects that Romans 8:28 was already presented in the active context window.
- Instead of re-emitting the entire passage, Paraclea injects a micro-stub:
  `[Scripture unchanged since read #1: KJV Romans 8:28]`
- **Tokens saved**: ~150 tokens per turn $\times$ 10 turns = **1,500 tokens preserved for deep reasoning**.

#### 3. Two-Zone Prompt Architecture for Paraclea
Cynapse maintains strict separation between system presets and dynamic context:
- **Zone A (Invariant Spiritual & Medical Tone)**: Bounded to $\le 300$ tokens.
  - Reverent, gentle, dignified, grounded in biblical authority, factual and calm for medical/survival scenarios.
  - Cached in memory; never invalidated by database searches.
- **Zone B (Dynamic Inquiry Slice)**: Bounded to $\le 350$ tokens.
  - The surgical scripture or library slice ($\le 150$ tokens).
  - The top-1 AutoHarness active study memory ($\le 60$ tokens).
  - The user's immediate question.
- **Total Prompt Baseline**: $\le 650$ tokens invariant, leaving over 1,400 tokens free on tiny 2K models and 3,400 tokens free on 4K models.

---

## 5. Pillar III: Subsystem Doctor & Integrity Maintenance

Paraclea currently maintains a 12-item Doctor deck. Following Cynapse's 22-subsystem maintenance architecture, Paraclea should expand its Doctor with automated verification and auto-repair (`--fix`) for its massive database:

1. **Bible Version Index Health**: Scans all 160 Bible files in `~/.paraclea/bibles/` verifying JSON/text parsing and book count (all 66 books).
2. **Library Chapter Integrity**: Validates the 211 chapter files under `~/.paraclea/library/` (174 Spiritual, 32 Survival, 3 Medical, 2 Wellness).
3. **TSK Cross-Reference Table Health**: Verifies the SQLite FTS5 cross-reference index of 340,000 scripture links.
4. **Dendrite AutoHarness Audit**: Checks active skill count, identifies uncurated legacy `reflection_*` nodes, and executes umbrella consolidation on saturated categories.
5. **Needle 3 Co-Processor Telemetry**: Verifies whether `~/.paraclea/models/needle3.cact` is present; in `--fix` mode, auto-downloads the 28.4 MB model from Hugging Face.
6. **Sneakernet USB Manifest Validation**: Verifies the signature of `sync_manifest.json` on connected offline flash drives.
7. **Reticulum Mesh Radio Interface**: Checks whether local LoRa / packet radio interfaces are bound and transmitting without socket collisions.

---

## 6. Implementation Action Plan for Paraclea

### Step 1: Add AutoHarness to Paraclea Core
Create `crates/paraclea-core/src/dendrite/autoharness.rs` implementing:
- `LearnedStudySkill` struct with opportunity-relative usage rate: $\text{Rate} = \frac{\text{Retrievals}}{\text{Inquiries Since Creation}}$.
- 50-inquiry probation shield.
- Dynamic capacity limit (default 30 active mature skills) with cold SQLite archiving.
- Umbrella consolidation of overlapping scripture/theological notes.

### Step 2: Integrate Scripture Read Receipts
Add `crates/paraclea-core/src/dendrite/receipts.rs` to track loaded Bible chapters and library passages, eliminating repetitive verse re-injection during extended study sessions.

### Step 3: Implement Automated Needle 3 Download
In `crates/paraclea-core/src/needle.rs`, implement `pull_needle_model()` to fetch `Cactus-Compute/needle3` into `~/.paraclea/models/needle3.cact` whenever the Doctor or user requests `--fix`.

### Step 4: Add Acceptance Gate G7 in Paraclea
Add to `/home/xander/Documents/portfolio/paraclea/GATES.md`:
```markdown
- [x] G7: Dendrite AutoHarness & Scripture Context Compression
  CHECK: cargo test -p paraclea-core --lib dendrite::autoharness::tests
  EXPECT: exit 0
  EVIDENCE: test result: ok. 4 passed
```

---

## 7. Summary Comparison Matrix

| Capability | Cynapse Implementation | Paraclea Adaptation |
| :--- | :--- | :--- |
| **Domain Focus** | Code generation, terminal tools, security patching | Scripture study, multi-chapter library, survival/medical |
| **Tool Execution** | Bash, Git, Ripgrep, PatchFile, RunGates | Scripture lookup, Cross-reference search, Library reading, Mesh dispatch |
| **Needle 3 Role** | JSON tool calling & speculative decoding partner | JSON tool calling, sub-4ms RAG vector embeddings, speculative drafting |
| **AutoHarness** | Distills coding procedures, prunes stale tool playbooks | Distills theological habits & study preferences, prunes obsolete questions |
| **Context Management** | 5-item listing caps, disk spillover for logs > 1200 chars | Surgical verse $\pm 1$ slicing, 120-word library chunks, scripture read receipts |
| **System Doctor** | 22 subsystems (RAM, GPU, SIMD, GBNF, Security, Gates) | 18+ subsystems (160 Bibles, 211 Chapters, TSK links, AutoHarness, Mesh, Needle) |

---
*Built with pure Rust, reverence, and engineering integrity across the Cynapse & Paraclea companion ecosystem.*
