# Export rendering throughput

## Workload and method

The reported problem was a 31-minute Reel estimating 2.5 to 3 hours to export.
Its 39 segments use a GPU Stage with captions, rising headers, Rolling Numbers,
and some Value Tokens. Two representative segment plans were copied into ignored
`target/perf-baseline/` for local experiments. Their audio placements were omitted;
their visual actors and timelines were retained. The complete Reel was not timed.

The baseline is revision `46fd6121d0c2067f22a176e2187924a9914e9453`.
The candidate is this checkout's uncommitted rendering change. Both executables
use the release profile, 1920×1080, 60 FPS, the `opencode` theme, and identical
export settings. Stage exposure retains 24 shutter samples. Editor exposure
retains 16 samples before one second and eight thereafter. Equal visual samples
still merge exactly as before. Neither motion nor encoder settings changed.

Measurements ran on an Apple M1 Max with Metal, Rust 1.98.1, and FFmpeg 9.0.2.
Unrelated renders were already active and were left running. The comparison
alternates executable order, excludes one warmup for each executable, and reports
median process wall time, including startup and FFmpeg. These are local
comparisons under background load, not isolated hardware limits.

`scripts/bench-render.py` retains each command, timing, renderer progress output,
plan hash, and decoded-frame comparison under the chosen ignored output directory.
The current script also records binary hashes and host details. Early measurements
precede that metadata addition. H.264 is lossy, so raw shutter snapshots supplement
decoded-video equality.

## Results

| Export window | Frames | Runs per executable | Baseline median | Candidate median | Speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| Long Reel opening, 0–2 s | 120 | 3 | 25.346 s | 11.179 s | 2.27× |
| Long Reel rising header, 0.6–0.8 s | 12 | 5 | 3.045 s | 1.405 s | 2.17× |
| Long Reel mixed Value Tokens, 0.6–0.8 s | 12 | 3 | 3.049 s | 1.658 s | 1.84× |
| Long Reel settled overlay, 5–5.2 s | 12 | 3 | 0.487 s | 0.489 s | 1.00× |
| Moving Stage without CPU overlays, 1–1.2 s | 12 | 3 | 0.396 s | 0.393 s | 1.01× |
| Hero editor opening, 0–0.1 s | 6 | 3 | 1.770 s | 1.792 s | 0.99× |
| Hero editor transition, 3–3.2 s | 12 | 3 | 8.787 s | 8.891 s | 0.99× |

Every row's decoded frames were identical between executables. The gain is in
CPU-heavy Stage overlays. The tested editor exports, settled overlays, and Stage
alone are essentially unchanged. The Value Token case deliberately exercises the
full-frame fallback rather than the new bounded overlay region.

The two-second opening remains slower than real time. It contains expensive
blurred header entrances and is not representative of an entire 31-minute Reel.
These measurements do not establish a new whole-film render time or parity with
fframes. A complete before/after Reel export would answer that question.

## Verification

`cargo test --workspace` passed 381 tests after the FPS follow-up, with 52 GPU or manual benchmark tests
ignored by default. The targeted GPU test
`sparse_stage_overlays_match_full_frame_exposure` also passed explicitly in release
mode. It compares regional and full-frame accumulation for fractional placement,
offscreen ink, typing carets, chips, header reflections, masks, and rolling text.
Pure differential tests compare optimized text and projected overlays against
their original compositors. Exposure tests compare floating-point bits and bytes
across parallel chunks, partial chunks, overlapping regions, and unusual weights.

All 13 baseline shutter snapshots had zero changed pixels after the final
optimization: three moving Stage frames, three editor frames, four opening or
settled Reel frames, and three mixed Value Token frames. The final opening shutter
frame was also inspected at full scale. GPU, bundled fonts, and FFmpeg were
available for artifact verification.

`cargo fmt --check` and `git diff --check` passed. Strict workspace Clippy on Rust
1.98.1 fails on the existing `chunks_exact_to_as_chunks` lint. An untouched baseline
checkout also failed on that lint. With only that lint allowed, the full strict
workspace check passed:

```bash
cargo clippy --workspace --all-targets --all-features -- \
  -D warnings -A clippy::chunks_exact_to_as_chunks
```

## What changed

A main-thread profile of the first two seconds found about half of active samples
inside full-frame CPU exposure accumulation, with another quarter in text sampling.
The GPU Stage already accumulates its own shutter on the GPU. Repainting sparse
text over that frame caused repeated CPU passes over all 2,073,600 pixels.

- Stage plans containing only captions, headers, Rolling Numbers, plain text,
  and callouts now accumulate overlay exposure inside conservative ink regions.
  These regions include every shutter sample's glyph support, masks, reflection
  clips, chips, and carets. Other recipes retain full-frame accumulation.
- The constant-pixel lookup table bypasses a full-frame rewrite only after
  checking that every byte and channel maps to itself. Unusual exposure weights
  retain the original conversion.
- Text draws prepare horizontal bilinear indices, weights, and clipping once,
  prepare vertical sampling per row, and skip transparent contributions and
  padded raster areas. Bounds are measured per draw because sprites may change.
- Projected transparent card overlays reject empty filter footprints using a
  temporary alpha occupancy grid. Their original sampler remains unchanged.
- One lazy Rayon pool, capped at four workers, handles large independent text
  rows and exposure pixel chunks. Work joins before the next draw or shutter
  sample. Every pixel retains the original arithmetic and sample order. Small
  work, sharp text, one available CPU, or worker creation failure stay serial.

[fframes research](../PRIOR_ART.md#fframes-rendering) informed preparing stable work
and bounding expensive processing. This change keeps Psychopomp's existing GPU
Stage, deterministic timeline, and FFmpeg subprocess. It does not introduce a
second renderer or change rendering quality.

## Repeat a comparison

From the repository root, retain the baseline executable before changing code:

```bash
cargo build --release -p psychopomp-render
mkdir -p target/perf-baseline
cp target/release/psychopomp target/perf-baseline/psychopomp
```

After building the candidate, compare an unchanged Scene Plan or Reel:

```bash
python3 scripts/bench-render.py \
  target/perf-baseline/psychopomp target/release/psychopomp \
  scenes/hero/hero.plan.json \
  --range 3..3.2 --theme opencode --runs 5 \
  --output-dir target/render-benchmark
```

The script exits unsuccessfully if the decoded video frames differ. For exact
pixel verification, write baseline snapshots before the edit and compare afterward:

```bash
target/perf-baseline/psychopomp plan snapshot scenes/hero/hero.plan.json \
  0.05,3,3.1 target/perf-baseline/hero-snapshots --theme opencode --shutter
target/release/psychopomp plan snapshot scenes/hero/hero.plan.json \
  0.05,3,3.1 target/perf-baseline/hero-snapshots --theme opencode --shutter --compare
```

The optimized executable is `target/release/psychopomp` in this checkout after
`cargo build --release -p psychopomp-render`. Existing installed executables and
already-running exports do not pick up these edits.

## Output FPS follow-up

Exports and shutter stills now accept `--fps 24`; 60 remains the default. FPS is
an integer from 1 to 1000. The delivery rate controls the frame grid and FFmpeg
input rate. The shutter remains 180° of that frame period, with the same sample
counts. Scene Plans, Reels, and audio retain their authored times. The benchmark
script accepts the same `--fps` when both compared executables support it.

Targeted artifact verification rendered one second of the same long-film segment
at both rates. FFprobe reported 60 frames at 60/1 and 24 frames at 24/1, both with
exactly one second of video. A Reel export also reported 24 frames at 24/1 for
one second. Persistent-server exports at both rates retained identical decoded
audio bytes, a zero audio start time, and a one-second audio duration. A 24 FPS
shutter still from the server matched its CLI counterpart byte for byte. Three
default shutter snapshots still matched the earlier baseline with zero changed
pixels. Pure tests cover invalid CLI/JSON rates, defaults, clipped partial
frames, clock continuity, and shutter scaling.
