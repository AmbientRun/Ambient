use ambient_native_std::asset_cache::AssetCache;
use clap::Parser;

use crate::{cli::ClientCli, client};

use super::{serve, HostCli, PackageArgs};

#[derive(Parser, Clone, Debug)]
/// Builds and runs the package locally
pub struct Run {
    #[command(flatten)]
    pub package: PackageArgs,
    #[command(flatten)]
    pub host: HostCli,
    #[command(flatten)]
    pub run: ClientCli,
}

pub fn handle(
    rt: &tokio::runtime::Runtime,
    args: &Run,
    assets: AssetCache,
    release_build: bool,
) -> anyhow::Result<()> {
    let server_handle = rt.block_on(serve::handle_inner(
        &args.package,
        &args.host,
        assets.clone(),
        release_build,
    ))?;

    let package_path = args.package.package_path()?;
    client::run(
        rt,
        assets,
        server_handle.resolve_as_localhost(),
        &args.run,
        package_path.fs_path,
    )
}
