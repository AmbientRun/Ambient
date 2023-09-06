use std::{path::Path, time::Duration};

use anyhow::Context;
use clap::Args;
use itertools::Itertools;
use notify::{RecursiveMode, Watcher};

use super::build::BuildOptions;

#[derive(Debug, Args, Clone)]
pub struct Serve {
    #[clap(flatten)]
    build: BuildOptions,
}

impl Serve {
    pub async fn run(&self) -> anyhow::Result<()> {
        if !tokio::fs::try_exists("web/www/node_modules")
            .await
            .context("Failed to query node_modules directory")?
        {
            log::info!("Installing node modules");
            tokio::process::Command::new("npm")
                .args(["install", "-d"])
                .current_dir("web/www")
                .kill_on_drop(true)
                .spawn()
                .context("Failed to spawn npm")?
                .wait()
                .await
                .context("Failed to run npm install")?;
        }

        let serve = self.serve();
        let watch = self.watch_and_build();

        tokio::select! {
            v = serve => v?,
            v = watch => v?,
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn serve(&self) -> anyhow::Result<()> {
        let dir = Path::new("web/www")
            .canonicalize()
            .context("Web server directory does not exist")?;

        // De-UNC the path.
        let working_dir = dunce::simplified(&dir).to_owned();

        let status = tokio::process::Command::new("cmd")
            .args(["/C", "npm", "run", "dev"])
            .current_dir(working_dir)
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn npm")?
            .wait()
            .await
            .context("Failed to run dev web server")?;

        if !status.success() {
            anyhow::bail!("Web server exited with non-zero status: {status:?}")
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    pub async fn serve(&self) -> anyhow::Result<()> {
        let dir = Path::new("web/www")
            .canonicalize()
            .context("Web server directory does not exist")?;

        let status = tokio::process::Command::new("npm")
            .args(["run", "dev"])
            .current_dir(dir)
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn npm")?
            .wait()
            .await
            .context("Failed to run dev web server")?;

        if !status.success() {
            anyhow::bail!("Web server exited with non-zero status: {status:?}")
        }

        Ok(())
    }

    pub async fn watch_and_build(&self) -> anyhow::Result<()> {
        let (tx, rx) = flume::unbounded();

        // Kick off initial build
        let _ = tx.send(vec![]);

        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                if let Ok(event) = event {
                    let paths = event
                        .paths
                        .into_iter()
                        .filter(|v| filter_path(v))
                        .collect_vec();

                    // for path in event.paths.iter() {
                    //     if filter_path(path) {
                    //         log::info!("Event: {path:?}");
                    //         found = true;
                    //     }
                    // }

                    if !paths.is_empty() {
                        let _ = tx.send(paths);
                    }
                }
            })?;

        for entry in find_top_level_dirs(".") {
            let entry = entry?;
            log::info!("Watching entry {entry:?}");
            watcher.watch(&entry.path(), RecursiveMode::Recursive)?;
        }

        while let Ok(mut paths) = rx.recv_async().await {
            tokio::time::sleep(Duration::from_millis(1000)).await;

            paths.extend(rx.drain().into_iter().flatten());
            log::info!("Changed paths: {paths:?}");
            log::info!("Rebuilding...");
            #[cfg(not(target_os = "windows"))]
            if let Err(err) = self.build.build().await {
                log::error!("Failed to build: {err}");
                log::info!("Finished building the web client");
            }

            #[cfg(target_os = "windows")]
            log::info!("Rebuild is disabled on windows");
        }

        Ok(())
    }
}

// pub fn update_watch_subdir(
//     watching: &mut BTreeSet<PathBuf>,
//     watcher: impl Watcher,
//     dir: impl AsRef<Path>,
// ) -> anyhow::Result<()> {
//     for entry in find_watched_dirs(dir.as_ref()) {
//         let entry = entry?;

//         let path = entry.path();
//         if watching.insert(path.to_path_buf()) {
//             log::info!("Watching new entry: {path:?}");
//             watcher.watch(entry.path(), RecursiveMode::NonRecursive)?;
//         }
//     }

//     Ok(())
// }

fn filter_path(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();

    path.components().into_iter().all(|seg| {
        let seg: &Path = seg.as_ref();
        match seg.to_str() {
            None => {
                log::error!("Path is not UTF-8: {path:?}");
                false
            }
            Some(
                "node_modules" | "target" | ".git" | "build" | "tmp" | "pkg" | "generated.rs"
                | "bindings.rs",
            ) => false,
            Some(v) if v.contains("timestamp") => false,
            Some(_) => true,
        }
    })
}

pub fn find_top_level_dirs(
    dir: impl AsRef<Path>,
) -> impl Iterator<Item = Result<std::fs::DirEntry, std::io::Error>> {
    let dir = dir.as_ref();
    log::info!("Walking directory {dir:?}");

    std::fs::read_dir(dir).unwrap().filter_map_ok(|entry| {
        let path = entry.path();

        if filter_path(path) {
            Some(entry)
        } else {
            None
        }
    })
}
