# Scene Programs Compile Into Inspectable Plans

Psychopomp keeps Rust as its authoring language. A Scene Program can import libraries, load data, run calculations, use loops, and define helpers. Running that program emits a versioned Scene Plan containing only stable actors, continuous channels, state channels, cues, and media placements.

```text
Rust Scene Program -> Scene Plan -> persistent psychopomp-render process
```

The separation keeps scene compilation lightweight and lets one renderer process retain its GPU device, font caches, and rendering implementation across repeated agent requests.

## Build An Explainer

Start from the closest existing Scene Program and change its content, not its
machinery:

| You are explaining | Start from | Library pieces |
| --- | --- | --- |
| A pull request: broken behavior, the fix, the diff | `scenes/config-migration` (smallest) or `scenes/pr-walkthrough` | `psychopomp_pr_walkthrough::film`, `narration`, `editor::diff`, `sequence` rows |
| A system, as a 3D film of cards, orbs, and packets | `scenes/opencode-jr-architecture`, `scenes/pr-walkthrough/src/flagship.rs` | `stage::StageActor` (`settle_in`, `send`, `hit`, `jolt`), `caption` |
| Code changing step by step, presented live | `scenes/effect-succeed-slides`, `scenes/interactive-showcase` | `editor` recipes, `PresentationStepPlan` |
| Springs, easing, retargeting, or a metric as curves; a plan's channels over time | `scenes/charts` | `plot::PlotActor` (`draw`, `ride`, `velocity`), `lanes::LanesPlan::from_scene_plan` |
| A payload, config, or emitted plan as structured data | `scenes/tree` | `tree::TreeActor` (`open`, `reveal`, `highlight`, `set`) |
| Pointing at a card or code range while it moves | `scenes/callouts` | `callout::CalloutActor` (`show`, `move_to`, `emphasize`) |
| Real product behavior from a screen recording | `scenes/video`, `scenes/opencode-session-tool` | `video::VideoActor` (`fly_in`, `focus`, `unfocus`) |
| Before and after, side by side | `scenes/compare` | `ReelSegmentPlan::wiped` with `ReelWipePlan` holds and labels |
| A version or count changing | `scenes/rolling-number` | `rolling::RollingNumberActor::roll` |
| A single titled idea | `scenes/agent-demo` | `PlanBuilder` channels and cues |

1. Write the narration script and voice it with `bun scripts/narrate.ts` (`--draft`
   for a local voice). Place clips with `Narration::load(dir)?.clip(id)?.place(..)`
   and time everything from `spoken.at("phrase")`, so re-voicing re-times the film.
2. Declare actors with `PlanBuilder`; write motion through typed handles. Time
   literals use `author::SECOND` and `author::seconds(f64)`.
3. Emit with `ScenePlan::write_or_print`, `DeckPlan::write_with_slides`, or
   `ReelPlan::dipped(..)`, then `plan validate` and `plan inspect`.
4. Review exact frames before encoding: `bun scripts/sheet.ts <plan> 0:10:0.5
   [--crop x,y,w,h] [--shutter]`, `plan frame <plan> <t> out.png --shutter`, and for
   code steps `plan steps`.
5. Render one cue with audio (`plan render <plan> out.mp4 --cue <id>`), then the
   whole film. When refactoring, `plan snapshot <plan> <times> <dir>` before and
   `--compare` after proves the pixels did not change.

The sections below cover presentation, narrated reels, [every recipe's payload and
channels](#recipe-payloads-and-channels), and the persistent renderer.

## Run The Example

Emit a plan from the lightweight example Scene Program:

```bash
cargo run -p agent-demo -- target/agent-demo.json
```

Emit the canonical editor-heavy hero plan:

```bash
cargo run -p psychopomp-hero -- target/hero.json
```

Emit the narration-rich OpenCode session-tool lesson:

```bash
cargo run -p psychopomp-opencode-session-tool -- \
  scenes/opencode-session-tool/opencode-session-tool.plan.json
```

Emit the minimal Solid Store to Quark before-and-after:

```bash
cargo run -p psychopomp-quark-before-after -- \
  scenes/quark-before-after/quark-before-after.plan.json
```

A Scene Program's `main` is usually one line:
`build_plan()?.write_or_print(std::env::args().nth(1))?` writes the plan to the
given path (or stdout), and `DeckPlan::write_with_slides(path)` writes a deck plus
one `<scene id>.json` per slide beside it.

Inspect or validate the result without initializing a GPU:

```bash
cargo run -- plan schema
cargo run -- plan validate target/agent-demo.json
cargo run -- plan inspect target/agent-demo.json
```

Validation owns recipe decoding, references, exclusive root selection, and
generated-channel collisions before opening fonts, GPU resources, or video caches.
Preparation retains those typed inputs. Native admission is checked separately:
a plan can be valid for export while still using unsupported interactive state or
media. See [the preparation boundary](ARCHITECTURE.md#scene-programs-and-rendering-compile-separately).

Compare two validated plans or render one exact PNG frame:

```bash
cargo run -- plan diff target/before.json target/after.json
cargo run --release -- plan frame target/agent-demo.json 1.25 output/frame.png
cargo run --release -- plan frame target/agent-demo.json 1.25 output/frame.png --shutter  # as exported, with motion blur
cargo run --release -- plan snapshot target/agent-demo.json 0:4:0.5 output/snap           # write frames
cargo run --release -- plan snapshot target/agent-demo.json 0:4:0.5 output/snap --compare # fail if any pixel changed
```

Render only one cue or exact range on the original scene clock:

```bash
cargo run --release -- plan render target/agent-demo.json output/intro.mp4 --cue intro
cargo run --release -- plan render target/agent-demo.json output/window.mp4 --range 0.2..0.4
```

A Render Window trims and rebases intersecting media to output time zero, but visual sampling remains on the original global scene clock. Starting a window in the middle of a spring therefore preserves its position and velocity.

Exports default to 60 FPS. Add `--fps 24` to `plan render` for 24 FPS, or choose
any integer from 1 to 1000. This changes output cadence without retiming the
Scene Plan, Reel, or audio. Shutter duration remains 180° of the selected frame
period, with the same temporal sample counts. `plan frame` and `plan snapshot`
accept the same `--fps` to match an export when using `--shutter`.

Delivery remains frame-based: a window whose duration is not exactly
frame-aligned emits one final frame sampled only within the remaining window
interval. Encoded duration rounds up to the next boundary at the selected FPS.

## Play As A Presentation

The themed slideshow-component showroom runs natively:

```sh
cargo run -p psychopomp-component-prototypes -- --slideshow
cargo run --release -- plan present target/slideshow-components/deck.json
```

Use **1–8** for rich text, lists/quotes, header entrances, width-revealing type
expressions, Venn diagrams, full-divider tables, an existing composition, and
reflection/stagger header variants. Slide **3** retains the original rise trial;
slide **8** compares a mirrored rise, 60 ms word offsets, and a quicker reflected
variant with 25 ms offsets. Left/Right reverses these entrances without orphaned
delayed words. An unchanged hold does not replay the header.
**T / Shift+T** cycles Original, Evergreen, Tokyo Night, and Pure Black in either
direction. The choice is saved immediately in
`$XDG_CONFIG_HOME/psychopomp/preferences.json`, or
`~/.config/psychopomp/preferences.json` when XDG is unset. Malformed preferences
produce a warning, not a crash or silent overwrite. The window title names the
active theme. Held/paused frames repaint without retargeting or advancing motion.
**C** remains a temporary grid-line audition; changing theme restores its accent.

`--theme original|evergreen|tokyo-night|black` overrides the starting presentation
theme without saving it until T is used. File export ignores personal preferences
and accepts the same explicit option:

```sh
cargo run --release -- plan frame target/slideshow-components/rich-text-showcase.json 8 output/rich.png --theme tokyo-night
cargo run --release -- plan render target/slideshow-components/venn-showcase.json output/venn.mp4 --range 3..4.2 --theme evergreen
```

Headers can fade/deblur, reveal across their measured width, or rise through a
stationary clip. Separate Presentation Steps provide deliberate header-only
holds; there are no callbacks that might fire after a skipped or reversed step.

### Inspect Motion Slowly

- **S / Shift+S:** cycle forward/backward through **1x / 0.5x / 0.25x / 0.1x**.
- **P:** pause/resume without changing the selected destination.
- **. / ,:** step forward/backward by **16.667 ms of scene time**, then remain paused.
  Backward inspection stops at the current navigation boundary; arrows still
  animate between step destinations. Frame-stepping is disabled in reduced motion.
- **Shift+R:** replay the current step and pause immediately at its entry pose.
- **D:** toggle the debug HUD: local time, time since navigation, current speed,
  pending starts/next due time, and per-word header spring progress.

For the stagger investigation:

```sh
cargo run --release -- plan present target/slideshow-components/header-variations.json --speed 0.25 --debug
```

Press **Right** once to choose the entrance step, then **Shift+R** and tap **.**
to inspect it from the beginning, or **P** to watch at quarter speed. The 60 ms
word gaps are overlapping starts, not one word finishing before the next. Rapid
reversals redirect already-moving words immediately; Replay resets them to rest
and replays the original stagger. The 25 ms quick variant deliberately overlaps
even more. No choreography timing is changed by these debug controls.

Slow motion keeps the normal display sampling cadence; it does not lower FPS.
Speed/debug are not persisted and do not affect exports. `--benchmark` and
`--benchmark-gpu` reject non-normal speed or an enabled debug HUD.

Compare plain and row-banded tables with unfilled and original 3D volumes:

```bash
cargo run -p psychopomp-keyed-grid -- --styles
cargo run --release -- plan present target/grid-styles/deck.json
```

These four scenes share `keyed-grid`; `GridStylePlan::plain_table` adds table
placement and paint, not another playback engine. Use **1–4** to select a treatment.

The reusable-component visual trials run with:

```bash
bash scenes/component-prototypes/run.sh
```

Use **1–4** to compare Typeset, Collection, Connector, and their composition.
These are explicitly provisional native recipes; see the showroom's README for
limits and the pending aesthetic verdict. This does not change the lesson deck.

The seven-slide functional-data-modeling adaptation includes the opening
types/cardinality sequence, Boolean ↔ Toggle, joystick representation fit, OR,
AND, and an illegal-state code edit:

```bash
cargo run -p psychopomp-data-modeling
cargo run --release -- plan present target/data-modeling/deck.json
cargo run -- plan steps target/data-modeling/illegal-states.json
```

It adds a small `value-token` overlay recipe with ordinary continuous channels;
no generic State Channel or recorded-media playback support is implied. See
`scenes/data-modeling/README.md` for source fidelity, controls, and counting limits.

Scene Plan v2 has optional `presentationSteps` metadata. Omitting it preserves
existing serialized plans and video behavior. Steps are separate from cues: each
has a stable ID, title, entry start, and exact held endpoint. They are ordered and
non-overlapping, and may be still-only (`startNanos == holdNanos`). Authors must
choose meaningful hold times; validation checks timing, not visual settling.

```rust
scene.presentation_step("initial", "Start with a value", 0, 0);
scene.presentation_step("reveal", "Reveal the type", 1_000_000_000, 2_500_000_000);
```

Build the Effect Institute `effect-succeed` adaptation:

```bash
cargo run -p psychopomp-effect-succeed-slides -- target/effect-succeed-slides.json
cargo run --release -- plan present target/effect-succeed-slides.json
```

This opens a native Rust window and samples the prepared scene directly, without
exporting clips. Left/Right animate toward the previous/next held pose; another
keypress immediately redirects the current springs without dropping velocity.
R explicitly restarts from the current step's entry pose. Space/P pause or resume,
Home/End select first/last, F toggles full screen, and Escape closes the window.

X toggles smooth/pixelated scaling; M toggles reduced motion. `--reduced-motion`
starts without transition motion, and `--full-quality` disables the fast flat-editor
preview profile. The default profile caches unchanged chrome and omits final
optical resampling of code glyphs, while preserving the same motion tracks. Resizing
scales the authored canvas rather than reflowing it.

Fractional glyph sampling, reveal-edge coverage, and continuous blur happen before
window scaling. Dynamic focus/highlight overlays reuse the WGSL recipe without
rebuilding static chrome through the optical compositor. Semantic coordinates
follow sampled visible widths and line positions; literal coordinates remain literal.

Final window scaling is GPU-backed with an sRGB texture and linear/nearest
sampling. `--benchmark` runs a ten-second native interruption sequence and prints
frame-submission timing; it keeps its window on top and exits automatically.

Native pacing follows the current monitor's reported refresh rate (including
120/144 Hz), falling back to 60 Hz when unavailable. `--fps 120` overrides the
sampling cap without changing video FPS or disabling FIFO synchronization. A
60 Hz display still cannot show 120 distinct frames/sec. `--benchmark-gpu` adds
explicit GPU-completion waits to the benchmark and separates completed rendering
work from waiting for a drawable; it is a diagnostic mode, not normal playback.

The interruptible player accepts any plan driven only by Continuous Channels:
Task, grid, editor, and Tree snapshots lower into continuous visual tracks.
Plans with generic State Channels, media (including Video Cards), or Rolling
Numbers are still rejected until their interactive timing is defined. The same Scene Plan still exports as MP4
through `plan render`, with its original timing and media placements. Live source
reloading, native higher-DPI glyph rasterization, and presentation audio remain open.

### Present A Deck

```bash
cargo run -p psychopomp-interactive-showcase
cargo run --release -- plan present target/interactive-showcase/deck.json
```

The four-slide demo includes the original code reveal, Effect Task lifecycle/retry,
parallel Tasks, and keyed code insertion/removal with an attached highlight.
`DeckPlan` v1 contains an ID and titled `SlidePlan` values, each embedding an
independent Scene Plan. Single-plan presentation remains supported.

- **⌘→ / ⌘←:** next/previous slide, wrapping around (`'` / Shift+`'` remain aliases).
- **1–9:** jump directly to a slide.
- **Left/Right:** previous/next step within the active slide.
- **Home/End:** first/last step within that slide.
- **C / Shift+C:** next/previous grid line color on grid slides (preview-only).
- **T / Shift+T:** next/previous presentation theme on every slide (saved).
- **S / Shift+S:** playback speed; **D:** debug HUD; **, / .:** paused frame inspection;
  **Shift+R:** replay paused at the current entry pose.
- **Space/P, R, M, X, F, Escape:** pause, replay, reduced motion, filtering,
  fullscreen, and close, as in single-plan presentation.

Inactive slides retain their step and freeze their local clock. Returning resumes
only motion that was running when the slide was left; explicit pause stays paused.
Running Tasks keep animating after their dimensions settle, until paused or advanced.
Reduced motion freezes their ambient animation too. The showcase program emits
individual slide JSON files beside the deck for `plan frame`, `plan render`, and
`plan steps`; export those Scene Plans, not the deck wrapper.

For a deck, `--benchmark` also switches slides every two measured seconds. Those
results include slide-switch costs and are not directly comparable with a
single-scene throughput benchmark.

### Present A Growing 3D Grid

```sh
cargo run -p psychopomp-keyed-grid
cargo run --release -- plan present target/keyed-grid/deck.json
```

This two-slide proof grows a connected row/table/3D line lattice, then regroups
the same 24 tuples as `(A × B) × C` and `A × (B × C)`. Straight-on and angled
orthographic views share identical 3D geometry. A camera-only step reveals
existing depth; separate extent tracks grow the grid from its fixed leading
corner without scaling individual cells. The projection centers the currently
visible geometry through growth and rotation. Opaque cells hide rear lines;
slice focus reveals one layer through continuous cutaway tracks. Optional
`GridCellLabelPlan` values provide symbols and secondary text, with row, column,
and depth headings drawn along the sides. Existing navigation,
pause, replay, and reduced motion apply to cell motion and camera angles alike.
Labels use the selected growth-edge disclosure: their feather follows the sampled
X/Y/depth extent rather than simultaneous per-label wipes. C cycles Orange, Muted
copper, Slate blue, Sage, and Chalk without changing the current motion. The
palette is a native preview preference, not a Scene Plan or export mutation.
`scenes/keyed-grid/README.md` explains the slice semantics and export commands.

`GridRecipePlan` contains three immutable `GridAxisPlan` values, an initial
`GridSnapshotPlan`, and timed snapshots. The concrete recipe caps the catalog at
256 cells and supports one root grid with text/Task overlays. Cells are keyed by
their indices in that immutable catalog; no automatic matching occurs. Do not
author in the generated `__grid.*` channel namespace. No mesh import, picking,
free orbit, or general-purpose correspondence API is exposed by this proof.

### Mask Rolling Text

The `text` recipe accepts an optional canvas-space `verticalMask`:

```json
{"text":"Run the computation","center":[960,780],"fontSize":30,
 "verticalMask":{"top":750,"bottom":810,"fade":12}}
```

The mask stays fixed while the actor's `y` channel moves the text through it.
Coverage ramps linearly from zero at `top` to full opacity at `top + fade`, stays
full in the middle, and falls to zero at `bottom`. Everything outside is clipped.
`fade` may be zero for a hard aperture, but cannot exceed half its height.
`plan validate` checks finite ordered bounds and the fade range without a GPU.

This is a text alpha mask, not a dark rectangle composited over the scene.
Other actors and the background remain unchanged. The showcase captions use a
shared 60-pixel aperture and 12-pixel edge fades, following `visual-types`' rolling
content treatment. Existing opacity channels still prevent skipped, unselected
captions from appearing while their y destinations change.

### Inspect Maximum Stability

```bash
cargo run -- plan steps target/effect-succeed-slides.json
```

No GPU is initialized. Each editor step reports before/after text, `beforeDelta`,
`delta`, changed part IDs, retained-line movement, and before/after y positions.
`«…»` marks changed parts, not unchanged text displaced by neighboring layout.
Warnings identify common text inside exchanged ranges or replaced lines, and
held endpoints that still contain moving or partially visible code. Warnings are
heuristics: they do not merge semantically different IDs. For partial holds,
reported text describes participating parts, not the exact clipped glyphs.
The persistent server accepts `{"id":1,"command":"steps","plan":"path.json"}`.

### Schedule Several Line-Order Changes

Leave `EditorRecipePlan.snapshots` empty for the original shared `layout`/`content`
placement. For keyed edits, declare every possible line once, retain the opening
`initial_line_ids`, and schedule later orders:

```rust
recipe.snapshots = vec![
    EditorSnapshotPlan {
        at_nanos: 1_000_000_000,
        line_ids: vec!["import".into(), "helper".into(), "definition".into()],
    },
    EditorSnapshotPlan {
        at_nanos: 3_000_000_000,
        line_ids: vec!["import".into(), "definition".into()],
    },
];
```

The last order must equal `final_line_ids`. A line may be absent from both initial
and final orders but present between them. Preparation derives ordinary per-line
y/opacity tracks with the 0.45-second zero-bounce profile. Equal-time snapshots
coalesce before computing layout; removed lines fade in place. Generated tracks
replace legacy shared placement in this mode; Inline Reveals remain independent.
Use presentation holds after settling (two seconds after a change is ample for
these examples). `plan validate` checks recipe ranges, snapshot timing, and generated
channel collisions without a GPU.

Old JSON plans need no new fields. Rust recipe literals use `snapshots: Vec::new()`
to retain the old behavior. Do not author in the generated `line.<id>.y`,
`line.<id>.opacity`, or `__attachment-*` namespaces.

## Make A Narrated Explainer Reel

`scenes/pr-walkthrough` walks through five pull requests: for each, a Sequence
Diagram plays the broken behavior and replays the fix in the same slots, then an
editor animates the actual change as a diff with Line Marks. The workflow is
reusable for any code explainer:

- `psychopomp::narration::Narration::load(dir)` reads `narration.json`;
  `clip(id)?.place(&mut scene, start)` adds the Script Clip and returns a
  `Spoken` whose `at(phrase)`, `at_any`, and `at_after` give plan-clock times.
- `psychopomp::editor::diff::Diff` of `keep`/`add(step, ..)`/`remove(step, ..)`
  lines declares the stepped editor; `declare(scene, step_times, warning, entrance)`.
  A hand-built editor recipe (one with semantic ranges to pin a callout, say)
  gets the same room-opening steps from `diff::step_snapshots(previous, next,
  at)` plus `diff::gap_lines(&snapshots)`.
- `SequenceRowPlan::message|reply|note|end(..)` with `.in_slot(n)` and
  `.with_aside(text)` build rows; `SequenceParticipantPlan::new(id, label, detail)`
  builds participants; `SequenceActor::row_channel`/`participant_channel` address
  their channels.
- `ReelPlan::dipped(id, plans, transition_nanos)` joins segments with dips.
- `psychopomp_pr_walkthrough::film` is the PR-film template itself (`header`,
  `chip`, `footer`, `behavior`, `code`); `scenes/config-migration` reuses it.

```sh
# 1. Voice the script (Fish Audio via 1Password; --draft uses macOS `say`).
2password run --env 'FISH_AUDIO_API_KEY=op://…' -- \
  bun scripts/narrate.ts scenes/pr-walkthrough/narration/script.json
# 2. Emit the reel; phrase lookups fail loudly if narration changed.
cargo run -p psychopomp-pr-walkthrough
cargo run --release -- plan validate scenes/pr-walkthrough/pr-walkthrough.reel.json
cargo run --release -- plan inspect scenes/pr-walkthrough/pr-walkthrough.reel.json
# 3. Review exact frames, then one segment with audio, then everything.
bun scripts/sheet.ts scenes/pr-walkthrough/pr-walkthrough.reel.json 4,30,60 --theme opencode
cargo run --release -- plan render scenes/pr-walkthrough/pr-walkthrough.reel.json output/errors.mp4 --cue errors-behavior --theme opencode
cargo run --release -- plan render scenes/pr-walkthrough/pr-walkthrough.reel.json output/pr-walkthrough.mp4 --theme opencode
```

`narrate.ts` regenerates only clips whose text, voice, engine, or settings changed and writes
`narration.json` with exact decoded durations. Draft and final clips share file
names, so switching voices re-times the reel without touching the Scene Program.

For ElevenLabs v4, use `"engine": "elevenlabs"`, `"model": "eleven_v4"`, the
voice ID, and optional `"settings": { "stability": 0.5, "similarity": 0.7 }` in
the script. Inject `ELEVENLABS_API_KEY`; `speed` is unsupported. Directions in
square brackets guide performance; Whisper timestamps the resulting speech.
The script records model/request IDs and checkpoints completed clips. The
flagship's directed example is `scenes/pr-walkthrough/narration-v4/script.json`.
Copy it under `output/` before generating to keep alternate audio there:

```sh
mkdir -p output/eleven-v4/narration
cp scenes/pr-walkthrough/narration-v4/script.json output/eleven-v4/narration/script.json
2password run --env 'ELEVENLABS_API_KEY=op://…' -- bun scripts/narrate.ts output/eleven-v4/narration/script.json
cargo run -p psychopomp-pr-walkthrough -- pr-50825 --narration output/eleven-v4/narration --output output/eleven-v4/reel.json
```

The selected reel is built lazily, so a flagship-only script needs only its three
clips. Alternate exports resolve audio against the selected narration directory
and original scene assets. The explicit `sig term`/`sigterm` cue alternatives
handle ASR word segmentation without changing recorded timings. Rebuild and
review the new clock before rendering; replacing just the audio desynchronizes it.

A reel is `{ "version": 1, "id", "segments": [{ "transitionNanos", "transitionStyle": "crossfade" | "dip" | "zoom" | "wipe", "transitionFocus"?, "transitionWipe"?, "plan" }] }`.
A `zoom` needs `transitionFocus: [x, y, width, height]` in the outgoing frame; compute
it with `stage::Camera::project` so it matches the card the camera flies into.
Relative media paths resolve against the reel file. Prefer `dip` between frames
that are both dense with text; a crossfade between two editors turns both unreadable.

A `wipe` sweeps a divider across with the incoming segment behind it. Its optional
`transitionWipe` is `{ "direction"?: "right" | "left" | "down" | "up", "holds"?:
[{ "position", "holdNanos" }], "labels"?: [outgoing, incoming] }`: the divider
travels in `direction` (`left` leaves the outgoing frame on the left, the usual
before/after order), eases minimum-jerk into each hold `position` (0..1 of its
travel), rests there for `holdNanos`, and sweeps on. Holds must leave time to sweep
within `transitionNanos`. During a hold both segments keep running on their own
clocks, so author the outgoing segment's tail and the incoming segment's head as
still frames. A held wipe suits frames that compare spatially (the same diagram
with different status, two looks of one layout); halves of the same code lines
read poorly side by side, so prefer a Stepped Diff for code.

```rust
ReelSegmentPlan::wiped(after, seconds(4.4), ReelWipePlan::new(WipeDirection::Left)
    .hold(0.5, seconds(2.6))
    .labeled("BEFORE", "AFTER"))
```

The showroom is `cargo run -p psychopomp-compare` (writes `target/compare.json`):
a held before/after between two Stage frames, then a plain downward wipe.

`scenes/pr-walkthrough` also emits `pr-50825.reel.json`, a Stage film of #50825
that zooms from the client card into its code:

```sh
cargo run -p psychopomp-pr-walkthrough pr-50825
PSYCHOPOMP_SHADER_DIR=crates/psychopomp-render/src/render \
  bun scripts/sheet.ts scenes/pr-walkthrough/pr-50825.reel.json 2,13,19,46 --theme neutral
cargo run --release -- plan render scenes/pr-walkthrough/pr-50825.reel.json output/pr-50825.mp4 --theme neutral
```

## Recipe Payloads And Channels

Every renderer recipe, by actor `recipe` name. The first two bullets point to
their fuller documentation elsewhere.

- `title-card` (root, `{ title }`), `text` (`text`, `center`, `fontSize`, and options
  such as [`verticalMask`](#mask-rolling-text)), `editor` (root, `EditorRecipePlan`) with an optional
  attached `pointer` (`PointerRecipePlan`), `effect-task` (`TaskRecipePlan`),
  `keyed-grid` (root, `GridRecipePlan`), and `value-token` (`ValueTokenPlan`).
- The provisional `prototype-typeset`, `prototype-width-text`,
  `prototype-collection`, `prototype-connector`, `prototype-rich-text`,
  `prototype-header`, and `prototype-venn` overlays (`psychopomp::component_prototype`);
  see `scenes/component-prototypes/README.md`.
- `sequence`: `participants` (`id`, `label`, `detail`) and `rows` of `kind`
  `message` (`from`, `to`, `label`, `tone`, `reply`), `note` (`over`, `text`), and
  `end` (`participant`, `label`), each with optional `slot` and `aside`. Channels:
  `opacity`, `x`, `y`, `lifelines`, `participant.<id>.opacity|emphasis`,
  `row.<id>.reveal|opacity|strike`. Use `SequenceActor` to write reveals by row.
- `caption`: `origin`, `align`, `size`, `lines` of `{ text, tone }` spans, `chip`.
  Channels: `opacity`, `x`, `y`, `typed`, `caret`. `CaptionActor::type_in` writes
  one exact step per character; `show` and `hide` fade.
- `rolling-number`: `origin` (aligned edge x, center y), `align`, `size`, `bold`,
  `tone`, static `prefix`/`suffix` spans (`{ text, tone }`), the initial `value`,
  and `rolls` of `{ atNanos, value }` in increasing time. Optional
  `durationNanos` (500 ms), `stagger` (`outward` | `start` | `end` | `none`),
  `direction` (`auto` | `up` | `down`), `blur` (smear strength, 1; 0 disables),
  and `chip`. Channels: `opacity`, `x`, `y`. Digits roll; `,` between digits
  groups and `.` between digits starts a fraction, so `rc.` and `/` are literals.
  `RollingNumberActor::roll` appends a change at a phrase's time; `show`/`hide`
  fade like a caption. Plans using it are export-only (not `plan present`).
  ```rust
  let mut version = RollingNumberActor::declare(&mut scene, "version",
      RollingNumberPlan::new([960.0, 300.0], 72.0, "rc.112")
          .aligned(CaptionAlign::Center).tone(Tone::Accent)
          .prefix(vec![CaptionSpanPlan::new("opencode ", Tone::Muted)]).chip())?;
  version.show(&mut scene, at);
  version.roll(&mut scene, phrase_start, "rc.117")?;
  ```
  The showroom is `cargo run -p psychopomp-rolling-number` (writes
  `target/rolling-number.json`); render it with
  `cargo run --release -- plan render target/rolling-number.json output/rolling-number.mp4 --theme opencode`.
- `tree`: `origin` (top-left of the first row), `width` (highlight extent and
  truncation), `size` (24), `maxRows` (a scrolling window), `indent` (2 columns),
  the JSON `value`, `expanded` (JSONPaths open at time zero), and `changes` of
  `{ path, value }` (later scalar values, in order per path). Rows are keyed by
  JSONPath (`$`, `$.actors[0].id`, `$["odd\u0020key"]`). Channels: `opacity`,
  `x`, `y`, `scroll` (rows), and per path `node.<path>.open` (0 folded to 1 open,
  non-empty objects and arrays only), `node.<path>.highlight`, and
  `node.<path>.value` (variant index: 0 is `value`'s, `n` is that path's `n`th
  change). An opening node's closing bracket slides out from under its row and
  rows below move by exactly the room it opens; its children fade in at full
  pitch inside that room, and nothing above it moves. A value change rolls up
  through its row's window. `TreeActor` writes `open`/`close` (0.45 s
  zero-bounce), `highlight(path, at, seconds)`, `set(path, value, at)`,
  `scroll_to(row, at)`, and `reveal(path, at)`, which scrolls the least distance
  that shows a path's block once the authored folds settle. Object keys display
  in `serde_json::Value` order (sorted). Trees are channel-only, so they run in
  `plan present` too.
  ```rust
  let mut tree = TreeActor::declare(&mut scene, "plan",
      TreePlan::new([560.0, 150.0], 900.0, emitted).max_rows(22).expanded(["$"]))?;
  tree.open(&mut scene, "$.continuousChannels", at)?;
  tree.reveal(&mut scene, "$.continuousChannels", at)?;
  tree.highlight(&mut scene, "$.continuousChannels[0].property", later, 2.0)?;
  tree.set(&mut scene, "$.continuousChannels[0].initial", json!(1.0), later)?;
  ```
  The showroom is `cargo run -p psychopomp-tree` (writes `target/tree.json`, the
  plan `agent-demo` emits); render it with
  `cargo run --release -- plan render target/tree.json output/tree.mp4 --theme neutral`.
- Axes (shared by `plot` and `lanes`): `{ range: [start, end], ticks?, label?, unit? }`.
  `AxisPlan::new(range).every(step)` or `.nice(count)` chooses ticks; labels share
  the fewest exact decimals and append `unit`.
- `plot`: `origin` (top-left of the data frame), `size`, `x` and `y` axes,
  `series` of `{ id, label?, tone?, dashed?, points: [[x, y]], slopes? }` (x never
  decreasing; a repeated x draws a step; `slopes` are exact dy/dx per point, else
  central differences), and `marks` of `{ id, x, label? }`. Channels: `opacity`,
  `x`, `y`, `axes` (draw-on), `playhead` (an x value; no playhead without the
  channel), `playhead.opacity`, `series.<id>.draw|opacity|ride|velocity`, and
  `mark.<id>.opacity`. A riding series puts a dot at the playhead; `velocity`
  adds its tangent arrow (where the dot will be 8% of the x range later) and a
  signed `v` readout above the frame. The Scene Program computes every curve:
  `PlotSeriesPlan::sampled(id, label, tone, range, samples, |x| ..)` or
  `::motion(.., |t| MotionState)` for exact velocities. Sample motions from the
  compiled Property Track to show what the engine actually does.
  ```rust
  let mut plot = PlotActor::declare(&mut scene, "plot", &PlotPlan::new(
      [250.0, 250.0], [1420.0, 560.0],
      AxisPlan::new([0.0, 1.6]).every(0.2).label("time (s)"),
      AxisPlan::new([0.0, 1.3]).every(0.5).label("position"))
      .series(PlotSeriesPlan::motion("bouncy", "bouncy", Tone::Accent, [0.0, 1.6], 321, &track)))?;
  plot.show(&mut scene, at, 1.0);               // fade in, draw the axes
  let drawn = plot.draw(&mut scene, "bouncy", at, 1.4);
  let arrived = plot.ride(&mut scene, "bouncy", [0.0, 1.6], drawn, 3.0);
  plot.velocity(&mut scene, "bouncy", drawn, 1.0);
  plot.stop_ride(&mut scene, "bouncy", arrived);
  ```
- `lanes`: `origin` (top-left, above the cue row), `width`, `time` axis (seconds),
  optional `labelWidth` (400) and `laneHeight` (52), `lanes` of `{ id, label,
  tone?, keys?: [seconds], curve?: [[seconds, value]] }` (sparklines scale to
  their own range), and `cues` of `{ id, start, end, label? }`. Channels:
  `opacity`, `x`, `y`, `reveal` (ruler, lanes top to bottom, then cues),
  `playhead` (seconds), `playhead.opacity`, and `lane.<id>.opacity|emphasis`.
  The cue under the playhead brightens and crossed keys light, then cool.
  `LanesPlan::from_scene_plan(&plan, origin, width, |channel| Some(label))` builds
  lanes from a plan's continuous channels: keys at their event times, sparklines
  from the compiled tracks, and the plan's cues. `LanesActor::show`, `scrub`, and
  `emphasize` write the channels.
  The showroom is `cargo run -p psychopomp-charts` (writes `target/charts.json`);
  render it with
  `cargo run --release -- plan render target/charts.json output/charts.mp4 --theme neutral`.
- `video` (Video Card): `mediaId` (a planned `video` media placement), `size`
  (decoded `[width, height]`), `fps`, `center`, `width` (card width at scale 1;
  height follows the footage aspect), and optional `title` (a 44 px title bar).
  Channels: `x`, `y` (offsets from `center`), `scale`, `opacity`, `rotation`,
  `tilt-x`, `tilt-y`, `blur` (near-edge defocus of a tilted card), and the focus
  window `focus-x`, `focus-y` (its center, as fractions of the frame; 0.5) and
  `focus-size` (fraction of the frame visible; 1). The footage's source time follows
  its placement on the plan clock, holding the first frame before it and the last
  after it. `VideoActor::declare(scene, id, plan, video::media(id, path, (from, to),
  at))` adds the card and its placement; `fly_in`, `focus(region_in_source_px)`,
  `unfocus`, and `hide` write the motion. Video Cards draw beneath other overlays,
  over any root, and plans using them are export-only.
  ```rust
  let mut card = VideoActor::declare(&mut scene, "recording",
      VideoPlan::new("session", [1920, 760], 60).at([960.0, 520.0], 1520.0).titled("vim  /  opencode v2"),
      video::media("session", "../assets/opencode-v2-session-tool/max-hot-reload-split.mp4", (0, 9 * SECOND), 0))?;
  card.fly_in(&mut scene, seconds(0.25));
  card.focus(&mut scene, seconds(3.0), [920.0, 225.0, 900.0, 356.0], 0.9);
  ```
  The showroom is `cargo run -p psychopomp-video` (writes `target/video.json`).
- `callout`: `anchors` (one to eight; the first is where it starts), `lines` (one
  to three lines of `{ text, tone }` spans), `size` (24), `side` (where the label
  sits: `top`, `bottom`, `left`, `right`, `top-left`, `top-right` (default),
  `bottom-left`, `bottom-right`), `reach` (first-leg length, 64 px), `elbow`
  (diagonals turn into a 28 px horizontal shelf), `tone` (leader and mark,
  `accent`), and `chip`. Each anchor has an `id` and a `kind`: `point` (`at`),
  `stage` (`element`, a positioned element of the Stage root), or `editor`
  (`target`, a Semantic Target of the editor root), with an optional `edge`
  (`center` by default, or a side or corner of the outline) and an optional
  per-anchor `side`. Channels: `opacity`, `draw` (leader draw-on; the mark
  appears with it), `label` (fade and 8 px rise), `emphasis`, and
  `anchor.<id>` weights (1 for the first anchor, 0 otherwise). The renderer
  resolves every weighted anchor at every sample from the prepared root and
  blends them, so the leader stays on its card through camera moves, jolts, and
  shutter samples, and on its code range as lines move or the panel zooms.
  `CalloutActor` writes `show` (draw on, then the label), `hide` (label out,
  then retract), `move_to(anchor)` (every weight springs on one critically
  damped profile, so interrupted moves keep their velocity), and `emphasize`
  (instant flare, convex decay).
  ```rust
  let mut note = CalloutActor::declare(&mut scene, "retries",
      &CalloutPlan::new(CalloutAnchorPlan::Stage { id: "client".into(),
          element: "client".into(), edge: CalloutSide::Top, side: None },
          vec![CaptionSpanPlan::new("retries 3×", Tone::Plain)])
          .anchor(CalloutAnchorPlan::Stage { id: "api".into(), element: "api".into(),
              edge: CalloutSide::Top, side: Some(CalloutSide::TopLeft) })
          .elbow())?;
  note.show(&mut scene, at);
  note.move_to(&mut scene, "api", later)?;
  ```
  The showroom is `cargo run -p psychopomp-callouts` (writes the reel
  `target/callouts.json` and its segments under `target/callouts/`); render it
  with `cargo run --release -- plan render target/callouts.json output/callouts.mp4 --theme neutral`.
- Editor Line Marks: `"mark": "added" | "removed"` on a line, with presence
  channel `mark.<line-id>`; `panel-x`, `panel-y`, and `panel-opacity` move and fade the card (the Stepped Diff
  enters on `panel-y`).
- `stage` (root): `elements` of `kind` `card` (`at`, `size`, `title`, `status`,
  `tone`), `orb` (`at`, `radius`, `points`), `beam` (`from`, `to`, `bend`),
  `packet` (`beam`, `reverse`, `label`), `label` (`at`, `size`, `spans`), and `ring`
  (`at`, `radius`, `thickness`), plus `post` (`bloom`, `grain`, `vignette`,
  `backdrop`). Channels are `<element>.<property>` (for example `service.shatter`,
  `link.draw`, `probe.age`, `client.blur|content`) and `camera.x|y|z|focus|dof|shake|kick-x|kick-y|punch`,
   `post.bloom|chroma|exposure|vignette|rewind`. `post.rewind` is a 1.4-second local
   age for VHS rewind interference (-1 inactive). Cards also take the deletion
   channels `cool|damage|glitch|cut|ghost` and the status-spinner clocks
   `spinner|release|mark` (seconds; -1 inactive), with `mark: "check" | "cross"`.
   A packet is one clock: `age` (seconds since
  dispatch, -1 before) and `flight`; the renderer derives its gather, flight, trail,
  landing ring, and light from them. Beams choose their own ports and curve; leave
  `bend` at 0 unless two beams need separating.
- `StageActor` implements the `explainer-motion` beats: `settle_in` (a panel drifts
  16 px into place, scales from 1.035, and sharpens; its content follows 65 ms later),
  `connect` (soft fixed-size port reveal, eased wire draw, then a quiet hold),
  `send` (gather, flight, landing), `hit` (instant attack, convex decay),
  `kick` (a two-frame shove that springs back past rest), `jolt` (an impact: the
  camera kicks along the blow, a squared-trauma noise rumble with slight roll
  decays, and the frame punches in about 2%),
  `twang`, and `land`. `bounce` and `to` spring any channel (undeclared
  channels start at 0; declare other starting poses with `channel`), `ease` follows
  any curve, and `clock` starts an elapsed-seconds channel that runs to the scene's
  end for effect rigs such as the card spinner (`clock_for` stops it after a fixed
  lifetime, as for `burst` or `post.rewind`). Use a `Smootherstep` ease for staged
  camera moves with exact timing, springs for responsive camera/panel settling,
  and instant-attack fades for light.
  Orb `pulse` changes illumination, not geometry or attached beam ports. Card
  `flash` lifts ink and rim, not the entire fill. Connecting does not implicitly
  trigger `land`, `twang`, `surge`, or `flow`; author those only when the story
  calls for an impact or ongoing traffic. Packet labels use measured text bounds
  to stop short of the endpoint bodies while their packets finish travelling.
  A label that cannot fit in the corridor is omitted rather than partially hidden.
  Packets entering an orb trigger a directional surface ripple at the visible
  shell, before reaching the submerged endpoint. The flagship now uses critical
  `to` springs for camera moves; a `Smootherstep` ease remains available for
  minimum-jerk timing.
- Orb `rotation` is an angular offset in radians; animate it for a spin entrance
  rather than changing the ambient `spin` multiplier. `blur` adds defocus in world
  pixels. `burst` defaults to -1 (intact): set 0 on impact and ease linearly to
  5.2 over 5.2 seconds for collapse, fire, smoke, and ballistic embers. Reverse
  that clock to reassemble, then set -1 when it reaches zero. The first active
  burst also supplies the composite's gravity pinch and refractive shockwave.
  Volumes and sparks respect orb opacity; the pressure wave is a scene response.

Continuous channel events are `set`, `spring`, and `ease`
(`{ "operation": "ease", "atNanos", "target", "durationNanos", "curve" }` with
`curve` one of `linear`, `smoothstep`, `smootherstep`, `cubic-out`, `cubic-in-out`,
`{ "decelerate": s }`, or `{ "cubic-bezier": [x1, y1, x2, y2] }`). Use `ease` for
timed curves; never approximate one with stepped `set` events, which stutter.
Reusable math is `psychopomp::math` (`lerp`, `remap_clamp`, `smoothstep`, `easing`,
`dynamics::settle`, `curve::Polyline`, `shapes::connect`, glam vectors); use it in
Scene Programs too. `--theme opencode` renders with the OpenCode TUI's tokens;
`--theme neutral` with the OpenCode blog's clear-neutral diagram palette (the
#50825 film's look).

## Keep The Renderer Running

`psychopomp plan serve` reads one JSON request per line from standard input and writes one JSON response per line to standard output. Progress and GPU diagnostics use standard error, leaving standard output machine-readable.

```bash
cargo run --release -- plan serve
```

Inspect a plan:

```json
{"id":1,"command":"inspect","plan":"target/agent-demo.json"}
```

Render one frame while retaining the initialized renderer:

```json
{"id":2,"command":"frame","plan":"target/agent-demo.json","output":"output/frame.png","at_nanos":1250000000}
```

Render an exact range:

```json
{"id":3,"command":"render","plan":"target/agent-demo.json","output":"output/window.mp4","start_nanos":200000000,"end_nanos":400000000}
```

Render requests accept `"fps":24`, defaulting to 60 when omitted. Frame requests
accept `"shutter":true` and the same `"fps":24` to inspect export motion blur.
FPS must be an integer from 1 to 1000, and instantaneous frames ignore cadence.

Render a cue:

```json
{"id":4,"command":"render","plan":"target/agent-demo.json","output":"output/intro.mp4","cue":"intro"}
```

Stop the process:

```json
{"id":5,"command":"shutdown"}
```

## Author With Typed Handles

`PlanBuilder` creates stable handles and derives channel IDs from actor identity:

```rust
let mut scene = PlanBuilder::new("lesson", 3_000_000_000);
let title = scene.actor(
    "title",
    "title-card",
    serde_json::json!({ "title": "Effect" }),
)?;
let opacity = scene.continuous(&title, "opacity", 0.0);
scene.spring(&opacity, 200_000_000, 1.0, 0.4, 0.0);
scene.cue("intro", 0, 2_000_000_000);
let plan = scene.finish()?;
```

Renderer Recipe payloads remain adapter-owned. The lightweight core validates stable IDs, channel references, event ordering, finite values, cue ranges, exact media ranges, and Scene Plan versioning without knowing what a Task, editor, Video Card, or title card looks like.

Scene Plan v2 scalar values may reference a component of a stable Semantic Target. The target's selector remains recipe-owned; for the hero, the editor recipe resolves logical code range IDs through `cosmic-text` before compiling highlight and pointer channels into the shared Timeline.

The plan runtime's recipes are listed under [Recipe Payloads And Channels](#recipe-payloads-and-channels). Planned audio lowers into exact script or layer placements for FFmpeg. Planned video is accepted only when a `video` actor consumes its media ID; unconsumed video and all image media still return request errors. The Video Card maps the global scene clock through the media placement into source time, so cue and range renders do not restart footage. Editor and Video Card recipes independently produce RGBA content but delegate framing to the same private immediate-mode card compositor; this reuse does not add recursive presentation nodes to Scene Plan.

## Package Direction

```text
scenes/* ------------> psychopomp
                           ^
                           |
psychopomp-render ----------+
```

- `crates/psychopomp`: lightweight plans, authoring values, motion, composition, stable code, and validation
- `crates/psychopomp-render`: concrete renderer, encoder, development server, and CLI
- `scenes/*`: lightweight Rust Scene Programs

The default hero command embeds `scenes/hero/hero.plan.json` for compatibility. A workspace test regenerates the plan from `scenes/hero/src/lib.rs` and requires byte equality, so the checked artifact cannot drift from its Rust source.

`scenes/opencode-session-tool/opencode-session-tool.plan.json` is likewise checked against its Rust Scene Program. It demonstrates one planned split Vim/OpenCode Video Card, layered SFX, continuous card motion, nine live-capability text overlays, and named cue selection through the same renderer process.

The plan and authoring Modules intentionally share one lightweight crate. They should become separate crates only after another language, protocol consumer, or independent version lifecycle demonstrates that seam.
