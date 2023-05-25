// Intentionally overrides the global Clippy settings because Campfire is not
// part of Ambient engine.
#![allow(clippy::disallowed_types)]

use anyhow::{bail, Context};
use clap::Parser;
use itertools::Itertools;
use std::{process::Command, time::Instant};

mod failure;
use failure::*;

mod progress;
use progress::*;

const TESTS: &[&str] = &[
    "guest/rust/examples/basics/async",
    "guest/rust/examples/basics/decals",
    "guest/rust/examples/basics/first_person_camera",
    "guest/rust/examples/basics/fog",
    "guest/rust/examples/basics/image",
    "guest/rust/examples/basics/input",
    "guest/rust/examples/basics/primitives",
    "guest/rust/examples/basics/procedural_generation",
    "guest/rust/examples/basics/raw_text",
    "guest/rust/examples/basics/third_person_camera",
    "guest/rust/examples/basics/transparency",
    "guest/rust/examples/games/tictactoe",
    "guest/rust/examples/ui/auto_editor",
    "guest/rust/examples/ui/button",
    "guest/rust/examples/ui/dock_layout",
    "guest/rust/examples/ui/editors",
    "guest/rust/examples/ui/flow_layout",
    "guest/rust/examples/ui/rect",
    "guest/rust/examples/ui/screens",
    "guest/rust/examples/ui/slider",
    "guest/rust/examples/ui/text",
];

// Todo: accept filters like `guest/rust/examples/ui/` for running only UI tests.
#[derive(Parser, Clone)]
pub enum GoldenImages {
    /// For each test, updates the golden image
    Update,
    /// For each test, check the current image against the committed image
    Check,
}

const TEST_NAME_PLACEHOLDER: &str = "{test}";

pub(crate) fn main(gi: &GoldenImages) -> anyhow::Result<()> {
    let start_time = Instant::now();
    run_all_tests(
        "Building",
        &[
            "run",
            "--release",
            "--",
            "build",
            "--release",
            TEST_NAME_PLACEHOLDER,
        ],
    )?;
    match gi {
        GoldenImages::Update => {
            run_all_tests(
                "Updating",
                &[
                    "run",
                    "--release",
                    "--",
                    "run",
                    "--release",
                    TEST_NAME_PLACEHOLDER,
                    "--headless",
                    "--no-proxy",
                    "golden-image-update",
                    // Todo: Ideally this waiting should be unnecessary, because
                    // we only care about rendering the first frame of the test,
                    // no matter how long it takes to start the test. Being able
                    // to stall the renderer before everything has been loaded
                    // eliminates the need for timeouts and reduces test
                    // flakiness.
                    "--wait-seconds",
                    "5.0",
                ],
            )?;
        }
        GoldenImages::Check => {
            run_all_tests(
                "Checking",
                &[
                    "run",
                    "--release",
                    "--",
                    "run",
                    "--release",
                    TEST_NAME_PLACEHOLDER,
                    "--headless",
                    "--no-proxy",
                    "golden-image-check",
                    // Todo: See notes on --wait-seconds from above.
                    "--timeout-seconds",
                    "30.0",
                ],
            )
            .context(
                "Checking failed, possible causes: \
                - Missing golden image: consider running `cargo cf golden-images update` first. \
                - Golden image differs: investigate if the difference was intentional.",
            )?;
        }
    }
    log::info!(
        "Running {} golden image tests took {:.03} seconds",
        TESTS.len(),
        start_time.elapsed().as_secs_f64()
    );
    Ok(())
}

fn run_all_tests(command: &str, args: &[&str]) -> anyhow::Result<()> {
    let pb = Progress::new(TESTS.len());
    pb.println(format!("{command} {} tests", TESTS.len()));
    let mut failures = vec![];
    for &test in TESTS {
        pb.set_message(test);
        let start_time = Instant::now();
        let args = args
            .iter()
            .map(|&arg| {
                if arg == TEST_NAME_PLACEHOLDER {
                    test.to_string()
                } else {
                    arg.to_string()
                }
            })
            .collect_vec();
        let output = Command::new("cargo").args(&args).output()?;
        if !output.status.success() {
            failures.push(Failure::from_output(test, &output));
        }
        pb.println(format!(
            "{} | {:.03}s | cargo {}",
            status_emoji(output.status.success()),
            start_time.elapsed().as_secs_f64(),
            args.join(" "),
        ));
        pb.inc();
    }
    pb.finish();
    if !failures.is_empty() {
        for failure in &failures {
            failure.log();
        }
        bail!("{} tests failed", failures.len());
    }
    Ok(())
}

fn status_emoji(status: bool) -> char {
    if status {
        '✅'
    } else {
        '❌'
    }
}
