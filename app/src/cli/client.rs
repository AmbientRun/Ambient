use ambient_audio::AudioStream;
use ambient_core::window::ExitStatus;
use ambient_native_std::asset_cache::AssetCache;
use ambient_network::native::client::ResolvedAddr;
use anyhow::Context;

use crate::client;

use super::{ProjectPath, RunCli};

pub fn handle(
    run: &RunCli,
    rt: &tokio::runtime::Runtime,
    assets: AssetCache,
    server_addr: ResolvedAddr,
    original_project_path: ProjectPath,
) -> anyhow::Result<()> {
    // Hey! listen, it is time to setup audio
    let audio_stream = if !run.mute_audio {
        log::info!("Creating audio stream");
        match AudioStream::new().context("Failed to initialize audio stream") {
            Ok(v) => Some(v),
            Err(err) => {
                log::error!("Failed to initialize audio stream: {err}");
                None
            }
        }
    } else {
        log::info!("Audio is disabled");
        None
    };

    let mixer = if run.mute_audio {
        None
    } else {
        audio_stream.as_ref().map(|v| v.mixer().clone())
    };

    // If we have run parameters, start a client and join a server
    let exit_status = rt.block_on(client::run(
        assets,
        server_addr,
        run,
        original_project_path.fs_path,
        mixer,
    ));

    // if exit_status == ExitStatus::FAILURE {
    //     anyhow::bail!("client::run failed with {exit_status:?}");
    // }

    Ok(())
}
