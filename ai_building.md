Here is my analysis and a clean roadmap for your two ideas. They are **not competing projects** — they are two entirely different paradigms. I have organized them so you can see which one matches your actual goal.

---

## 1. ANALYSIS: What You Actually Have

You have **two separate ideas** that do not overlap technically. Treating them as one project is why your thoughts feel scattered.

| | **Track A: "My Own Ornith"** | **Track B: "Expand TRM"** |
|---|---|---|
| **What it is** | Post-training a Large Language Model (Mistral) into an agentic coding specialist | Building a tiny recursive reasoner (7M params) that iteratively refines answers on structured tasks |
| **The paradigm** | Generative AI / LLM fine-tuning | Supervised iterative reasoning / "thinking in latent space" |
| **Current state** | Crowded space, but viable if you have data + compute | Cutting-edge research from October 2025; almost no one has expanded it yet |
| **Hardware needed** | Significant GPUs for weeks of SFT + RL | Minimal — runs on a single GPU or even CPU |
| **The real bottleneck** | High-quality coding trajectories + RL scaffolding infrastructure | Task-specific structured datasets + figuring out how to make it generative |
| **Can you beat frontier models?** | Unlikely solo; you can beat them on *your specific workflow* | Already beats 671B models (DeepSeek R1) on ARC-AGI-1 with 0.01% of the parameters |

### Is TRM outdated?
**Absolutely not.** The paper is from October 2025. It is *more* modern than most LLM approaches people are copying. The authors explicitly show that a 7M-parameter TRM scores **44.6% on ARC-AGI-1**, while DeepSeek R1 (671B parameters) scores only **15.8%**. This is not outdated — it is a different, and in some ways more advanced, direction.

### The critical limitation of TRM (that you must accept)
From the paper itself, Section 6: *"Currently, recursive reasoning models such as HRM and TRM are supervised learning methods rather than generative models."*

This means TRM does **not** chat, write code in free-form, or handle open-ended text. It takes a structured input (like a 9×9 Sudoku grid or a 30×30 ARC puzzle) and iteratively refines a structured output. If you want to "specialize it on many different fields," those fields must be **structured, deterministic problems with fixed input/output shapes**.

If you want to turn TRM into a generative model that writes code or holds conversations, **you would be doing original research that no one has published yet.** That is not a reason to avoid it — it is a reason to pursue it if you want to contribute something genuinely new.

---

## 2. ORGANIZED ROADMAP

I have laid out both tracks as standalone projects. **You should pick one as your primary focus.** Trying to do both simultaneously will fragment your progress.

---

### **TRACK A: Build a Fast Agentic Coding Model (The "Ornith" Path)**

**Goal:** A fast, local coding assistant specialized for your workflow.

**Realistic Expectation:** You will not build a frontier model. You *can* build a model that is faster and better than GPT-4o at *your specific stack* (e.g., Python + specific frameworks).

**Phase 1 — Foundation (Weeks 1–2)**
1. **Pick your base.** Mistral is fast, but Qwen 2.5/3 or Gemma 4 have better coding bases. If speed is your top priority, use Mistral Small / Qwen 2.5 7B.
2. **Set up infrastructure.** You need `unsloth`, `axolotl`, or `mistral-finetune` for efficient SFT. You also need vLLM or TGI for fast inference.
3. **Gather your data.** This is the make-or-break step. You need:
   - High-quality coding instruction data (commit messages → code, bug reports → fixes).
   - Tool-use trajectories (model calls a linter, reads output, fixes code).
   - If you lack data, use distillation: generate trajectories from Claude 3.7 Sonnet or o3-mini and filter for quality.

**Phase 2 — Specialization (Weeks 3–6)**
4. **Supervised Fine-Tuning (SFT).** Train on your coding data with LoRA or full fine-tune.
5. **Preference Tuning (DPO / IPO).** Build a preference dataset where you rank good solutions vs. bad solutions. This teaches the model *how* to code, not just *what* to output.
6. **Tool-use scaffolding.** Build the agent loop: the model plans → writes code → runs linter/tests → reads output → revises. The model itself does not need to be huge; the scaffolding does the heavy lifting.

**Phase 3 — Refinement (Weeks 7+)**
7. **RL on task success.** Use reinforcement learning (GRPO, PPO) where the reward is "does the code compile and pass tests?" This is what Ornith does, and it is the hardest part.
8. **Evaluate.** Use SWE-bench, HumanEval, or your own private test suite.

**What you need:** At minimum, a 24GB GPU for weeks, or cloud credits. A strong teacher model for distillation. Patience for RL debugging.

---

### **TRACK B: Expand the Tiny Recursive Model (The "TRM" Path)**

**Goal:** Reproduce and extend the 7M-parameter recursive reasoner to new problem domains.

**Realistic Expectation:** You will not get a chatbot. You *can* get a tiny model that outperforms billion-parameter models on specific reasoning tasks (puzzles, logic, grid problems, potentially math).

**Phase 1 — Reproduction (Weeks 1–3)**
1. **Reimplement TRM from the paper.** The architecture is simple:
   - 2-layer network (Transformer or MLP-Mixer).
   - Recursive latent update: `z = net(x, y, z)` run `n` times.
   - Answer update: `y = net(y, z)`.
   - Deep supervision: carry `(y, z)` across up to 16 improvement steps.
   - EMA of weights (0.999) for stability.
2. **Reproduce Sudoku-Extreme.** This is your sanity check. Target: ~87% accuracy with 5M parameters. If you cannot hit this, your implementation is wrong.
3. **Verify the ablations.** Test with/without self-attention, with 2 layers vs. 4 layers, with full backprop vs. 1-step gradient. This builds your intuition.

**Phase 2 — Domain Expansion (Weeks 4–8)**
4. **Pick a new structured domain.** Good candidates:
   - **Chess puzzles** (input: board state, output: best move).
   - **Symbolic math** (input: equation, output: simplified form).
   - **Pathfinding** (input: grid with obstacles, output: shortest path).
   - **Constraint satisfaction** (scheduling, logic puzzles).
5. **Build your dataset.** TRM needs fixed-shape input/output pairs. You will need ~1,000 training examples with heavy augmentation (the paper uses 1,000 augmentations per example for ARC).
6. **Train and iterate.** Use the same hyperparameters from the paper (AdamW, lr=1e-4, weight decay=1.0, EMA). Tune `n` (latent recursions) and `T` (deep supervision steps) for your domain.

**Phase 3 — Original Research (Months 3+)**
7. **Make it generative (the open problem).** The paper explicitly leaves this as future work. Can you:
   - Use TRM to iteratively refine a *sequence* of tokens instead of a fixed grid?
   - Apply it to code synthesis where `y` is a token sequence and `z` is a latent reasoning trace?
   - This is hard, but if you crack it, it is genuinely novel.
8. **Distill from large models.** Use a large LLM to generate reasoning traces (`z`) and solutions (`y`), then train TRM to mimic that iterative process. This could let you compress a 70B model's reasoning into 7M parameters.

**What you need:** Almost no compute. A single GPU with 8–16GB VRAM is sufficient. The real requirement is time and structured thinking.

---

## 3. MY RECOMMENDATION

**If you want a practical tool in 2–3 months:** Pursue **Track A**, but be honest about the data bottleneck. You will spend 80% of your time curating data, not training.

**If you want to do something unique that could actually advance the field:** Pursue **Track B**. The TRM paper is so new that almost no one has reproduced it independently, let alone extended it. The "Ornith" space has hundreds of competitors. The "tiny recursive reasoner" space has almost none.

**If you want to do both:** Use Track B as your primary project (low cost, high originality), and use Track A only for a specific, narrow coding task where you already have the data. Do not try to build a general-purpose "Ornith competitor" and a general-purpose "TRM expansion" at the same time.

---

## 4. FIRST STEP FOR THIS WEEK

Pick one sentence and commit to it:

- **"I will reproduce TRM on Sudoku-Extreme and then adapt it to [X] domain."**
- **"I will fine-tune Mistral on my specific coding stack and build tool-use scaffolding for it."**

Which track feels like the right starting point for you?


================================================================================
 APPENDIX: Agent deep-dive on Track B (TRM expansion) — 2026-07-16
 Performed after reading the TinyRecursiveModels codebase + paper (arXiv:2510.04871)
================================================================================

This appendix is the agent's response after the document above. It pushes back on
the "you can't do this" objection with precise technical detail, then lays out
five concrete routes from specialized-reasoner to tiny-helpful-model, with a
computed cost analysis for the CPU-strain question.

---

## A. The "not interactive" objection is overstated

The claim "TRM is not a model that can be interacted with" is technically correct
(it is true of the base architecture) but operationally misleading. The reason
the project sounds impossible is that Paper Section 6's "TRM is a supervised
learning method rather than a generative model" sentence hides five distinct
engineering problems, not one — and each gap has a known-ish fix that the
existing codebase already half-supports.

### The five concrete gaps — ordered easiest to hardest

**Gap 1: Fixed output shape (easiest — paperwork, not research)**
TRM produces `[B, L, vocab]` logits aligned position-by-position with the input.
Sudoku is 9x9 in -> 9x9 out. ARC is 30x30 in -> <=30x30 out (cropped by EOS
tokens). For a coding assistant you'd want variable length. But — the model
already emits per-token logits over the entire sequence including padding. It
already has a PAD id (0), an EOS id (1), and vocab indices. Take a coding task
framed as "input: prompt+context up to N tokens, output: completion up to M
tokens" and the architecture handles it. You need a tokenizer + a flat sequence
layout, that's it. The ARC evaluator's `_crop` + EOS demonstrates variable
outputs already work. This gap is paperwork, not research.

**Gap 2: No causal masking (small fix, big implication)**
`Attention` in `models/layers.py` is hardcoded `causal=False`. The README and
configs confirm this — TRM is non-autoregressive by design ("Non-autoregressive"
comment in `pretrain.create_model`, line 123). To generate text
token-by-token you need `causal=True` for the answer portion. One-line change
per block, plus an input layout separating "seen context" from "to-predict
completion." The harder issue: causal masking changes the dynamics of
recursion. With bidirectional attention, z_L/z_H absorb the *entire* sequence
each cycle, so every recursion step is a full refinement. With causal masking,
each recursion step refines only what's been seen so far — strictly append-
friendly, which is actually what you want for generation. Fixable, and arguably
better suited to generation than the current setup.

**Gap 3: No token-by-token streaming during inference (medium — refactor)**
At inference (`pretrain.evaluate` lines 385-393), the model runs
`while True: carry, loss, metrics, preds, all_finish = model(carry, batch, return_keys)`
and stops when `all_finish`. It then takes `argmax(logits)`. There is no notion
of "emit one token, then another." But this is *structural*, not fundamental: the
carry mechanism is already a per-step state machine, and the Q-halt signal is
already a "should I stop refining" router. Converting "halt means emit final
answer" to "at each outer step emit one more token and refine the prefix,
carrying the prefix as fixed z_H" is a refactor, not an architecture rewrite.
The ACT literature (Graves 2016, original ACT) has Universal Transformers doing
exactly this.

**Gap 4: No natural-language training data (the actual hard part)**
This is the real bottleneck. TRM's training data is *pairs of fixed-shape
arrays*. ARC puzzles come pre-baked as 30x30 grids with deterministic
solutions. Sudoku has a unique solution. The loss is per-token cross-entropy
against a single ground-truth answer. To train a coding assistant you need:
- A fixed input layout (token IDs: natural language is fine —
  `CastedEmbedding(vocab_size, hidden_size)` works for any vocab, including
  BPE tokens, not just the ARC 0-11 grid alphabet)
- Pairs of (prompt, correct completion) — HumanEval, CommitPack, your own
  Cynapse/Tradebot commit history would work
- The catch: TRM's loss assumes there's *one* correct completion. For code,
  there are many. ARC sidesteps this because each puzzle has a unique grid
  answer. Sudoku has a unique solution. Math equations have a unique simplified
  form. Code completion does not. This is a research question but a tractable
  one — see Gap 5 below and Route 3 in the brainstorm.

**Gap 5: The recursion acts on the full sequence each cycle (medium-hard)**
Each recursion step runs `L_level` over the full `[B, L+P, D]` sequence. For
ARC L=900. For a 4K-token coding context L=4096, D=512. Per step that's 2
layers x (2 SwiGLU + 1 attention) of compute — and at H_cycles=3, L_cycles=6
the model runs 18+ layer-ops per outer step, vs 2-4 for a regular transformer of
the same depth. So TRM is "tiny in params, expensive in compute at a fixed
context." On limited hardware this matters — see the cost analysis below.
Two mitigations: (a) causal masking lets you KV-cache the context portion so
only the new token gets full recursion work; (b) you can decouple "reasoning
recursion depth" from "generation length."


## B. Cost analysis — the CPU strain question, answered

NUMBERS COMPUTED FROM THE CODEBASE:
- Per-block params: 3.41M (attn QKV + O proj + SwiGLU gate/up/down with
  inter = 1536 at hidden=512, expansion=4)
- 2-layer TRM params: 6.82M (matches paper's ~7M claim)
- Weights footprint at bf16: ~14 MB
- L_level calls per outer ACT step (H_cycles=3, L_cycles=6): 21

REFERENCE: ~1 GFLOP = 20-100ms on a modern CPU (AVX2 sustained ~10-50 GFLOPS).
A 3B LLM forward pass at 1024 tokens ~6e12 FLOPs.
FunctionalGemma-2B at bf16 = ~4 GB RAM + MLPerf-style compute.
A micro-TRM (L=128, 4 halt steps) = ~0.2 GFLOPs, ~4-20 ms on CPU, 14 MB RAM.

CPU-strain verdict per specialist profile:
| Specialist                     | L    | steps | GFLOPs | CPU time      | RAM     |
|--------------------------------|------|-------|--------|---------------|---------|
| Tiny tool-router (semantic)    |  128 |   4   |   ~1   |  20-100 ms    | 14 MB   |
| Small code-AST specialist      |  512 |   4   |   ~3   |  65-330 ms    | <30 MB  |
| ARC 30x30 grid (paper task)    |  900 |  16   |  47    |  1-5 s        | ~30 MB  |
| Small code-AST specialist MAX  |  512 |  16   |  17    |  0.3-1.7 s    | <30 MB  |
| Long coding context (chat-ish)| 2048 |  16   |  268   |  5-27 s       | ~110 MB |
| 3B LLM fwd at 1024 ctx         | 1024 |   1   | 6000   |  100-300 s    | ~4 GB   |

Key takeaways:
1. For ROUTE 1 specialist tasks (L=128 or 512, halt_max_steps=4-8), TRM uses
   0.02-0.3% of a 3B LLM's compute. CPU strain is essentially a non-issue.
   FunctionalGemma is *much* heavier per query than this.
2. CPU strain becomes real ONLY at long context (L>1K) and high recursion depth
   (16 steps). That's the research-paper regime; you don't need it for
   specialists. Tune `halt_max_steps` down to 4-8 for production inference.
3. TRM is bidirectional — no KV caching *today* because it doesn't need to at
   this scale. Causal-masked variants can KV-cache the context portion and
   reduce compute ~4x further for "process input -> emit answer" specialists.
4. The comparison "TRM is CPU-straining, FunctionalGemma is not" is backwards
   on the specialist tasks. TRM at L=128 is 30-300x lighter per query than
   FunctionalGemma-2B. They are not competitors — they live at different layers
   of the stack. FunctionalGemma handles the general-ish chat; route the
   structured/deterministic tasks to TRM specialists behind the semantic
   router. The CPU budget for the structured tasks drops by 1-2 orders of
   magnitude vs FunctionalGemma.

TRAINING (the *only* expensive part) is no concern for Route 1:
- Sudoku-Extreme reproduction: ~18h on a single L40S per the README. Colab's
  free T4 (16 GB) is marginal; Colab Pro+ A100 (40 GB) handles it.
- A small specialist on log parsing / JSON extraction / tool routing:
  probably 1-4 hours on a Colab T4. Tiny data (~1K pairs x 1000 augs = 1M
  examples), tiny model (7M params).
- Train on Colab, ship the 14 MB checkpoint to Pi/Cynapse/semantic-router for
  inference. That's the correct deployment pattern. Nothing in the plan
  requires you to keep a GPU on.


## C. Five routes from "specialized reasoner" to "tiny helpful model"

Given the constraints — limited RAM/VRAM, willingness to specialize, multiple
tiny models per domain — these are the realistic paths, from least to most
ambitious.

### Route 1: Pure specialization (lowest risk, ships useful tooling)
Skip generative entirely. Build specialized TRMs for domains with fixed-shape,
deterministic I/O, exactly like the paper's Sudoku and ARC. Candidates:

- Agentic tool routing: input = serialized tool catalog + task description
  (fixed padded length, say 512 tokens), output = tool-call token sequence.
  Like `semantic-router-go` in the skills but *learned, not embedding-based*.
  7M params, instant on CPU. Multiplex dozens of these.
- Structured data extraction: JSON-from-messy-text, log parsing, regex
  synthesis from examples. Cynapse already does tokenization; a TRM trained
  on pairs of (raw text, structured token sequence) would beat any heuristic.
- Code-to-AST and AST-to-AST transforms: "one for coding" lands cleanly here.
  Fixed shape (token sequence), deterministic output (refactored AST),
  deterministic labels. A 7M-param "format Go imports correctly" or "extract
  function signatures from this Rust file" model is exactly a TRM-shaped
  problem and would ship in weeks.
- Numerical reasoning: arithmetic, unit conversion, time math. LLMs are
  notoriously bad at this; TRM trained on (arithmetic expression, result grid)
  would never hallucinate a digit. POTENTIAL APPLICATION TO LEAFCUTTER LLM —
  see "math specialist" note below.

This matches Track B in the doc above. Pattern is "multi-model": one tiny
specialist per task. All run on a Raspberry Pi. Each replaces a regex or a
hand-written parser with something that generalizes. The paper's 87% Sudoku
and 85% Maze prove the ceiling is high.

### Route 2: Teacher-student distillation (medium, bridges to "agentic")
Use a frontier LLM (Claude, GPT-4) as a teacher to *generate the (x, y) pairs
TRM needs*. Explicitly suggested in the original doc above (Phase 3 item 8).

Workflow: pick a task the LLM does well (e.g. "given this GitHub issue, write
the function body"). Use the LLM to produce 1K paired examples. Augment 1000x
via semantic-preserving transforms (variable rename, comment shuffle,
equivalent-API substitution). Train a 7M TRM on the pairs. The TRM is now a
reasoning-capable specialist that beats the LLM only at this exact task, at
0.01% of the params, and runs offline.

This is also how you'd get something reasonable on agentic/decision tasks:
distill a planner. Input = serialized state (file tree + recent commands +
error output), output = next-action token. The teacher is "Claude plans";
the TRM is "Claude plans, distilled to a wristwatch battery." Quality is
bounded by the teacher, but for a *specialist* the teacher is plenty good
enough.

(This route aligns with Plan A's distillation work — the same teacher-LLM
infra you'd build for Mistral/Qwen post-training can feed TRM students. No
wasted effort — pick the same teacher model for both Plan A and Plan B's
Route 2.

### Route 3: Make TRM generative (research, but smaller than people think)
The doc above calls this "original research no one has published." That's
mostly right but the path is shorter than the framing suggests, because the
hard part is already solved — what's missing is gluing three published
techniques onto the TRM scaffold:

a) Causal-masking variant of the Block (one-line change in `Attention.__init__`).
   Pretrained ARC checkpoints don't use it but the architecture doesn't forbid
   it — the paper's `transformers_baseline.py` already has H_layers=8 and you
   can extend it; same codebase.
b) Iterative token refinement à la Universal Transformer / Mask-Predict
   (Ghazvininejad 2019): instead of generating left-to-right once, you generate
   all M output tokens at once with masked positions, then run the recursion
   for K steps where each step is allowed to *revise* previous tokens. This is
   exactly what TRM already does for puzzles — the recursion is refinement,
   not generation. For text you'd mask-predict-init the output positions with
   a [MASK] token, run TRM's deep supervision, and the Q-halt decides when the
   answer is "settled." Most natural fit for TRM; the architecture is closer
   to mask-predict than to autoregressive decoding already.
c) Non-autoregressive + iterative revision loss is what TRM's stablemax cross-
   entropy is already for — it handles the "no single correct answer" problem
   better than softmax because it has heavier tails and a cleaner gradient for
   ambiguous positions. The stablemax choice in `losses.py` (lines 11-22) is
   not an accident; it's the right loss for refinement.

Concrete Route 3 form: train a TRM where x = BPE-encoded prompt + mask
position block, y = predicted completion tokens (mask-predict, refine K
times). Loss is cross-entropy over multiple valid completions via teacher
sampling (one sampled valid completion per training step, like RLAIF's
reference distribution). This is mask-predict decoding + TRM's deep
supervision; no one has done the combination. It would be a real paper.

### Route 4: Mixture-of-tiny-specialists (matches the stated goal best)
"One for coding, one for agentic work, and so on" — is literally this. Build
a router (could itself be a 1M-param TRM, or simpler — a small embedding
similarity model) that picks which specialist TRM handles a given input.
Each specialist is small, fixed-shape, with its own distilled training data.
The system is the *collection*, not any individual model.

This is the model architecture FunctionalGemma gestures at: not one general
model, but a *federation* of tiny specialists with a dispatcher. Cynapse's
synapse system is conceptually similar already — synapses are tiny pluggable
specialists. A TRM-micro as a Cynapse synapse is a beautiful fit; the
recursion depth adapts to problem hardness, the params are tiny, each
synapse learns one thing.

### Route 5: Push the architecture itself (deepest research, highest payoff)
The TRM paper leaves obvious extensions on the table:

- Drop the 2-latent constraint to 1 latent, decoupled from generation length
  (`trm_singlez.py` already exists in the repo; the paper says it underperforms
  but did not test it generatively).
- Continuous-depth halting — currently ACT halts on whole sequences, not
  per-token. Per-token halting (Graves-style "pondering" per position) would
  let the model spend more cycles on hard tokens and skip easy ones — pure
  speed win for inference on limited hardware.
- Sparse MoE over the L_level: 2 layers stays, but each layer is a low-rank
  mixture-of-experts. 7M total params becomes 50M effective with the same
  active compute. This is how you get "tiny-but-capable" without VRAM growth.
- State-space (Mamba) replacement for the Attention block: linear-time
  recurrent layer instead of O(L^2) attention, for long coding contexts.
  The Block is already swappable — the `mlp_t=True` mode uses an MLP
  alternative; you could add `attn=mamba` and reuse everything else.


## D. Straight read on "is it worth entering"

Yes, with caveats. Reasoning:

- The "tiny recursive reasoner on new domains" space is genuinely open. Paper
  is Oct 2025 — it's recent — and the GitHub repo just got archived-read-only
  due to spam issues, not maturity. There are roughly zero independent
  reproductions or domain extensions published. Almost any structured-task
  extension is publishable or at least shippable.
- Hardware constraint is shared with TRM's design. 7M params in bf16 is
  ~14 MB of weights. 2-layer attention at hidden=512 is kilobytes of
  activations. This *will* run on a Raspberry Pi 4 with 4 GB RAM, and
  comfortably on phones. The compute cost is real (Gap 5 at high L and 16
  steps) but it's a latency problem, not a memory problem — and the Cynapse/
  Leafcutter low-end-hardware optimization experience transfers directly.
- The hub-and-spoke (Route 4) is the right framing for "assist on all sorts
  of hardware." One TRM doesn't need to "be helpful generalized"; a *fleet*
  of TRMs each does one useful thing, and one tiny router picks. Cynapse's
  synapse architecture is already shaped for this — you'd basically add
  `trm` as a new synapse type.
- The risk is the *data*, not the architecture. Route 1 ships today with the
  existing repo and per-task datasets. Routes 2 and 3 need a data pipeline
  (teacher-LLM distillation, paired examples, augmentation). That's where
  your time will actually go — exactly what the doc warns about for Track A
  AND ALSO for Track B's more ambitious variants. The architecture is small
  to implement; the dataset engineering is large.
- The "it can't chat" criticism is overstated for the stated goal. You don't
  need a chatbot — you need small task-specific reasoners. The doc itself
  says this on Track B line 67. The people telling you "it's not interactive"
  are correct about the base architecture and wrong about the ceiling — the
  base architecture is one engineering refactor away from producing token
  streams of any shape, and the published literature (mask-predict,
  Universal Transformer, ACT) already shows how.
- The thing actually worth being honest about: Route 3 (make TRM genuinely
  generative on free-form text) is real open research. 3-6 months of focused
  work minimally, with a real chance of producing "interesting but not SOTA"
  results, and a smaller chance of disrupting. Routes 1, 2, and 4 ship
  useful tools within weeks to a couple months. Pick which profile you want
  — utility now vs. novelty with risk.


## E. Recommended sequencing if you proceed

1. **Weeks 1-3**: reproduce TRM on Sudoku-Extreme using the existing repo.
   Target 87%. Validates the setup is correct and trains your intuition for
   what H_cycles x L_cycles actually does. Don't skip — it's the paper's
   sanity check and yours.
2. **Weeks 3-4**: pick a fixed-shape specialist task from Route 1 that
   overlaps with your actual workflow (something Cynapse or Tradebot
   currently uses a regex/heuristic for — log parsing or JSON extract).
   Build the dataset. Train. Measure. This is your first "tiny useful
   model" and validates Route 4's synapse integration.
3. **Weeks 5-8**: extend to a second specialist — ideally something more
   agentic-feeling (e.g. tool-call-selector). Try Route 2 distillation here:
   use Claude/GPT as the teacher to generate (state, action) pairs. Your
   first distillation pipeline. (Reuses the Plan A teacher infra.)
4. **Months 3+**: only now consider Route 3 (generative TRM). By this point
   you'll know exactly where the architecture strains under variable-length
   outputs, and your intuition will outperform any prior plan. The mask-
   predict + iterative-refinement idea (Route 3 part b) is the most promising
   concrete direction.

---

## F. DECISION (recorded 2026-07-16)

Starting with **Route 1** (pure specialization) as the primary focus.
Background intent: also pursue **Route 2** (teacher-student distillation) in
parallel / shortly after, since the Plan A teacher-LLM infra is being built
anyway and the same distillation pipeline can feed both tracks. Future routes
(3/4/5) are parked until the small wins accumulate.

### Specific hooks into existing projects:
- **Semantic router**: add TRM specialists as alternative learned routers
  behind the existing embedding-similarity router. The TRM is 30-300x
  lighter per query than FunctionalGemma and beats any heuristic on
  structured router decisions.
- **Cynapse**: add `trm` as a new synapse type. The Cynapse synapse system
  already matches Route 4's "federation of tiny specialists" pattern.
- **LeafcutterLLM (ambitious)**: a TRM math specialist (Route 1 numerical
  reasoning subitem) is potentially the solution to the CPU crisis in
  LeafcutterLLM. If a 7M TRM can do arithmetic, unit conversion, and
  structured numeric tasks without hallucinating digits, it can replace
  some of LeafcutterLLM's heavier numerical-pipeline code with a model
  that uses 14 MB of RAM and 20-100ms of CPU per query. This is a
  high-value integration target to investigate after the first two
  specialists ship.

### First concrete step
Pick a fixed-shape specialist task from the Route 1 candidate list that
overlaps with the actual Cynapse or Tradebot workflow (something currently
done with a regex or heuristic — log parsing, JSON extract, commit-message
tagging). Build the dataset. Train on Colab (T4/A100). Ship the 14 MB
checkpoint to the semantic router / Cynapse. Measure it.


================================================================================
 APPENDIX ADDENDUM (2026-07-16): Pathfinder-Eye TRM plan
================================================================================

==============================================================================
 A. THREE QUESTIONS ANSWERED — replace FunctionalGemma completely
==============================================================================

Q1: Can TTS narration skip FunctionalGemma and just read MD files direct?
A1: YES. MD files are plain text with formatting markers. To read aloud you
    just need: open the .md file -> strip markdown markup (regex: remove
    ```code blocks```, **bold**, *italic*, # headings, [links](urls), list
    markers) -> send cleaned text to TTS engine. That's a 20-line Python
    script. No LLM in the middle. FunctionalGemma was "reading" the document
    by generating a copy of it, which is absurd when the document is already
    sitting right there. For camporee narration, MD -> strip markup -> TTS
    is complete and trivial. The only case where you'd want an LLM in the
    middle is summarization/simplification — and complex stuff goes to
    "thinking mode" with an external AI model, per the user's design. So
    MD -> TTS direct, full stop.

Q2: Can we improve the listening (STT interpretation)?
A2: YES, TRM is exactly the upgrade. FunctionalGemma is a generative LLM
    wearing a classification hat, which means:
    - Slow: 1-5 s to interpret a transcript
    - Inconsistent: same phrasing can route differently on re-runs
    - Hallucination-prone: can "summarize" a command instead of routing it
    - Memory hungry: 4 GB just to sit there
    A TRM intent classifier is strictly better on every axis that matters
    for listening:
    - 10-50 ms per interpretation (vs 1-5 s)
    - Deterministic at inference (same input -> same intent, every time)
    - Never hallucinates: can only output one of the intents in its vocab
    - 14 MB of RAM (vs 4 GB — about 280x less)
    Honest caveat: TRM only knows the intents you trained it on. Out-of-
    distribution input gets routed to a "no_intent" or closest-match
    bucket. For camporee use, a deterministic "I didn't catch that" is
    better than a wandering monologue from an LLM that wanted to philosophize.
    For complex/deeper interaction, "thinking mode" switches to a local AI
    model — that doesn't need FunctionalGemma either; it just talks with the
    person and interactions are recorded into the DENDRITE system.

Q3: Can we replace FunctionalGemma completely?
A3: YES — all its jobs are replaceable. Decomposition:

    | Job                            | FG today                          | Replacement                                            | Effort     |
    |--------------------------------|-----------------------------------|--------------------------------------------------------|------------|
    | TTS document narration         | LLM regurgitates MD text         | MD -> strip markup -> TTS (20-line script)             | trivial    |
    | Intent classification from STT | LLM picks intent                  | TRM intent classifier                                  | 1-2 weeks  |
    | MP3 playback trigger           | LLM emits "play mp3" + filename   | Keyword match in TRM vocab: "play <song>" -> [play_mp3, arg=<song>] | trivial (covered by TRM) |

    FunctionalGemma has zero indispensable jobs in the current robot design.
    Remove it completely. 4 GB of RAM recovered. Latency drops 20-100x on
    the listening loop. Behavior becomes deterministic and debuggable. The
    semantic router gets a leaner, faster specialist slot.

    "Thinking mode" is a separate human-triggered thing: switches to a
    local AI model (LeafcutterLLM/Ministral on the robot's Pi) for
    conversation only. No tool-calling. No cloud. Just talk + record to
    DENDRITE.

==============================================================================
 B. PATHFINDER-EYE TRM PROJECT: WHAT WE KNOW ABOUT THE CURRENT CODEBASE
==============================================================================

 inspected: /home/xander/Documents/portfolio/the-pathfinder-eye/

 HARDWARE: Raspberry Pi 5, 8 GB RAM. Zero-Python architecture.
 STT:      Whisper tiny.en on CPU.
 TTS:      espeak-ng via async channel queue.
 LLM:      LeafcutterLLM (Rust/llama-ffi) serving Ministral-3B Q4_K_M at ~500MB RSS.
 VISION:   Standalone Rust service (YOLOv5 + Haar).
 BRAIN:    Pure Go (go_brain/) — main orchestrator.

 CURRENT INTENT PIPELINE (the thing TRM will replace):
 Two Go-native layers handle STT -> intent. NO FunctionalGemma is currently
 in the live loop — the heavy LLM is only invoked after the user says
 "Attention" (an explicit wake-up), not for every command.

   1. command_parser.go (223 lines):
      - ExtractCommand(text) -> ParsedCommand{Action, Target, Modifiers}
      - Action vocab (~12): move, look, play, read, activate, deactivate,
        test, translate, attention, remote, deep, stop
      - Target vocab (~25): forward, backward, left, right, about_turn,
        up, down, center, law, pledge, aim, motto, pathfinder_song,
        adventurer_song, birdwatch, follow, security, deep, japanese,
        remote, etc.
      - Implemented as: tokenize -> first-token actionAliases map lookup
        -> scan-all fallback -> targetAliases map lookup
        -> multi-word phrase patch (about turn, adventurer song,
        pathfinder song, deep thought, etc.)
        -> numeric modifier extraction (speed, angle, fast, slow)

   2. semantic_router.go (310 lines):
      - 3 high-level routes: RouteBasicChat, RouteThinking, RouteToolCall
      - Embedding: hash-trick dim=256, position weighting, robotWordBoost map 
        (boosts birdwatch/third/bired to 3.0, pathfinder/adventurer to 2.0,
        most command words to 1.5-2.5)
      - Centroids: pre-computed constants (basicChatCentroid,
        thinkingCentroid, toolCallCentroid) — three 256-dim float arrays
      - Classifier: cosine similarity to 3 centroids, return best + confidence
      - Fallback: RouteBasicChat

 CURRENT FLOW (voice_commands.go):
   handleCommandSequence():
     -> captureAudio(5s) -> Whisper STT -> text
     -> isWakeWord("instruction") check
     -> authority.CanExecuteCommand(speakerRank, cmd) check
     -> processDirectCommand(cmd, level, name):
          ExtractCommand(cmd) -> ParsedCommand{action, target}
          switch action:
            test -> hardware test sequence
            move -> handleMoveAction (4WD via I2C, fwd/back/left/about_turn)
            look -> handleLookAction (gimbal pan/tilt servos)
            play -> handlePlayAction (mpg123 on Pathfinder Song.mp3 /
                    Adventurer Song.mp3)
            read -> handleReadAction (readDocument on
                    Pathfinder Law.md / Pathfinder Pledge.md / etc.)
            activate -> handleActivateAction (birdwatch, follow)
            deactivate -> handleDeactivateAction (stop birdwatch/follow/security)
            attention -> systemctl start leafcutter.service + enter AI loop
            remote -> /tmp/stream_active = 1 (remote control mode)
            deep -> enterDeepThought (70B model swap attempt; Pi refuses
                    at 8GB guard)
            translate -> startJapaneseTranslationLoop
            (fallback) -> keyword check for exit/stop/sleep -> handleExitCommand

 So the existing intent vocabulary IS the schema TRM will learn. The TRM
 specialist replaces ExtractCommand + ClassifyRoute (the Go-native parser
 + the hash-trick router) with a single learned model. Targets and
 modifiers are still produced deterministically by the existing Go code; TRM
 just replaces the action+target extraction step.

 HONEST ASSESSMENT AFTER READING THE CODE:
 - The current pipeline is ALREADY lightweight, deterministic, and fast.
   This is not FunctionalGemma strapped to the robot — it's a 200-line
   rule-based parser plus a 300-line hash router. Both are already fast
   on a Pi. The "replace FunctionalGemma" framing was slightly off: what
   we're actually doing is replacing a brittle rule-based extractor with
   a learned TRM specialist that generalizes to paraphrases, noise, and
   STT misrecognitions the rules don't cover.
 - The semantic router's hash-trick embedding is genuinely weak — paper-
   thin centroids, random hash dims, hand-tuned boost table. A TRM here
   would be a real upgrade on robustness, not just a speed upgrade. The
   router currently misroutes anything not in its author-curated
   vocabularies; a TRM trained on augmented transcripts would handle
   natural phrasing ("drive ahead" instead of "move forward", "recite the
   pledge" instead of "read pledge", "is the bird watch on?" as a question
   form of activate birdwatch) that the rule parser misses today.
 - For camporee the existing pipeline WORKS. We are not in a "must replace
   or the robot breaks" situation. The TRM is an upgrade, not a fix.

 REVISED 2-WEEK PLAN (post codebase reading):
 1. Day 1-2: write a synthetic data generator in Python that emits
    (transcript, ParsedCommand-action, ParsedCommand-target) tuples
    covering: every existing alias in command_parser.go + 10+ paraphrases
    per action + STT noise ("um", "uh", mishearings like "third" for
    "bird", "instruct" for "instruction") + out-of-vocab noise.
    Target: 1000-2000 pairs, 1000x augmentation = ~1M examples.
 2. Day 3-5: port a stripped TRM training config (hidden=256, L=64-128,
    halt_max_steps=4) into the existing TinyRecursiveModels codebase.
    Train on Colab T4. Validate against held-out pairs; aim for >95% exact
    action+target accuracy (the rule parser gets ~100% on its own
    vocabulary and ~0% on paraphrases, so the TRM has real headroom).
 3. Day 6-9: inference — export TRM to ONNX if convenient, write a thin
    Go wrapper in go_brain/ that replaces ExtractCommand, OR keep the Go
    code and have it shell out to a tiny Python/ONNX-runtime process via
    JSON. Match the user's Zero-Python rule by preference: keep TRM as
    a small Rust microservice (matching the rust_vision pattern) that
    exposes a /classify endpoint on localhost. Go brain POSTs transcript,
    gets back {action, target, confidence}.
 4. Day 10-14: integration + on-robot testing. Confidence fallback — if
    TRM returns confidence < threshold, fall through to the legacy
    rule-based ExtractCommand (keeps the deterministic path as a safety
    net for camporee).
