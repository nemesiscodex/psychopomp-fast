//! Prepared captions: decoded and validated once; channel names are strict.
use anyhow::Result;
use psychopomp::{
    caption::CaptionPlan,
    plan::{ActorPlan, ContinuousChannelPlan},
};

use super::preflight::{decode, strict_channels};
use crate::render::HeadlessRenderer;

pub(super) struct PreparedCaption {
    id: String,
    plan: CaptionPlan,
}

impl PreparedCaption {
    /// Full-width support of the caption's text, chip and caret raster loops.
    pub(super) fn ink_rows(&self, sample: impl Fn(&str, &str, f32) -> f32) -> Option<[f32; 2]> {
        if sample(&self.id, "opacity", 1.0).clamp(0.0, 1.0) <= 0.001 {
            return None;
        }
        let y = self.plan.origin[1] + sample(&self.id, "y", 0.0);
        let height = (self.plan.size * 1.5).ceil();
        let line = self.plan.line_height();
        let last = line * (self.plan.lines.len() as f32 - 1.0);
        let mut top = y - height * 0.5 - 1.0;
        let mut bottom = (y + last - height * 0.5) + height + 1.0;
        if self.plan.chip {
            top = top.min(y - line * 0.5 - 4.0);
            bottom = bottom.max(y + line * (self.plan.lines.len() as f32 - 0.5) + 4.0);
        }
        if sample(&self.id, "caret", 0.0).clamp(0.0, 1.0) > 0.001 {
            let half = self.plan.size * 1.08 * 0.5;
            top = top.min(y - half);
            bottom = bottom.max((y + last - half) + self.plan.size * 1.08);
        }
        Some([top, bottom])
    }

    pub(super) fn new(actor: &ActorPlan, channels: &[ContinuousChannelPlan]) -> Result<Self> {
        let plan = decode(actor, "caption", CaptionPlan::validate)?;
        strict_channels(&actor.id, channels, "caption", |property| {
            matches!(property, "opacity" | "x" | "y" | "typed" | "caret")
        })?;
        Ok(Self {
            id: actor.id.clone(),
            plan,
        })
    }

    pub(super) fn render(
        &self,
        pixels: &mut [u8],
        renderer: &mut HeadlessRenderer,
        sample: impl Fn(&str, &str, f32) -> f32,
    ) {
        renderer.composite_caption(pixels, &self.plan, |property, default| {
            sample(&self.id, property, default)
        });
    }
}
