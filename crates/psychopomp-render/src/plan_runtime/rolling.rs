//! Rolling Numbers: the recipe is decoded and its channels checked during
//! preflight; preparation measures its glyphs and compiles every change into
//! closed-form tracks once.
use anyhow::{Result, ensure};
use psychopomp::{
    plan::{ActorPlan, ContinuousChannelPlan},
    rolling::{CompiledRoll, RollingNumberPlan},
};

use super::preflight::{decode, strict_channels};
use crate::render::HeadlessRenderer;

pub(super) struct RollingNumberInput {
    id: String,
    plan: RollingNumberPlan,
}

impl RollingNumberInput {
    pub(super) fn new(
        actor: &ActorPlan,
        channels: &[ContinuousChannelPlan],
        duration_nanos: u64,
    ) -> Result<Self> {
        let plan = decode(actor, "rolling number", RollingNumberPlan::validate)?;
        ensure!(
            plan.rolls
                .last()
                .is_none_or(|roll| roll.at_nanos <= duration_nanos),
            "rolling number actor '{}' changes after the scene ends",
            actor.id
        );
        strict_channels(&actor.id, channels, "rolling number", |property| {
            matches!(property, "opacity" | "x" | "y")
        })?;
        Ok(Self {
            id: actor.id.clone(),
            plan,
        })
    }

    pub(super) fn prepare(self, renderer: &mut HeadlessRenderer) -> PreparedRollingNumber {
        let roll = renderer.compile_rolling_number(&self.plan);
        PreparedRollingNumber {
            id: self.id,
            plan: self.plan,
            roll,
        }
    }
}

pub(super) struct PreparedRollingNumber {
    id: String,
    plan: RollingNumberPlan,
    roll: CompiledRoll,
}

impl PreparedRollingNumber {
    /// Wheels are masked to one stationary row; symbols use the sprite support.
    pub(super) fn ink_rows(&self, sample: impl Fn(&str, &str, f32) -> f32) -> Option<[f32; 2]> {
        if sample(&self.id, "opacity", 1.0).clamp(0.0, 1.0) <= 0.001 {
            return None;
        }
        let y = self.plan.origin[1] + sample(&self.id, "y", 0.0);
        let height = (self.plan.size * 1.5).ceil();
        let sprite_y = y - height * 0.5;
        let mut top = (sprite_y - 1.0).min(y - self.plan.row_height() * 0.5);
        let mut bottom = (sprite_y + height + 1.0).max(y + self.plan.row_height() * 0.5);
        if self.plan.chip {
            let caption =
                psychopomp::caption::CaptionPlan::line([0.0, 0.0], self.plan.size, Vec::new());
            let half = caption.line_height() * 0.5 + 4.0;
            top = top.min(y - half);
            bottom = bottom.max((y - half) + half * 2.0);
        }
        Some([top, bottom])
    }

    /// Samples differ while a change settles, even with no channel moving.
    pub(super) fn moving(&self, time: f64) -> bool {
        self.roll.moving(time)
    }

    pub(super) fn render(
        &self,
        pixels: &mut [u8],
        renderer: &mut HeadlessRenderer,
        time: f64,
        sample: impl Fn(&str, &str, f32) -> f32,
    ) {
        renderer.composite_rolling_number(pixels, &self.plan, &self.roll, time, |property, d| {
            sample(&self.id, property, d)
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::plan_runtime::validate_renderer_plan;
    use psychopomp::{
        author::PlanBuilder,
        rolling::{RollingNumberActor, RollingNumberPlan},
    };

    #[test]
    fn preflight_rejects_unknown_channels_and_late_changes_without_a_gpu() {
        let build = |property: &str, at: u64| {
            let mut scene = PlanBuilder::new("rolling", 2_000_000_000);
            let mut number = RollingNumberActor::declare(
                &mut scene,
                "count",
                RollingNumberPlan::new([960.0, 540.0], 64.0, "0/8").roll(at, "8/8"),
            )
            .unwrap();
            number.channel(&mut scene, property, 1.0);
            scene.finish().unwrap()
        };
        validate_renderer_plan(&build("opacity", 1_000_000_000)).unwrap();
        assert!(validate_renderer_plan(&build("typed", 1_000_000_000)).is_err());
        assert!(validate_renderer_plan(&build("opacity", 3_000_000_000)).is_err());
    }
}
