// Intentionally overrides the global Clippy settings because Campfire is not
// part of Ambient engine.
#![allow(clippy::disallowed_types)]

use anyhow::{bail, Context};
use clap::Parser;
use futures::{stream, StreamExt, TryStreamExt};
use indicatif::HumanDuration;
use itertools::Itertools;
use serde::Deserialize;
use std::{
    borrow::Cow,
    path::PathBuf,
    process::Output,
    time::{Duration, Instant},
};
use tokio::process::Command;

mod failure;
use failure::*;

mod progress;
use crate::example::all_examples;
use progress::*;

const TEST_BASE_PATH: &str = "guest/rust/examples";
const TEST_MANIFEST: &str = "golden-image-manifest.toml";

#[derive(Parser, Clone)]
pub struct GoldenImages {
    /// Only run tests which start with the specified prefix
    #[arg(long)]
    prefix: Option<String>,

    /// Path to the ambient executable
    #[arg(long)]
    ambient_path: Option<String>,

    /// Selects testing mode
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Parser, Clone)]
enum Mode {
    /// For each test, updates the golden image
    Update,
    /// For each test, check the current image against the committed image
    Check,
}

pub(crate) async fn main(gi: &GoldenImages) -> anyhow::Result<()> {
    let start_time = Instant::now();

    // Get tests.
    let tests = if let Mode::Update = gi.mode {
        all_examples(false)?
            .into_iter()
            .map(|(_, p)| p)
            .collect_vec()
    } else {
        tokio::spawn(parse_tests_from_manifest()).await??
    };

    if gi.ambient_path.is_none() {
        run("Build ambient", "", build_package, &["ambient"], true, &[]).await?;
    }

    let ambient_path = gi
        .ambient_path
        .as_ref()
        .map(|x| x as &str)
        .unwrap_or("./target/release/ambient");

    // Filter tests.
    let tests = if let Some(prefix) = &gi.prefix {
        let total_test_count = tests.len();
        let filtered_tests = tests
            .into_iter()
            .filter(|test| test.starts_with(prefix))
            .collect_vec();
        log::info!(
            "--prefix {prefix} resulted in {} out of {total_test_count} tests",
            filtered_tests.len(),
        );
        filtered_tests
    } else {
        tests
    };

    if tests.is_empty() {
        bail!("Nothing to do!");
    }

    // Build tests.
    run("Building", ambient_path, build_tests, &tests, true, &[]).await?;

    match gi.mode {
        Mode::Update => {
            run(
                "Updating",
                ambient_path,
                update_tests,
                &tests,
                true,
                &[("RUST_LOG", "info")],
            )
            .await?;
        }
        Mode::Check => {
            run(
                "Checking",
                ambient_path,
                check_tests,
                &tests[..],
                false,
                &[("RUST_LOG", "info"), ("RUST_BACKTRACE", "1")],
            )
            .await
            .context(
                "Checking failed, possible causes:
    - Golden image differs: investigate if the difference was intentional.
    - Missing golden image: consider running `cargo cf golden-images update` first.
",
            )?;
        }
    }

    println!(
        "Running {} golden image tests took {:.03} seconds",
        tests.len(),
        start_time.elapsed().as_secs_f64()
    );

    Ok(())
}

#[derive(Deserialize)]
struct Manifest {
    tests: Vec<String>,
}

async fn parse_tests_from_manifest() -> anyhow::Result<Vec<String>> {
    let manifest_path = PathBuf::from(TEST_BASE_PATH).join(TEST_MANIFEST);
    let manifest = tokio::fs::read_to_string(&manifest_path)
        .await
        .context("Failed to read test manifest")?;

    let manifest: Manifest = toml::from_str(&manifest)?;
    log::info!(
        "Read manifest from '{}', parsed {} tests",
        manifest_path.display(),
        manifest.tests.len()
    );
    Ok(manifest.tests)
}

fn build_package(_i: usize, _: &str, name: &str) -> (String, Vec<String>) {
    let args = vec![
        "build".to_string(),
        "--release".to_string(),
        "--package".to_string(),
        name.to_string(),
    ];

    ("cargo".to_string(), args)
}

fn build_tests(_i: usize, ambient_path: &str, name: &str) -> (String, Vec<String>) {
    let test_path = format!("{TEST_BASE_PATH}/{name}");

    let args = ["build".to_string(), "--release".to_string(), test_path];

    (ambient_path.to_string(), args.to_vec())
}

fn update_tests(i: usize, ambient_path: &str, name: &str) -> (String, Vec<String>) {
    let test_path = format!("{TEST_BASE_PATH}/{name}");
    let quic_port = (9000 + i as u16).to_string();
    let http_port = (10000 + i as u16).to_string();

    let args = vec![
        "run".to_string(),
        "--release".to_string(),
        test_path,
        "--headless".to_string(),
        "--no-proxy".to_string(),
        "--quic-interface-port".to_string(),
        quic_port,
        "--http-interface-port".to_string(),
        http_port,
        "golden-image-update".to_string(),
        // Todo: Ideally this waiting should be unnecessary, because
        // we only care about rendering the first frame of the test,
        // no matter how long it takes to start the test. Being able
        // to stall the renderer before everything has been loaded
        // eliminates the need for timeouts and reduces test
        // flakiness.
        "--wait-seconds".to_string(),
        "30.0".to_string(),
    ];

    (ambient_path.to_string(), args)
}

fn check_tests(i: usize, ambient_path: &str, name: &str) -> (String, Vec<String>) {
    let test_path = format!("{TEST_BASE_PATH}/{name}");
    let quic_port = (9000 + i as u16).to_string();
    let http_port = (10000 + i as u16).to_string();

    let args = [
        "run".to_string(),
        "--release".to_string(),
        test_path,
        "--headless".to_string(),
        "--no-proxy".to_string(),
        "--quic-interface-port".to_string(),
        quic_port,
        "--http-interface-port".to_string(),
        http_port,
        "golden-image-check".to_string(),
        // Todo: See notes on --wait-seconds from above.
        "--timeout-seconds".to_string(),
        "30.0".to_string(),
    ];

    (ambient_path.to_string(), args.to_vec())
}

async fn run<S: AsRef<str>>(
    name: impl Into<Cow<'static, str>>,
    ambient_path: &str,
    runner: impl Fn(usize, &str, &str) -> (String, Vec<String>),
    tests: &[S],
    parallel: bool,
    env: &[(&str, &str)],
) -> anyhow::Result<()> {
    let name = name.into();
    let concurrency = if parallel { num_cpus::get() } else { 1 };

    println!("{name} {} tests across {concurrency} CPUs", tests.len(),);
    let pb = Progress::new(name, tests.len() as _);

    pb.enable_steady_tick(Duration::from_millis(100));

    let outputs: Vec<_> = stream::iter(tests)
        .enumerate()
        .map(|(i, test_name)| {
            let test_name = test_name.as_ref();

            pb.set_in_flight(test_name);
            let start_time = Instant::now();

            let (cmd, args) = runner(i, ambient_path, test_name);

            async move {
                let sh_cmd = [&cmd as &str]
                    .into_iter()
                    .chain(args.iter().map(|v| &**v))
                    .join(" ");

                let output = Command::new(cmd)
                    .envs(env.into_iter().copied())
                    .args(args)
                    .kill_on_drop(true)
                    .output()
                    .await
                    .context("Failed to spawn test")?;

                let dur = start_time.elapsed();

                Ok((test_name, sh_cmd, dur, output)) as anyhow::Result<_>
            }
        })
        .buffer_unordered(concurrency)
        // Handle the progress bar
        .map_ok(
            |(test_name, cmd, dur, output): (&str, _, Duration, Output)| {
                let dur = HumanDuration(dur);

                pb.remove_in_flight(test_name);

                let status = status_emoji(output.status.success());

                pb.println(format_args!("{status:>4} | {dur:#} | {cmd}"));
                pb.inc();

                (test_name, output)
            },
        )
        .try_collect()
        .await
        .context("Failed to spawn test command")?;

    pb.finish();

    // Collect failures.
    let failures = outputs
        .into_iter()
        .filter_map(|(test, output)| {
            if output.status.success() {
                None
            } else {
                Some(Failure::from_output(test.to_owned(), &output))
            }
        })
        .collect_vec();

    if !failures.is_empty() {
        for failure in &failures {
            eprintln!("{failure}")
        }

        log::error!(
            "Failed tests: \n    {}",
            failures.iter().map(|v| &v.test).join("\n    ")
        );

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
