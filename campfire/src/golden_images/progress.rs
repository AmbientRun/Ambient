use std::sync::Mutex;

use indicatif::{ProgressBar, ProgressStyle};

pub(super) struct Progress(Mutex<ProgressBar>);

impl Progress {
    pub(super) fn new(count: usize) -> Self {
        Self(Mutex::new(
            ProgressBar::new(count as _).with_style(
                ProgressStyle::with_template("{msg} {wide_bar} elapsed={elapsed} eta={eta}")
                    .expect("Invalid progress bar style"),
            ),
        ))
    }
    pub(super) fn println(&self, str: impl AsRef<str>) {
        self.0.lock().unwrap().suspend(|| {
            println!("{}", str.as_ref());
        });
    }
    pub(super) fn println_and_inc(&self, str: impl AsRef<str>) {
        let pb = self.0.lock().unwrap();
        pb.suspend(|| {
            println!("{}", str.as_ref());
        });
        pb.inc(1)
    }
    pub(super) fn finish(&self) {
        self.0.lock().unwrap().finish_and_clear();
    }
}
