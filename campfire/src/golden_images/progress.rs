use std::borrow::Cow;

use indicatif::{ProgressBar, ProgressStyle};

pub(super) struct Progress(ProgressBar);

impl Progress {
    pub(super) fn new(count: usize) -> Self {
        Self(
            ProgressBar::new(count as _).with_style(
                ProgressStyle::with_template("{msg} {wide_bar} eta={eta} {pos}/{len}")
                    .expect("Invalid progress bar style"),
            ),
        )
    }
    pub(super) fn println(&self, str: impl AsRef<str>) {
        self.0.suspend(|| {
            println!("{}", str.as_ref());
        });
    }
    pub(super) fn set_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.0.set_message(msg);
    }
    pub(super) fn inc(&self) {
        self.0.inc(1);
    }
    pub(super) fn finish(&self) {
        self.0.finish_and_clear();
    }
}
