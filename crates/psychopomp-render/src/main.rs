//! PROTOTYPE: deterministic motion-graphics scenes rendered headlessly with wgpu.

mod encode;
mod exposure;
mod pixel_workers;
mod plan_runtime;
mod render;
mod video;

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if let [command, rest @ ..] = arguments.as_slice()
        && command == "plan"
    {
        return plan_runtime::command(rest);
    }
    let (arguments, fps) = plan_runtime::delivery_fps(&arguments)?;
    let output = match arguments.as_slice() {
        [] => PathBuf::from("output/psychopomp-prototype.mp4"),
        [output] => PathBuf::from(output),
        _ => bail!("usage: psychopomp [output] [--fps FPS] | psychopomp plan <command>"),
    };

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }

    pollster::block_on(plan_runtime::render_builtin_hero(&output, fps))
}
