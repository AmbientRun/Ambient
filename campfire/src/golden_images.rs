// Intentionally overrides the global Clippy settings because Campfire is not
// part of Ambient engine.
#![allow(clippy::disallowed_types)]

use anyhow::{bail, Context};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::{process::Command, time::Instant};

const TEST_PATHS: &[&str] = &[
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

// Todo: implement update and check.
// Todo: accept filters like `guest/rust/examples/ui/` for running only UI tests.
#[derive(Parser, Clone)]
pub enum GoldenImages {
    /// For each test, updates the golden image
    Update,
    /// For each test, check the current image against the committed image
    Check,
}

fn status_emoji(status: bool) -> char {
    if status {
        '✅'
    } else {
        '❌'
    }
}

#[derive(Debug)]
struct Failure {
    test_path: &'static str,
    stdout: String,
    stderr: String,
}

impl Failure {
    fn from_output(test_path: &'static str, output: &std::process::Output) -> Self {
        let stdout =
            String::from_utf8(output.stdout.clone()).expect("stdout must be a valid UTF-8");
        let stderr =
            String::from_utf8(output.stderr.clone()).expect("stderr must be a valid UTF-8");
        Failure {
            test_path,
            stdout,
            stderr,
        }
    }
    fn log(&self) {
        log::error!("{} failed", self.test_path);
        log::error!("stdout:");
        self.stdout.lines().for_each(|line| eprintln!("{line}"));
        log::error!("stderr:");
        self.stderr.lines().for_each(|line| eprintln!("{line}"));
        eprintln!(); // Space between consecutive errors.
    }
}

pub(crate) fn main(_gi: &GoldenImages) -> anyhow::Result<()> {
    let start_time = Instant::now();
    build_tests().with_context(|| format!("Building {} tests", TEST_PATHS.len()))?;
    run_tests().with_context(|| format!("Running {} tests", TEST_PATHS.len()))?;
    log::info!(
        "Running {} golden image tests took {:.03} seconds",
        TEST_PATHS.len(),
        start_time.elapsed().as_secs_f64()
    );
    Ok(())
}

fn default_progress_bar() -> ProgressBar {
    ProgressBar::new(TEST_PATHS.len() as _).with_style(
        ProgressStyle::with_template("{wide_bar} {msg} eta={eta} {pos}/{len}")
            .expect("Invalid progress bar style"),
    )
}

fn build_tests() -> anyhow::Result<()> {
    let pb = default_progress_bar();
    pb.println(format!("Building {} tests", TEST_PATHS.len()));
    let mut failures = vec![];
    for &test_path in TEST_PATHS {
        pb.set_message(test_path);
        let start_time = Instant::now();
        let program = "cargo";
        let args = ["run", "--release", "--", "build", "--release", test_path];
        let output = Command::new(program).args(args).output()?;
        if !output.status.success() {
            failures.push(Failure::from_output(test_path, &output));
        }
        pb.println(format!(
            "{} | {:.03}s | {program} {}",
            status_emoji(output.status.success()),
            start_time.elapsed().as_secs_f64(),
            args.join(" "),
        ));
        pb.inc(1);
    }
    pb.finish_and_clear();
    if !failures.is_empty() {
        for failure in &failures {
            failure.log();
        }
        bail!("{} tests failed", failures.len());
    }
    Ok(())
}

fn run_tests() -> anyhow::Result<()> {
    let pb = default_progress_bar();
    pb.println(format!("Running {} tests", TEST_PATHS.len()));
    let mut failures = vec![];
    for &test_path in TEST_PATHS {
        pb.set_message(test_path);
        let start_time = Instant::now();
        // log::info!("Testing: {test_path}");
        let program = "cargo";
        let args = [
            "run",
            "--release",
            "--",
            "run",
            "--release",
            test_path,
            "--headless",
            "--no-proxy",
            "--golden-image-test",
            // Todo: Ideally this timeout should be unnecessary, because we only
            // care about rendering the first frame of the test, no matter how
            // long it takes to load the test. Being able to stall the renderer
            // before everything has been loaded eliminates the need for
            // timeouts and reduces test flakiness.
            "60",
            "--quic-interface-port",
            "9000",
            "--http-interface-port",
            "10000",
        ];
        let output = Command::new(program).args(args).output()?;
        if !output.status.success() {
            failures.push(Failure::from_output(test_path, &output));
        }
        pb.println(format!(
            "{} | {:.03}s | {program} {}",
            status_emoji(output.status.success()),
            start_time.elapsed().as_secs_f64(),
            args.join(" "),
        ));
        pb.inc(1);
    }
    pb.finish_and_clear();
    if !failures.is_empty() {
        for failure in &failures {
            failure.log();
        }
        bail!("{} tests failed", failures.len());
    }
    Ok(())
}
