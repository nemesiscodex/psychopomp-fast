# Animation Prior Art

Psychopomp should not invent its authoring and timeline model without pressure-testing it against established animation systems. This document records the parts worth borrowing and the constraints that make direct adoption insufficient.

## Current Design Question

The immediate question is how edited narration, transcript cues, and deterministic visual motion compose on one inspectable media clock. A graphical timeline remains deferred.

The API should eventually support both:

- relative choreography: sequence, parallel, delay, stagger, and reusable animation procedures
- explicit timing: keyframes or segments placed at known times

Both forms must compile into trajectories that can be sampled independently at arbitrary timestamps. Rendering frame 120 directly must match frame 120 from a complete render.

## Functional Data Modeling Talk

The local Scala source in
`/Users/kit/code/lessons/scala-course/frontend/src/main/scala/slides/content/modeling/`
contains `Slide_3_WhyModeling.scala` and `Slide_4_Modeling.scala`: the light-switch,
Pac-Man joystick, counting, chess, and sum/product-type examples. Its `DynamicGrid`
shows Cartesian products as growing keyed rows and columns. The first Psychopomp
adaptation (`scenes/keyed-grid`) keeps finite tuple identity separate from layout
and extends those arrangements into a real connected 3D line lattice and product
reassociation. The source uses adjoining 130-pixel cells, orange borders, dark
gray interiors, outside axis labels, and opposing ±45° CSS grid rotations for
its depth reveal. The initial Psychopomp solid-block treatment was rejected;
the corrected recipe has adjoining opaque faces with restrained borders, no cell
scaling, and an orthographic view of the same 3D geometry. A wireframe intermediate
was rejected because rear lines cluttered the diagram. The current chess-symbol
example adds outside headings and centers sampled visible bounds, including
during growth. It does not port the source's timer-driven random boards, joystick
assets, or complete talk. Infinite cardinalities and arbitrary mappings remain
future examples, not claims made by the 24-cell proof.

The old depth illustration is specifically `DoubleFingerPosition` at `mult8`:
`On / Middle / Off × Finger × Finger = 3 × 5 × 5 = 75`. `DynamicGrid` uses
opposing ±45° faces, with Thumb/Pinky as the extra-coordinate examples; it does
not enumerate a full opaque 75-cell volume. The later `IceCreamOrder` recap uses
`Flavor × Conveyance × Boolean = 4 × 3 × 2 = 24`, with `hasCherryOnTop` as its
third field. This is a candidate for a more intuitive 3D scene than numbered
chess boards; it has not yet replaced the current demonstration.

`scenes/data-modeling` adapts the opening `Slide_1_Types` Boolean/cardinality,
Int/String, and Boolean/Toggle correspondence, plus `Slide_4_Modeling`'s
joystick fit, `ToggleOrJoystick`, and nullable-pair-to-`UserOrError` examples.
Its layout and timing are new native choreography, not a pixel-faithful port.
String's infinity is qualified as an abstract unbounded-length model; actual
runtime bounds are called out. The error table counts four presence combinations
versus two case shapes, not all payload values. The old general claim about equal
cardinality is replaced by an explicit invertible finite pairing. Full
Alphabet/Alterbet decoding and live joystick input remain unported.

## OpenCode Architecture Diagrams

`/Users/kit/code/open-source/opencode-architecture/src/experiments/options/Merge.tsx`
was the source for Psychopomp's former Daemon / merge diagram port (since removed).
The supplied screenshot shows its third of four steps, not `MergeGoo.tsx`.
Read-only inspection used HEAD `b7e0fa8` plus the existing working-tree stylesheet.

The source demonstrates centered finite client/server growth, 450 ms spatial and
320 ms convergence springs, independent 300 ms scale / 140 ms focus, 120 ms server
and 60 ms path offsets, a shared daemon halo, and a 400 ms caption-column spring.
Psychopomp retains the visual/choreographic intent with stable node/port identity
and cancellation-aware waits. A spring-based trace replaces the CSS cubic-ease
path/comet and a fade-through replaces mount-based wait-mode label swapping. The
initial framed port used a stationary caption aperture; Kit then rejected the
surrounding UI, so the current bare Flat/Isometric variants omit that caption and
the dotted/card chrome entirely. The goo filter,
web article, and unrelated architecture experiments are not ported. The original
project remains unchanged.

The blog's later light effects (`src/experiments/Pulse.tsx`, `CardGlow`, and
`src/graphics/scenes/pluginLifecycleScene.ts`, from September) informed the Stage's
packets and connections: a 340 ms gather, cubic-in-out flight, a cooling heat trail,
a dot-to-ring landing, border reflections with an 80 px radial falloff, embers and
floods, the frame sweep, port pop, and bead wire draw, settle-in entrances, and
instant-attack flashes. Receivers never scale on a hit. The constants live in the
`explainer-motion` skill's `TECHNIQUES.md`.

The connection treatment is now deliberately quieter: a matte eased draw-on and
fixed-size socket reveals, then stillness. The travelling draw bead and automatic
contact surge, cable twang, receiver flash, and flow were removed after visual
review. Packets and explicitly authored impacts retain their separate treatment.

## Procedural Fire And Smoke

[Inigo Quilez's domain warping](https://iquilezles.org/articles/warp/) informs
the Stage Burst's layered noise: distort the coordinates before evaluating
density so the silhouette and internal folds read organically.
[GPU Gems 3, chapter 30](https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-30-real-time-simulation-and-rendering-3d-fluids)
informs the rendering: raymarch density, accumulate emission with front-to-back
absorption, and let cooler smoke obscure the hot interior. Psychopomp uses an
analytic age-driven density rather than the chapter's simulated velocity and
temperature fields, so arbitrary-time sampling and reverse reconstruction remain
deterministic. `render/effects/combustion.wgsl` owns the bounded volume; closed-form
gravity/drag embers and screen-space pressure refraction complete the impact.

## fframes rendering

[fframes](https://github.com/dmtrKovalenko/fframes), inspected at
`b7fc055f7028f4380ed6102d33040fd1bf491036`, separates persistent GPU rendering
resources from sampled drawing values. Its
[recorded pictures and filtered layers](https://github.com/dmtrKovalenko/fframes/blob/b7fc055f7028f4380ed6102d33040fd1bf491036/fframes-skia-renderer/src/render/mod.rs)
avoid repeating stable drawing work. Cache keys retain fractional placement and
resolved content; perspective transforms do not use its filtered-layer shortcut.
Its [bounded pipeline](https://github.com/dmtrKovalenko/fframes/blob/b7fc055f7028f4380ed6102d33040fd1bf491036/fframes-skia-renderer/src/skia_pipeline.rs)
also separates frame generation, rendering, and encoding.

Psychopomp applies the narrower principle of preparing invariant sampling work
once and restricting expensive pixel work to its actual support. Text bilinear
axes are prepared per draw, projected transparent overlays reject empty filter
footprints, and supported Stage overlays average only conservative ink strips.
The existing GPU Stage exposure and FFmpeg subprocess remain concrete adapters.
[Export measurements](perf/render-throughput.md) record the tested workloads and
pixel comparisons. These are rendering changes, not another timeline or quality
profile.

## Manim

[Manim](https://docs.manim.community/en/stable/) is the strongest reference for semantic scene construction.

Relevant ideas:

- `Scene.play` makes choreography read as a sequence of meaningful operations.
- `AnimationGroup`, `Succession`, and `LaggedStart` compose parallel, sequential, and staggered work.
- Animations expose normalized progress through `interpolate(alpha)`.
- Mobjects preserve visual identity while transforms act on them.
- Trackers and updaters let one animated value drive dependent geometry.

What Psychopomp should borrow:

- animations as composable values rather than scattered property calculations
- first-class sequence, parallel, and stagger composition
- actor-targeted semantic operations such as enter, move, focus, and transform
- normalized sampling beneath a readable imperative authoring surface

What Psychopomp should avoid:

- frame-order-dependent mutation as the source of truth
- unrestricted per-frame callbacks that cannot be serialized or sampled out of order
- requiring authors to understand renderer objects to express common choreography

## Motion Canvas

[Motion Canvas](https://motioncanvas.io/docs/) is the closest TypeScript reference for code-authored motion graphics.

Relevant ideas:

- generator functions make sequential choreography linear and readable
- yielded animation generators compose through flow helpers
- signals serve as values, setters, derived values, and tween constructors
- a property can be assigned immediately or animated over a duration through one coherent interface
- dependencies can derive layout or geometry from animated signals

What Psychopomp should borrow:

- generator-style choreography as a candidate authoring frontend
- one composition algebra shared by waits, tweens, springs, sequences, and parallel groups
- typed animatable properties and derived values
- reusable procedures that return animation values

What Psychopomp should test carefully:

- whether generator execution can compile once into a durable timeline instead of becoming runtime mutable state
- whether overloaded signal getter/setter/tween syntax remains understandable for a serializable scene compiler
- how interrupted springs preserve velocity when a later operation retargets the same property

## Remotion

[Remotion](https://www.remotion.dev/docs/) is the strongest reference for deterministic frame-addressed evaluation.

Relevant ideas:

- the current frame is explicit input to rendering
- `Sequence` shifts local time and nested sequences compose offsets
- `Series` expresses consecutive ranges without manual arithmetic
- `interpolate` maps a sampled driver across keyframes
- `spring` is a pure function of frame and configuration

What Psychopomp should borrow:

- rendering as a pure function of composition time
- local time domains for nested clips and reusable components
- explicit trim, delay, and duration semantics
- interpolation as a separate operation from the source driver

What Psychopomp should improve:

- authors should not routinely calculate frame numbers
- time should use seconds or typed durations and remain independent of output frame rate
- interrupted physical motion should preserve velocity rather than restart from a newly sampled position
- layout, camera, and actor movement should all participate in temporal sampling

## Motion And React Motion

[Motion](https://motion.dev/) (formerly Framer Motion) and the older [React Motion](https://github.com/chenglou/react-motion) are references for target-driven animation.

Relevant ideas:

- authors state destinations rather than manually generating intermediate frames
- keyframes and springs share a property-oriented interface
- variants, stagger, and timelines orchestrate related actors
- layout changes can become motion while preserving element identity
- React Motion emphasizes spring destinations and natural interruption over fixed-duration curves
- Motion's `visualDuration` maps to angular frequency `2π / (visualDuration × 1.2)`; `bounce: 0` is critically damped

What Psychopomp should borrow:

- target-driven property animation
- physical interruption semantics
- ergonomic defaults and named motion profiles
- separation between stable actor identity and changing target state

What Psychopomp should avoid:

- dependence on browser layout or DOM lifecycle
- implicit real-time state that makes offline random-access sampling ambiguous
- APIs where convenience hides the compiled timeline and prevents inspection

## GPUI

[GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) is prior art for composing high-performance Rust interfaces from plain values. Its `RenderOnce`, `IntoElement`, and `Element` layers rebuild declarative element structure while retaining application state outside the tree; layout, prepaint, and paint remain distinct phases.

What Psychopomp should borrow:

- plain data components whose rendering can be rebuilt from one sampled state
- stable identity only where state, caching, or layout animation requires it
- explicit layout before paint
- small fluent fixed-canvas layout operations
- cached shaped text and expensive immutable surfaces
- call-order painting and lexical clipping for concrete renderer recipes

What Psychopomp should avoid:

- reactive entities, notifications, subscriptions, and invalidation as animation truth
- wall-clock animation elements that request future frames
- input dispatch, hitboxes, focus, accessibility, and platform-window lifecycle
- a public retained element tree or serialized flexbox vocabulary
- copying GPUI's runtime state machinery into an offline renderer whose complete state is already a function of media time

The former `deployment-queue` proof took this narrower path: recipe-owned target layout, stable keyed tracks across snapshot changes, and a private immediate-mode painter that reconstructed the complete surface at every Temporal Sample.

## Animations.dev

[Animations.dev](https://animations.dev/learn) by Emil Kowalski is practical prior art for judging product motion rather than merely implementing it. Its animation-theory, Family drawer, Dynamic Island, and good-versus-great lessons reinforce several rules that apply directly to Psychopomp scenes:

- record the result and inspect state changes frame by frame instead of tuning only from code
- give every animation one legible purpose; repeated decoration reduces the value of motion
- use decelerating motion for entry and restrained ease-in-out or no-bounce springs for persistent layout changes
- default product springs to no bounce unless a gesture or physical collision supplies the force that justifies it
- preserve velocity when an in-flight layout motion is redirected
- scale perceptual duration with element size and distance while keeping frequently encountered feedback short
- keep exits simpler than entries and avoid large scale changes; a value near the destination reads as continuity rather than creation from nothing
- preserve spatial consistency, including a stable direction and a single meaningful destination for reordering
- bridge rich state replacement deliberately: crossfade simple content, or sequence outgoing and incoming content when showing both creates visual collisions
- choose easing before duration, then inspect the result in slow motion and at normal speed
- prefer solid materials or eased multi-stop gradients; plain two-stop color gradients expose banding and often add hierarchy-free decoration

Psychopomp compiles these choices into deterministic arbitrary-time tracks rather than adopting a browser animation runtime.

### Pointer Motion Principles

[Emil Kowalski's animation guidance](https://emilkowal.ski/ui/great-animations) emphasizes natural spring motion, speed, purpose, interruptibility, and reviewing work in slow motion or frame by frame. His published design-engineering skill specifically recommends spring interpolation for decorative pointer-following motion because direct target assignment feels artificial.

Psychopomp applies that guidance with restraint:

- the pointer is explanatory rather than a frequently repeated control
- translation remains fast, interruptible, and velocity-preserving
- the pointer uses the filled Phosphor `HandPointingIcon` style that `effect-institute` selects by default
- acceleration makes the cursor lean against a direction change, creating physically grounded anticipation
- velocity turns the cursor into travel while deceleration carries it through the arrival
- pointer targets and highlight targets remain independent so attention can lead or leave the highlighted concept
- every pointer transform participates in temporal sampling and motion blur

Anticipation and follow-through come from classical animation, but should remain secondary action here. They must clarify direction and weight without delaying the pointer or turning functional explanation into decorative spectacle.

## Theatre.js

[Theatre.js](https://www.theatrejs.com/) is prior art for explicit keyframe authoring and timeline data.

Relevant ideas:

- sheets contain stable objects with typed animatable properties
- sequences hold keyframes and expose an explicit position
- code-defined objects can be driven by editor-authored timeline data
- a graph editor and dope sheet operate on the same property model used at runtime

What Psychopomp should borrow now:

- a serializable property-track and keyframe model
- stable addresses for actors and their properties
- the principle that programmatic and visual authoring can target the same compiled representation

What Psychopomp should defer:

- a graphical timeline, property inspector, graph editor, or extension system
- editor-specific project structures before the code-authored hero scene reveals the required data model

## Product Motion Guidance

[Apple's Human Interface Guidelines for Motion](https://developer.apple.com/design/human-interface-guidelines/motion), [Material 3's motion system](https://m3.material.io/styles/motion/overview/how-it-works), [IBM's classic animation principles](https://www.ibm.com/design/language/animation/classic-principles/), and [Fluent 2 choreography guidance](https://fluent2.microsoft.design/motion) constrain how classical animation principles should enter technical UI scenes.

Relevant ideas:

- feedback motion should be brief, precise, and immediately tied to a meaningful state change
- physics-based motion improves continuity and interruption, but visible oscillation should be reserved for expressive moments rather than every utility transition
- one primary action should establish the new state; glow, pulse, copy, and sound are subordinate actions that reinforce it
- success can use one restrained overshoot, failure should use a short directional impact and definitive rest, and terminal states should lose energy rather than behaving like a stronger recoverable failure
- running activity should remain readable without continuous decorative instability
- audio's main transient should coincide with visual contact, while longer sonic decay may provide follow-through after geometry has settled
- state must remain legible through shape, text, icon, and contrast without depending on motion, color, or sound alone

Psychopomp applies these constraints to Effect Task states. Running uses compression and a directional energy sweep rather than perpetual shake. Success prioritizes result expansion and content resolution. Failure stages a short horizontal impact before its error bubble, then becomes still. Death darkens and settles with less scale instead of reusing failure shake. Layout-only changes do not restart semantic flashes or pulses.

## Motion-Graphics Choreography

[Rachel Reid's anticipation article at School of Motion](https://schoolofmotion.com/blog/understanding-the-principles-of-anticipation) and [Adam Crawford's 10 Principles of Motion Design at VMG Studios](https://blog.vmgstudios.com/10-principles-motion-design) address expressive explanation rather than frequent application feedback. The relevant distinction is **choreography**, not adding the same easing to every property.

Principles to apply:

- **Staging:** hand attention from outgoing content to the new state. Two dissimilar, readable payloads in the same space create a collision, not continuity.
- **Overlapping action:** lighter content can lead the heavier container; they should not finish every channel in lockstep. The body then settles, and secondary information follows.
- **Anticipation:** a brief preparatory action can establish where to look before the main change. It need not be an obligatory opposite-direction bounce; externally caused reactions do not always need a wind-up.
- **Secondary motion:** scale, blur, opacity, and selective rotation should express one action together. A star contracting and turning into activity is one idea; spinning every result label is not.
- **Timing and weight:** preserve contrast between a quick content exit, slower container response, and readable hold. More animation is not necessarily more information.

The first application over-staged the planned Task recipe: it slowed the body and mapped one 0.32-second state spring through additional visibility windows for content, activity, and bubbles. That was continuous but felt stale against Effect Institute. The correction retains independent content scale/opacity/blur and selective symbol rotation while restoring the source's faster, overlapping, operation-specific springs. Labels and unchanged results remain still.

General principles are a vocabulary for studying a good reference, not a reason to override its timing. Independent sampled channels preserve interruption and skipped-step semantics without the source browser's mount resets or delayed callbacks. This does not relax Maximum Stability for code or prescribe icon-sized scale changes for application panels.

### Effect Task timing reference

The requested GPU blocks use `PixiEffectRow`, not the older DOM `StaticEffectNode`. Sources under `src/components/narrated/pixi-effect-row/engine/` in Effect Institute:

| Channel | Visual duration | Bounce | Source |
| --- | --- | --- | --- |
| Height, running and resting | 0.2 s | 0.5 | `config.ts::HEIGHT_SPRING_RUNNING/REST` |
| Width | 0.35 s | 0.35 | `config.ts::WIDTH_SPRING` |
| Icon opacity, scale, blur | 3/18 s ≈ 0.167 s | 0 | `NodeController.ts::ICON_*_LAMBDA`, `motion.ts::springFromLambda` |
| Result scale | 0.25 s | 0.4 | `config.ts::contentPopDuration/contentPopBounce` |
| Result blur | 0.15 s | 0 | `config.ts::contentBlurDuration` |
| Bubble opacity and scale | 3/14 s ≈ 0.214 s | 0 | `NodeController.ts::BUBBLE_*_LAMBDA` |
| Bubble blur | 3/16 s = 0.1875 s | 0 | `NodeController.ts::BUBBLE_BLUR_LAMBDA` |
| Bubble rise, 32 px | 0.25 s | 0.5 | `config.ts::bubbleRise*` |
| Base color | 3/12 s = 0.25 s | 0 | `NodeController.ts::COLOR_LAMBDA` |

These are Motion `visualDuration` parameters, not deadlines at which every spring is exactly settled. Running effects are enabled immediately in the source; the native recipe uses a short continuity ramp, not a staged content threshold. Source error bubbles also begin immediately and use their different curves to create overlap.

The regression fixture `crates/psychopomp-render/tests/fixtures/effect-task-timing.json` is generated with Motion DOM 12.42.2, as pinned by the inspected Effect Institute lockfile. Its adjacent Bun script loads cached UMD bundles without installing or modifying Effect Institute dependencies. Tests compare actual compiled Rust channel samples, not only duplicated configuration constants. The fixture uses tight rest tolerances to compare analytic curves; Psychopomp keeps its deterministic permanent-settling policy.

## Visual Types Rolling Content

### Width reveals and set geometry

The same checkout's `AnimatedType.tsx` reveals authored stable segments by width
and 4 CSS px blur (no uniform opacity fade); `animationConfigs.ts` uses 0.4-second,
zero-bounce entry and a 0.2-second exit. `AnimatedWidthText.tsx` instead exchanges a
whole measured value with width/opacity/blur using the 0.3-second default. Psychopomp's
`prototype-width-text` adopts the stable-segment variant, not whole-string replacement.
Its authored enter/leave profiles are 0.4/0.2 seconds; native navigation still uses
destination profiles with continuous position/velocity, rather than browser mount
resets. Its 4-output-pixel sampling-offset blur is not claimed to be CSS-blur parity.

`SubsetComparison.tsx` animates circles/rounded rectangles with the 0.3-second,
0.3-bounce profile, uses 20% fills and 60% 2-pixel outlines, and hatches the overlap
with 8-pixel-spaced diagonal 1.5-pixel lines at 40% opacity. Intersection geometry
is derived from current MotionValues, including nested/disjoint states. Psychopomp
keeps that sampled-geometry rule and profile, including a circle-to-square morph.
It uses explicit authored geometry rather than porting the source's label/radius
heuristics, TypeScript evaluator, result flash, framed lesson panel, or type badges.
One theme accent replaces result-specific red/green in this neutral set-diagram trial.
The native distance-field intersection also covers rounded/square boundaries.

`/Users/kit/code/experiments/typescript/visual-types` demonstrates a fixed viewing
window for moving text. `src/components/Lesson/CyclingSection.tsx` translates a
stable content stack inside an `overflow-hidden` viewport;
`Lesson/FadeOverlays.tsx` adds stationary linear top and bottom fades. The cycling
section supplies a 12-pixel fade height. `FocusedCodeBlock.tsx` applies the same
idea to a centered code line, sizing its fade regions from the measured line height.

The important feature is spatial occlusion: text disappears into the edges of
the window, rather than fading uniformly while visibly floating above or below
its resting row. Psychopomp's showcase captions use a fixed text alpha aperture
with the same 12-pixel fades. Applying it before blending, instead of painting
the source project's solid-background overlays, also preserves arbitrary
backgrounds. Existing presence channels keep skipped captions hidden, and
ordinary y tracks preserve interruption velocity. Text spring timings are unchanged.

## effect-institute

`/Users/kit/code/experiments/typescript/effect-institute` is local product prior art for semantic code states, not the immediate timeline model.

Relevant ideas:

- `line`, `stack`, `concat`, and `slot` preserve authored structure
- snapshots describe meaningful states instead of intermediate pixels
- stable line and part identity prevents unrelated code from being replaced
- focus and annotations target semantic content

Psychopomp retains these concepts: inline slots lower to independent reveal properties on stable code lines, as in `scenes/effect-succeed-slides`.

## Working Synthesis

The likely architecture has three distinct levels:

1. Authors construct stable actors and compose semantic animations using sequence, parallel, delay, stagger, tween, spring, and set operations.
2. The frontend compiles those operations into explicit property tracks, segments, and keyframes in a versioned scene IR.
3. Rust samples the compiled trajectories at arbitrary times and renders the resulting scene state.

The authoring API may feel imperative, like Manim or Motion Canvas, while the compiled representation remains declarative and inspectable, like Theatre.js tracks evaluated with Remotion-style explicit time.

## Questions To Prototype

1. Can one small composition algebra express sequence, parallel, overlap, stagger, and delay without special cases?
2. Can a generator-style TypeScript scene compile deterministically without retaining mutable execution state at render time?
3. Can explicit keyframes and relative choreography compile into the same property-track representation?
4. When two operations target the same property, are overlap and interruption rules obvious and velocity-preserving?
5. Can every compiled track explain its source operation, resolved start time, duration, and motion profile?

The next API prototype should answer these questions with panel entry, camera reframing, code-state change, and focus intensity. It should not include narration synchronization or a visual timeline editor.
