use std::{path::Path, time::Duration};

use psychopomp::{
    code::{CodeLine, StyledSpan, SyntaxStyle},
    editor::{EditorLinePlan, EditorPartPlan, EditorRecipePlan, EditorSnapshotPlan},
    motion::MotionState,
    plan::{
        ContinuousChannelPlan, PresentationStepPlan, ScalarPlan, SemanticTargetPlan,
        TargetComponentPlan, TargetScalarPlan, TrackEventPlan,
    },
    playback::PlaybackCommand,
};

use super::*;

fn target(id: &str, component: TargetComponentPlan, offset: f32) -> ScalarPlan {
    ScalarPlan::Target(TargetScalarPlan {
        target_id: id.into(),
        component,
        offset,
    })
}

fn attached_plan() -> ScenePlan {
    let mut plan = psychopomp_effect_succeed_slides::build_plan().unwrap();
    for (id, range) in [("equals", "equals"), ("name", "name")] {
        plan.semantic_targets.push(SemanticTargetPlan {
            id: id.into(),
            actor_id: "editor".into(),
            selector: json!({"lineId": "magicWord", "rangeId": range}),
        });
    }
    for (property, initial) in [
        ("highlight-x", target("equals", TargetComponentPlan::X, 0.)),
        (
            "highlight-y",
            target("equals", TargetComponentPlan::LineY, 0.),
        ),
        (
            "highlight-width",
            target("equals", TargetComponentPlan::Width, 0.),
        ),
        ("highlight-opacity", 1.0.into()),
    ] {
        plan.continuous_channels.push(ContinuousChannelPlan {
            id: format!("editor.{property}"),
            actor_id: "editor".into(),
            property: property.into(),
            initial,
            events: Vec::new(),
        });
    }
    plan
}

#[test]
fn renderer_validation_checks_editor_data_without_a_gpu() {
    let mut plan = attached_plan();
    validate_renderer_plan(&plan).unwrap();
    plan.actors[0].data["additionalInlineReveals"][0] = plan.actors[0].data["inlineReveal"].clone();
    assert!(
        validate_renderer_plan(&plan)
            .unwrap_err()
            .to_string()
            .contains("overlap")
    );
    let mut plan = attached_plan();
    let final_ids = plan.actors[0].data["finalLineIds"].clone();
    plan.actors[0].data["snapshots"] = json!([
        {"atNanos": 3_000_000_000u64, "lineIds": final_ids},
        {"atNanos": 1_000_000_000u64, "lineIds": final_ids}
    ]);
    assert!(
        validate_renderer_plan(&plan)
            .unwrap_err()
            .to_string()
            .contains("ordered")
    );
    plan.actors[0].data["snapshots"] =
        json!([{ "atNanos": 20_000_000_000u64, "lineIds": final_ids }]);
    assert!(validate_renderer_plan(&plan).is_err());
    plan.actors[0].data["snapshots"] =
        json!([{ "atNanos": 1_000_000_000u64, "lineIds": final_ids }]);
    plan.continuous_channels.push(ContinuousChannelPlan {
        id: "collision".into(),
        actor_id: "editor".into(),
        property: "line.import.y".into(),
        initial: 0.0.into(),
        events: Vec::new(),
    });
    assert!(
        validate_renderer_plan(&plan)
            .unwrap_err()
            .to_string()
            .contains("collides")
    );
}

#[test]
fn text_mask_validation_rejects_invalid_apertures_without_a_gpu() {
    let validate = |actor: &psychopomp::plan::ActorPlan| {
        let mut plan = ScenePlan::new("mask", 1_000_000_000);
        plan.actors.push(actor.clone());
        validate_renderer_plan(&plan)
    };
    let mut actor = psychopomp::plan::ActorPlan {
        id: "caption".into(),
        recipe: "text".into(),
        data: json!({"text": "Rolling", "center": [960, 780]}),
    };
    validate(&actor).unwrap();
    actor.data["verticalMask"] = json!({"top": 750, "bottom": 810, "fade": 12});
    validate(&actor).unwrap();
    for invalid in [
        json!({"top": 810, "bottom": 750, "fade": 12}),
        json!({"top": 750, "bottom": 750, "fade": 0}),
        json!({"top": 750, "bottom": 810, "fade": -1}),
        json!({"top": 750, "bottom": 810, "fade": 31}),
        json!({"top": 750, "bottom": 1e100, "fade": 12}),
        json!({"top": "750", "bottom": 810, "fade": 12}),
        json!({"top": 750, "bottom": 810}),
        json!(null),
    ] {
        actor.data["verticalMask"] = invalid;
        assert!(
            validate(&actor).is_err(),
            "accepted {}",
            actor.data["verticalMask"]
        );
    }
}

#[test]
#[ignore = "requires a headless GPU; rolling captions share a stationary mask in native and video sampling"]
fn rolling_captions_stay_inside_their_aperture_through_reversals() {
    let mut plan = psychopomp_interactive_showcase::build_deck()
        .unwrap()
        .slides[1]
        .plan
        .clone();
    plan.actors.retain(|actor| actor.id.starts_with("caption-"));
    plan.continuous_channels
        .retain(|channel| channel.actor_id.starts_with("caption-"));
    let mut unmasked = plan.clone();
    for actor in &mut unmasked.actors {
        actor.data.as_object_mut().unwrap().remove("verticalMask");
    }
    let mut renderer = pollster::block_on(new_renderer("rolling-caption")).unwrap();
    let prepared = PreparedPlan::prepare(plan, Path::new("."), &mut renderer).unwrap();
    let unmasked = PreparedPlan::prepare(unmasked, Path::new("."), &mut renderer).unwrap();
    let blank = renderer.render_title_card("", None, 0.);
    let assert_clipped = |pixels: &[u8]| {
        for y in (0..750).chain(810..1080) {
            let start = y * 1920 * 4;
            assert_eq!(
                &pixels[start..start + 1920 * 4],
                &blank[start..start + 1920 * 4],
                "mask leaked at row {y}"
            );
        }
    };
    // The held line stays completely sharp. Only moving edge pixels change.
    assert_eq!(
        prepared.render_sample(&mut renderer, 0.).unwrap(),
        unmasked.render_sample(&mut renderer, 0.).unwrap()
    );
    let expected = prepared.render_sample(&mut renderer, 3.12).unwrap();
    assert_clipped(&expected);
    assert_ne!(
        expected,
        unmasked.render_sample(&mut renderer, 3.12).unwrap()
    );
    prepared.render_sample(&mut renderer, 15.).unwrap();
    assert_eq!(
        expected,
        prepared.render_sample(&mut renderer, 3.12).unwrap()
    );

    let mut playback = prepared.playback(false).unwrap();
    playback.command(PlaybackCommand::Next, Duration::ZERO);
    let live = prepared
        .render_sample_using(&mut renderer, 0.12, &playback.timeline())
        .unwrap();
    assert_eq!(live, expected);
    for (time, command) in [
        (0.12, PlaybackCommand::Previous),
        (0.2, PlaybackCommand::Next),
        (0.24, PlaybackCommand::Last),
        (0.3, PlaybackCommand::First),
    ] {
        let before = prepared
            .render_sample_using(&mut renderer, time, &playback.timeline())
            .unwrap();
        playback.command(command, Duration::from_secs_f64(time));
        assert_eq!(
            before,
            prepared
                .render_sample_using(&mut renderer, time, &playback.timeline())
                .unwrap()
        );
        assert_clipped(
            &prepared
                .render_sample_using(&mut renderer, time + 0.02, &playback.timeline())
                .unwrap(),
        );
    }
    playback.set_reduced_motion(true, Duration::from_secs_f64(0.4));
    assert_eq!(
        prepared
            .render_sample_using(&mut renderer, 0.4, &playback.timeline())
            .unwrap(),
        prepared.render_sample(&mut renderer, 0.).unwrap()
    );
}

#[test]
fn task_preflight_rejects_id_only_collisions_and_accepts_actual_position_overrides() {
    let mut plan = psychopomp_interactive_showcase::build_deck()
        .unwrap()
        .slides[1]
        .plan
        .clone();
    plan.continuous_channels.push(ContinuousChannelPlan {
        id: "loadAnswer.x".into(),
        actor_id: "heading".into(),
        property: "x".into(),
        initial: 10.0.into(),
        events: Vec::new(),
    });
    assert!(
        validate_renderer_plan(&plan)
            .unwrap_err()
            .to_string()
            .contains("task channel")
    );
    plan.continuous_channels.last_mut().unwrap().id = "heading.custom-x".into();
    plan.continuous_channels.push(ContinuousChannelPlan {
        id: "custom-task-x".into(),
        actor_id: "loadAnswer".into(),
        property: "x".into(),
        initial: 960.0.into(),
        events: Vec::new(),
    });
    validate_renderer_plan(&plan).unwrap();
    plan.continuous_channels.push(ContinuousChannelPlan {
        id: "manual-width".into(),
        actor_id: "loadAnswer".into(),
        property: "width".into(),
        initial: 128.0.into(),
        events: Vec::new(),
    });
    assert!(
        validate_renderer_plan(&plan)
            .unwrap_err()
            .to_string()
            .contains("task channel")
    );
}

#[test]
#[ignore = "requires a headless GPU; normalized companion thresholds cannot amplify coordinate error"]
fn semantic_to_identical_literal_stays_within_coordinate_tolerances() {
    let mut plan = attached_plan();
    let mut renderer = pollster::block_on(new_renderer(&plan.id)).unwrap();
    let visible = CodeLine::new(
        "visible",
        vec![StyledSpan::new("const magicWord =", SyntaxStyle::Plain)],
    );
    let x = renderer.measure_text_range(&visible, " =").unwrap().x;
    let channel = plan
        .continuous_channels
        .iter_mut()
        .find(|channel| channel.property == "highlight-x")
        .unwrap();
    channel.events.push(TrackEventPlan::Spring {
        at_nanos: 1_000_000_000,
        target: x.into(),
        response_seconds: 0.48,
        damping_ratio: 1.0,
        position_threshold: 0.02,
        velocity_threshold: 0.05,
    });
    let mut set_plan = plan.clone();
    set_plan
        .continuous_channels
        .iter_mut()
        .find(|channel| channel.property == "highlight-x")
        .unwrap()
        .events[0] = TrackEventPlan::Set {
        at_nanos: 1_000_000_000,
        value: x.into(),
    };
    let prepared = PreparedPlan::prepare(plan, Path::new("."), &mut renderer).unwrap();
    for tick in 0..2000 {
        let state = prepared
            .motion_value(
                &prepared.timeline,
                "editor",
                "highlight-x",
                1.0 + f64::from(tick) / 1000.,
            )
            .unwrap();
        assert!((state.position - x).abs() <= 0.02, "{tick}: {state:?}");
        assert!(state.velocity.abs() <= 0.06, "{tick}: {state:?}");
    }
    let prepared = PreparedPlan::prepare(set_plan, Path::new("."), &mut renderer).unwrap();
    let mut playback = prepared.playback(false).unwrap();
    playback.command(PlaybackCommand::Next, Duration::ZERO);
    for tick in 0..2000 {
        let state = prepared
            .motion_value(
                &playback.timeline(),
                "editor",
                "highlight-x",
                f64::from(tick) / 1000.,
            )
            .unwrap();
        assert!(
            (state.position - x).abs() <= 0.002,
            "set-only {tick}: {state:?}"
        );
        assert!(state.velocity.abs() <= 0.003, "set-only {tick}: {state:?}");
    }
}

fn assert_close(a: MotionState, b: MotionState) {
    assert!(
        (a.position - b.position).abs() < 0.001,
        "position {a:?} != {b:?}"
    );
    assert!(
        (a.velocity - b.velocity).abs() < 0.001,
        "velocity {a:?} != {b:?}"
    );
}

#[test]
#[ignore = "requires a headless GPU; proves native Task state mixtures and ambient sampling"]
fn effect_blocks_reverse_without_losing_pixels_and_running_motion_is_not_cached() {
    let deck = psychopomp_interactive_showcase::build_deck().unwrap();
    let mut renderer = pollster::block_on(new_renderer("tasks")).unwrap();
    for slide in deck.slides.into_iter().filter(|slide| {
        slide
            .plan
            .actors
            .iter()
            .any(|actor| actor.recipe == psychopomp::task::TASK_RECIPE)
    }) {
        let prepared = PreparedPlan::prepare(slide.plan, Path::new("."), &mut renderer).unwrap();
        assert_eq!(
            prepared.visual_sample_key(0.5).unwrap(),
            prepared.visual_sample_key(0.75).unwrap()
        );
        assert_ne!(
            prepared.visual_sample_key(5.).unwrap(),
            prepared.visual_sample_key(5.1).unwrap()
        );
        let a = prepared.render_sample(&mut renderer, 5.).unwrap();
        let b = prepared.render_sample(&mut renderer, 5.1).unwrap();
        assert_ne!(a, b);
        assert_eq!(a, prepared.render_sample(&mut renderer, 5.).unwrap());
        let mut playback = prepared.playback(false).unwrap();
        for (time, command) in [
            (0., PlaybackCommand::Next),
            (0.13, PlaybackCommand::Last),
            (0.21, PlaybackCommand::Previous),
            (0.28, PlaybackCommand::First),
            (0.32, PlaybackCommand::Next),
        ] {
            let local = playback.sample(Duration::from_secs_f64(time)).at_nanos as f64 / 1e9;
            let old = playback.timeline();
            let before = prepared
                .render_sample_using(&mut renderer, local, &old)
                .unwrap();
            playback.command(command, Duration::from_secs_f64(time));
            for channel in &prepared.plan.continuous_channels {
                let property = PropertyId::new(&channel.id);
                assert_close(
                    old.sample_at(&property, local).unwrap(),
                    playback.timeline().sample_at(&property, local).unwrap(),
                );
            }
            assert_eq!(
                before,
                prepared
                    .render_sample_using(&mut renderer, local, &playback.timeline())
                    .unwrap()
            );
        }
    }
}

#[test]
#[ignore = "requires a headless GPU; staged Task content must match native/video and stay stable on unrelated steps"]
fn staged_task_content_matches_delivery_and_preserves_unaffected_pixels() {
    let deck = psychopomp_interactive_showcase::build_deck().unwrap();
    let mut renderer = pollster::block_on(new_renderer("task-content")).unwrap();
    let prepared =
        PreparedPlan::prepare(deck.slides[1].plan.clone(), Path::new("."), &mut renderer).unwrap();
    let mut playback = prepared.playback(false).unwrap();
    playback.command(PlaybackCommand::Next, Duration::ZERO);
    // From rest, manual Next and the authored entry use the same complete
    // content pose. Freeze ambient phase here, since the two clocks differ.
    for time in [0., 0.06, 0.1, 0.16, 0.24, 0.4] {
        let native = prepared.tasks[0]
            .channels()
            .into_iter()
            .map(|channel| {
                let property = PropertyId::new(&channel.id);
                let a = playback.timeline().sample_at(&property, time).unwrap();
                let b = prepared.timeline.sample_at(&property, 3. + time).unwrap();
                assert_close(a, b);
                (channel.property, a)
            })
            .collect::<HashMap<_, _>>();
        let mut a = vec![0; 1920 * 1080 * 4];
        let mut b = a.clone();
        prepared.tasks[0]
            .render(&mut a, &mut renderer, 0., |_, property| {
                native.get(property).copied()
            })
            .unwrap();
        prepared.tasks[0]
            .render(&mut b, &mut renderer, 0., |actor, property| {
                prepared.raw_motion_value(&prepared.timeline, actor, property, 3. + time)
            })
            .unwrap();
        assert_eq!(a, b, "content mismatch at {time}");
    }
    playback.pause(Duration::from_millis(120));
    let paused = playback.sample(Duration::from_secs(10));
    assert_eq!(paused, playback.sample(Duration::from_secs(20)));
    playback.set_reduced_motion(true, Duration::from_secs(20));
    let held = playback.sample(Duration::from_secs(20));
    assert_eq!(held, playback.sample(Duration::from_secs(30)));
    for channel in prepared.tasks[0].channels() {
        let property = PropertyId::new(&channel.id);
        assert_eq!(
            playback
                .timeline()
                .sample_at(&property, held.at_nanos as f64 / 1e9),
            prepared.timeline.sample_at(&property, 5.)
        );
    }
    // Event boundaries preserve pixels, not only endpoints, for exit, result,
    // reset, failure, and retry. The first sub-millisecond sample cannot pop.
    for time in [3., 6., 9., 15., 18.] {
        let before = prepared
            .render_sample(&mut renderer, time - 0.000001)
            .unwrap();
        let at = prepared.render_sample(&mut renderer, time).unwrap();
        let after = prepared
            .render_sample(&mut renderer, time + 0.000001)
            .unwrap();
        assert!(before.iter().zip(&at).all(|(a, b)| a.abs_diff(*b) <= 1));
        assert!(after.iter().zip(&at).all(|(a, b)| a.abs_diff(*b) <= 1));
    }
    // Retrying an error must not discard the bubble with the faster icon fade.
    let mut bubble = vec![0; 1920 * 1080 * 4];
    let mut without_bubble = bubble.clone();
    prepared.tasks[0]
        .render(&mut bubble, &mut renderer, 18.31, |actor, property| {
            let mut state =
                prepared.raw_motion_value(&prepared.timeline, actor, property, 18.31)?;
            if property.starts_with("content.") && property.ends_with(".opacity") {
                state.position = 0.;
            }
            Some(state)
        })
        .unwrap();
    prepared.tasks[0]
        .render(
            &mut without_bubble,
            &mut renderer,
            18.31,
            |actor, property| {
                let mut state =
                    prepared.raw_motion_value(&prepared.timeline, actor, property, 18.31)?;
                if (property.starts_with("content.") || property.starts_with("bubble."))
                    && property.ends_with(".opacity")
                {
                    state.position = 0.;
                }
                Some(state)
            },
        )
        .unwrap();
    assert_ne!(
        bubble, without_bubble,
        "the independent bubble fade was cut off with its icon"
    );
    let prepared =
        PreparedPlan::prepare(deck.slides[2].plan.clone(), Path::new("."), &mut renderer).unwrap();
    let before = prepared.render_sample(&mut renderer, 9.).unwrap();
    let after = prepared.render_sample(&mut renderer, 9.2).unwrap();
    // loadUser has already succeeded; only loadSettings fails on this step.
    for y in 390..635 {
        let start = (y * 1920 + 440) * 4;
        let end = (y * 1920 + 640) * 4;
        assert_eq!(
            &before[start..end],
            &after[start..end],
            "unchanged Task row {y}"
        );
    }
}

fn artifact(prepared: &PreparedPlan, renderer: &mut HeadlessRenderer, at: f64, name: &str) {
    if std::env::var_os("PSYCHOPOMP_STABILITY_ARTIFACTS").is_some() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/stability-proof");
        std::fs::create_dir_all(&directory).unwrap();
        delivery::render_frame(
            prepared,
            renderer,
            &directory.join(format!("{name}.png")),
            Time::try_seconds(at).unwrap(),
            false,
            crate::exposure::FrameRate::default(),
        )
        .unwrap();
    }
}

#[test]
#[ignore = "requires a headless GPU; checks actual shaped target geometry and reversal pixels"]
fn attachments_follow_inline_layout_and_preserve_interruption_pixels() {
    let plan = attached_plan();
    let mut renderer = pollster::block_on(new_renderer(&plan.id)).unwrap();
    let prepared = PreparedPlan::prepare(plan, Path::new("."), &mut renderer).unwrap();
    renderer.set_file_name(prepared.file_name());
    let visible = CodeLine::new(
        "visible",
        vec![StyledSpan::new("const magicWord =", SyntaxStyle::Plain)],
    );
    let expected = renderer.measure_text_range(&visible, " =").unwrap();
    let initial = prepared
        .motion_value(&prepared.timeline, "editor", "highlight-x", 0.)
        .unwrap();
    assert!((initial.position - expected.x).abs() < 0.001);
    let expanded = prepared
        .motion_value(&prepared.timeline, "editor", "highlight-x", 10.5)
        .unwrap();
    assert!(expanded.position - initial.position > 500.);
    for time in [0.0, 5.15, 7.15, 9.15, 12.5, 5.15] {
        let state = prepared
            .motion_value(&prepared.timeline, "editor", "highlight-x", time)
            .unwrap();
        let geometry = prepared
            .editor()
            .unwrap()
            .target_motion("equals", |actor, property| {
                prepared.raw_motion_value(&prepared.timeline, actor, property, time)
            })
            .unwrap();
        assert_close(state, geometry.x);
        if time > 0. && state.velocity.abs() > 0.01 {
            let left = prepared
                .motion_value(&prepared.timeline, "editor", "highlight-x", time - 0.001)
                .unwrap()
                .position;
            let right = prepared
                .motion_value(&prepared.timeline, "editor", "highlight-x", time + 0.001)
                .unwrap()
                .position;
            assert!((state.velocity - (right - left) / 0.002).abs() < 0.5);
        }
    }
    artifact(&prepared, &mut renderer, 0., "attached-initial");
    artifact(&prepared, &mut renderer, 5.15, "attached-mid-reveal");
    artifact(&prepared, &mut renderer, 10.5, "attached-expanded");

    renderer.set_interactive_preview(true);
    let mut playback = prepared.playback(false).unwrap();
    for _ in 0..5 {
        playback.command(PlaybackCommand::Next, Duration::ZERO);
    }
    let old = playback.timeline();
    let before = prepared
        .render_sample_using(&mut renderer, 0.15, &old)
        .unwrap();
    playback.command(PlaybackCommand::Previous, Duration::from_millis(150));
    let timeline = playback.timeline();
    assert_close(
        prepared
            .motion_value(&old, "editor", "highlight-x", 0.15)
            .unwrap(),
        prepared
            .motion_value(&timeline, "editor", "highlight-x", 0.15)
            .unwrap(),
    );
    assert_eq!(
        before,
        prepared
            .render_sample_using(&mut renderer, 0.15, &timeline)
            .unwrap()
    );
    prepared
        .render_sample_using(&mut renderer, 2.0, &timeline)
        .unwrap();
    assert_eq!(
        before,
        prepared
            .render_sample_using(&mut renderer, 0.15, &timeline)
            .unwrap()
    );
}

#[test]
#[ignore = "requires a headless GPU; verifies retargets between semantic and literal coordinates"]
fn attachment_switches_keep_position_and_velocity_in_live_playback() {
    let mut plan = attached_plan();
    let channel = plan
        .continuous_channels
        .iter_mut()
        .find(|channel| channel.property == "highlight-x")
        .unwrap();
    for (at, goal) in [
        (3, target("name", TargetComponentPlan::CenterX, 17.)),
        (7, 42.0.into()),
        (9, target("equals", TargetComponentPlan::X, 10.)),
    ] {
        channel.events.push(TrackEventPlan::Spring {
            at_nanos: at * 1_000_000_000,
            target: goal,
            response_seconds: 0.48,
            damping_ratio: 1.,
            position_threshold: 0.001,
            velocity_threshold: 0.001,
        });
    }
    let mut renderer = pollster::block_on(new_renderer(&plan.id)).unwrap();
    let prepared = PreparedPlan::prepare(plan, Path::new("."), &mut renderer).unwrap();
    assert_eq!(
        prepared.motion_value(&prepared.timeline, "editor", "highlight-x", 8.5),
        Some(MotionState::at(42.))
    );
    let mut playback = prepared.playback(false).unwrap();
    for (time, command) in [
        (0., PlaybackCommand::Last),
        (0.12, PlaybackCommand::Previous),
        (0.20, PlaybackCommand::Previous),
        (0.27, PlaybackCommand::First),
        (0.32, PlaybackCommand::Last),
    ] {
        let local = playback.sample(Duration::from_secs_f64(time)).at_nanos as f64 / 1e9;
        let before = prepared
            .motion_value(&playback.timeline(), "editor", "highlight-x", local)
            .unwrap();
        playback.command(command, Duration::from_secs_f64(time));
        assert_close(
            before,
            prepared
                .motion_value(&playback.timeline(), "editor", "highlight-x", local)
                .unwrap(),
        );
    }
}

#[test]
#[ignore = "requires a headless GPU; proves multi-step insertion, removal, re-entry, and attachment"]
fn keyed_line_steps_retain_identity_and_retarget_without_pixel_jumps() {
    let mut plan = attached_plan();
    let mut recipe: EditorRecipePlan = serde_json::from_value(plan.actors[0].data.clone()).unwrap();
    let initial = recipe.initial_line_ids.clone();
    recipe.lines.push(EditorLinePlan {
        id: "inserted".into(),
        parts: vec![EditorPartPlan {
            id: "body".into(),
            spans: vec![StyledSpan::new("const ready = true", SyntaxStyle::Plain)],
        }],
        semantic_ranges: Vec::new(),
        mark: None,
    });
    let mut inserted = initial.clone();
    inserted.insert(2, "inserted".into());
    let compact = vec!["inserted".into(), "magicWord".into(), "value".into()];
    recipe.snapshots = vec![
        EditorSnapshotPlan {
            at_nanos: 1_000_000_000,
            line_ids: inserted,
        },
        EditorSnapshotPlan {
            at_nanos: 3_000_000_000,
            line_ids: compact,
        },
        EditorSnapshotPlan {
            at_nanos: 5_000_000_000,
            line_ids: initial,
        },
    ];
    plan.actors[0].data = serde_json::to_value(recipe).unwrap();
    plan.presentation_steps = [
        ("initial", 0, 0),
        ("insert", 1_000_000_000, 2_500_000_000),
        ("remove", 3_000_000_000, 4_500_000_000),
        ("restore", 5_000_000_000, 7_000_000_000),
    ]
    .into_iter()
    .map(|(id, start, hold)| PresentationStepPlan {
        id: id.into(),
        title: id.into(),
        start_nanos: start,
        hold_nanos: hold,
    })
    .collect();
    let mut renderer = pollster::block_on(new_renderer(&plan.id)).unwrap();
    if std::env::var_os("PSYCHOPOMP_STABILITY_ARTIFACTS").is_some() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../output/stability-proof");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("keyed-lines.plan.json"),
            plan.to_json_pretty().unwrap(),
        )
        .unwrap();
    }
    let prepared = PreparedPlan::prepare(plan, Path::new("."), &mut renderer).unwrap();
    renderer.set_file_name(prepared.file_name());
    for (time, expected_y) in [(0., 88.), (2.5, 132.), (4.5, 44.), (7., 88.)] {
        let attached = prepared
            .motion_value(&prepared.timeline, "editor", "highlight-y", time)
            .unwrap();
        assert!(
            (attached.position - expected_y).abs() < 0.01,
            "{time}: {attached:?}"
        );
    }
    artifact(&prepared, &mut renderer, 2.5, "line-inserted");
    artifact(&prepared, &mut renderer, 4.5, "lines-removed");
    artifact(&prepared, &mut renderer, 7., "lines-restored");
    renderer.set_interactive_preview(true);
    let mut playback = prepared.playback(false).unwrap();
    for (time, command) in [
        (0., PlaybackCommand::Next),
        (0.1, PlaybackCommand::Next),
        (0.18, PlaybackCommand::Previous),
        (0.23, PlaybackCommand::Last),
        (0.31, PlaybackCommand::First),
    ] {
        let local = playback.sample(Duration::from_secs_f64(time)).at_nanos as f64 / 1e9;
        let old = playback.timeline();
        let before = prepared
            .render_sample_using(&mut renderer, local, &old)
            .unwrap();
        playback.command(command, Duration::from_secs_f64(time));
        for channel in prepared
            .plan
            .continuous_channels
            .iter()
            .filter(|channel| channel.property.starts_with("line."))
        {
            let property = psychopomp::timeline::PropertyId::new(&channel.id);
            assert_close(
                old.sample_at(&property, local).unwrap(),
                playback.timeline().sample_at(&property, local).unwrap(),
            );
        }
        assert_eq!(
            before,
            prepared
                .render_sample_using(&mut renderer, local, &playback.timeline())
                .unwrap()
        );
    }
}
