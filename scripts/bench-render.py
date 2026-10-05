#!/usr/bin/env python3
"""Compare two release renderers on one export window, including decoded pixels."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time


def positive_int(value):
    number = int(value)
    if number < 1:
        raise argparse.ArgumentTypeError("must be at least one")
    return number


def frame_hashes(video):
    result = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", str(video), "-map", "0:v:0",
         "-f", "framemd5", "-"],
        check=True, capture_output=True, text=True,
    )
    return [line for line in result.stdout.splitlines()
            if line and not line.startswith("#")]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("plan", type=Path)
    parser.add_argument("--range", required=True, dest="window")
    parser.add_argument("--theme", default="original")
    parser.add_argument("--fps", type=positive_int,
                        help="output FPS from 1 to 1000; omitted uses renderer default")
    parser.add_argument("--runs", type=positive_int, default=5)
    parser.add_argument("--output-dir", type=Path,
                        default=Path("target/render-benchmark"))
    args = parser.parse_args()
    if args.fps is not None and args.fps > 1000:
        parser.error("--fps must be from 1 to 1000")
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    plan = args.plan.resolve()
    binaries = {"baseline": args.baseline.resolve(),
                "candidate": args.candidate.resolve()}
    # Alternate order to spread changing background load across both versions.
    measurements = []
    for iteration in range(args.runs + 1):
        order = ["baseline", "candidate"]
        if iteration % 2:
            order.reverse()
        for label in order:
            command = [str(binaries[label]), "plan", "render", str(plan),
                       str(output / f"{label}.mp4"), "--range", args.window,
                       "--theme", args.theme]
            if args.fps is not None:
                command.extend(["--fps", str(args.fps)])
            started = time.perf_counter()
            result = subprocess.run(command, capture_output=True, text=True)
            seconds = time.perf_counter() - started
            if result.returncode:
                raise RuntimeError(f"{label} render failed:\n{result.stderr}")
            measurements.append(dict(iteration=iteration, label=label,
                                     seconds=seconds, command=command,
                                     stderr=result.stderr))
            print(f"{label} {'warmup' if iteration == 0 else iteration}: "
                  f"{seconds:.3f}s", flush=True)
    medians = {label: statistics.median(
        row["seconds"] for row in measurements
        if row["label"] == label and row["iteration"] > 0
    ) for label in binaries}
    before = frame_hashes(output / "baseline.mp4")
    after = frame_hashes(output / "candidate.mp4")
    identical = bool(before) and before == after
    report = dict(plan=str(plan), planSha256=hashlib.sha256(plan.read_bytes()).hexdigest(),
                  binarySha256={label: hashlib.sha256(binary.read_bytes()).hexdigest()
                                for label, binary in binaries.items()},
                  host=dict(platform=platform.platform(), cpuCount=os.cpu_count()),
                  window=args.window, theme=args.theme, fps=args.fps or 60, medians=medians,
                  speedup=medians["baseline"] / medians["candidate"],
                  decodedFrames=len(before), identicalDecodedFrames=identical,
                  measurements=measurements)
    (output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"Speedup: {report['speedup']:.2f}x; "
          f"{len(before)} decoded frames identical: {identical}")
    if not identical:
        raise SystemExit("decoded frame comparison failed")


if __name__ == "__main__":
    main()
