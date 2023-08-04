use std::{
    collections::{BTreeSet, HashSet},
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Args;
use futures::StreamExt;
use itertools::process_results;
use notify::{EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::{DebounceEventResult, Debouncer, FileIdMap};
use walkdir::DirEntry;

use super::build::{self, BuildOptions};

pub struct WatcherState<W: Watcher> {
    watcher: Debouncer<W, FileIdMap>,
    watching: HashSet<PathBuf>,
}

impl<W: Watcher> WatcherState<W> {
    pub fn add(&mut self, path: impl Into<PathBuf>) -> anyhow::Result<()> {
        let path = path.as_ref();

        if self.watching.insert(path.to_path_buf()) {
            log::info!("Watching new entry: {path:?}");
            self.watcher
                .watcher()
                .watch(&path, RecursiveMode::NonRecursive)?;
        }

        Ok(())
    }

    pub fn update_subdir(&mut self, dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let dir = dir.as_ref();
        for entry in find_watched_dirs(dir) {
            let entry = entry?;

            self.add(entry.path())?;
        }

        Ok(())
    }
}

impl<W, I> Extend<I> for WatcherState<W> where W: Watcher, I: Into<PathBuf> {
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        for item in iter {
            self.add(item);
        }
    }
}


impl<W > Extend<DirEntry> for WatcherState<W> where W: Watcher {
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        for item in iter {
            self.add(item.path());
        }
    }
}
#[derive(Debug, Args, Clone)]
pub struct Serve {
    #[clap(flatten)]
    build: BuildOptions,
}

impl Serve {
    pub async fn run(&self) -> anyhow::Result<()> {
        let (tx, rx) = flume::unbounded();

        let mut watcher = notify_debouncer_full::new_debouncer(
            Duration::from_millis(2000),
            None,
            move |event: DebounceEventResult| {
                tx.send(event);
            },
        )?;

        let watcher = WatcherState {
            watcher,
            watching:BTreeSet::new(),
        };


        watcher.extend(find_watched_dirs("."));

        let rx = rx.into_stream();
        while let Some(events) = rx.next().await {
            let events = events.map_err(|v| anyhow::anyhow!("File watch error: {v:?}"))?;
            for event in events {
                match event.event.kind {
                    EventKind::Any => todo!(),
                    EventKind::Access(_) => todo!(),
                    EventKind::Create(notify::event::CreateKind::File) => {
                        log::info!("File created: {path:?}");
                    }
                    EventKind::Create(notify::event::CreateKind::Folder) => {
                        log::info!("Folder created: {path:?}");
                        update_watch_subdir(watching, watcher, dir)
                    }
                    EventKind::Modify(_) => todo!(),
                    EventKind::Remove(_) => todo!(),
                    EventKind::Other => todo!(),
                }
            }
        }

        Ok(())
    }
}

pub fn update_watch_subdir(
    watching: &mut BTreeSet<PathBuf>,
    watcher: impl Watcher,
    dir: impl AsRef<Path>,
) -> anyhow::Result<()> {
    for entry in find_watched_dirs() {
        let entry = entry?;

        let path = entry.path();
        if watching.insert(path.to_path_buf()) {
            log::info!("Watching new entry: {path:?}");
            watcher.watch(entry.path(), RecursiveMode::NonRecursive)?;
        }
    }

    Ok(())
}

pub fn find_watched_dirs(dir) -> impl Iterator<Item = Result<walkdir::DirEntry, walkdir::Error>> {
    walkdir::WalkDir::new(dir).into_iter().filter_entry(|v| {
        let path = v.path();

        if path.starts_with(".") {
            log::debug!("Ignoring hidden path: {path:?}");
            return false;
        }

        match path.to_str() {
            None => {
                log::error!("Path is not UTF-8: {path:?}");
                false
            }
            Some("." | "node_modules" | "target") => false,
            Some(v) => {
                log::debug!("Watching path: {path:?}");
                true
            }
        }
    })
}
