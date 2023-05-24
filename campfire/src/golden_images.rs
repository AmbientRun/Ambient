use clap::Parser;
use std::{process::Command, time::Instant};

const TEST_PATHS: &[&'static str] = &[
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

pub(crate) fn main(_gi: &GoldenImages) -> anyhow::Result<()> {
    let start_time = Instant::now();
    for &test_path in TEST_PATHS {
        let test_start_time = Instant::now();
        log::info!("Testing: {test_path}");
        let program = "cargo";
        let args = [
            "run",
            "--release",
            "--",
            "run",
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
        let command_str = format!("{program}, {}", args.join(" "));
        log::info!("Executing: {command_str}");
        Command::new(program).args(args).status()?;
        log::info!(
            "{test_path} took {:.03} seconds",
            test_start_time.elapsed().as_secs_f64()
        );
    }
    log::info!(
        "Running {} golden image tests took {:.03} seconds",
        TEST_PATHS.len(),
        start_time.elapsed().as_secs_f64()
    );
    Ok(())
}
