use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use clap::Args;
use futures::StreamExt;
use itertools::process_results;
use notify::{
    event::{CreateKind, RemoveKind},
    EventKind, RecursiveMode, Watcher,
};
use notify_debouncer_full::{DebounceEventResult, Debouncer, FileIdMap};
use tokio::select;

use super::build::BuildOptions;

pub struct WatcherState<W: Watcher> {
    watcher: Debouncer<W, FileIdMap>,
    watching: BTreeSet<PathBuf>,
}

impl<W: Watcher> WatcherState<W> {
    pub fn new(watcher: Debouncer<W, FileIdMap>) -> Self {
        Self {
            watcher,
            watching: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path
            .as_ref()
            .canonicalize()
            .context("Failed to canonicalize path")?;

        if self.watching.insert(path.to_path_buf()) {
            log::debug!("Watching new entry: {path:?}");
            self.watcher
                .watcher()
                .watch(&path, RecursiveMode::NonRecursive)?;
        }

        Ok(())
    }

    pub fn remove(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref().canonicalize()?;

        if self.watching.remove(&path) {
            log::debug!("Watching new entry: {path:?}");
            self.watcher.watcher().unwatch(&path)?;
        }

        Ok(())
    }

    pub fn _update_subdir(&mut self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        for entry in find_watched_dirs(dir) {
            let entry = entry?;

            self.add(entry.path())?;
        }

        Ok(())
    }
}

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

        self.build.build().await?;

        let watch = self.watch_and_build();
        let serve = self.serve();

        select! {
            v = serve => v?,
            v = watch => v?,
        }

        Ok(())
    }

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

        let watcher = notify_debouncer_full::new_debouncer(
            Duration::from_millis(500),
            None,
            move |event: DebounceEventResult| {
                tx.send(event).unwrap();
            },
        )?;

        let mut watcher = WatcherState::new(watcher);

        log::debug!("Created watcher");

        process_results(find_watched_dirs("."), |mut v| {
            v.try_for_each(|v| watcher.add(v.path()))
        })
        .context("Failed to watch initial root")??;

        let mut rx = rx.into_stream();
        while let Some(events) = rx.next().await {
            let events = events.map_err(|v| anyhow::anyhow!("File watch error: {v:?}"))?;

            let mut needs_rebuild = false;
            for event in events {
                match event.event.kind {
                    EventKind::Create(CreateKind::File) => {
                        for path in &event.paths {
                            log::info!("File created: {path:?}");
                            watcher.add(path)?;
                        }
                        needs_rebuild = true;
                    }
                    EventKind::Create(CreateKind::Folder) => {
                        for path in &event.paths {
                            log::debug!("Folder created: {path:?}");

                            process_results(find_watched_dirs(path), |mut v| {
                                v.try_for_each(|v| watcher.add(v.path()))
                            })
                            .context("Failed to watch new folder")??;
                        }
                        needs_rebuild = true;
                    }

                    EventKind::Modify(v) => {
                        log::debug!("Modified {v:?}");
                        needs_rebuild = true;
                    }
                    EventKind::Remove(RemoveKind::Folder) => {
                        for path in &event.paths {
                            watcher.remove(path)?;
                        }
                        needs_rebuild = true;
                    }
                    v => {
                        log::debug!("Other event: {v:?}");
                    }
                }
            }

            if needs_rebuild {
                log::debug!("Rebuilding...");
                if let Err(err) = self.build.build().await {
                    log::error!("Failed to build: {err}");
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }

                log::debug!("Finished building the web client");
            }
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

pub fn find_watched_dirs(
    dir: impl AsRef<Path>,
) -> impl Iterator<Item = Result<walkdir::DirEntry, walkdir::Error>> {
    let dir = dir.as_ref();
    log::info!("Walking directory {dir:?}");

    walkdir::WalkDir::new(dir).into_iter().filter_entry(|v| {
        let path = v.path();
        let fname = v.file_name();

        // if  path.starts_with(".") {
        //     log::debug!("Ignoring hidden path: {path:?}");
        //     return false;
        // }

        match fname.to_str() {
            None => {
                log::error!("Path is not UTF-8: {path:?}");
                false
            }
            Some("node_modules" | "target" | ".git" | "build" | "tmp" | "pkg") => false,
            Some(_) => true,
        }
    })
}
