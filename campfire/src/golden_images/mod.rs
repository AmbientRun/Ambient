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

const TEST_BASE_PATH: &str = "guest/rust/examples";
const TESTS: &[&str] = &[
    "basics/async",
    "basics/decals",
    "basics/first_person_camera",
    "basics/fog",
    "basics/image",
    "basics/input",
    "basics/primitives",
    "basics/procedural_generation",
    "basics/raw_text",
    "basics/third_person_camera",
    "basics/transparency",
    "games/tictactoe",
    "ui/auto_editor",
    "ui/button",
    "ui/dock_layout",
    "ui/editors",
    "ui/flow_layout",
    "ui/rect",
    "ui/screens",
    "ui/slider",
    "ui/text",
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
                    format!("{TEST_BASE_PATH}/{}", test)
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
