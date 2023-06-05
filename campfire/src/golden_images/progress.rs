use core::fmt;
use std::{borrow::Cow, cell::RefCell, collections::BTreeSet, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;

pub(super) struct Progress<'a> {
    in_flight: RefCell<BTreeSet<&'a str>>,
    bar: ProgressBar,
}

impl<'a> Progress<'a> {
    pub(super) fn new(name: impl Into<Cow<'static, str>>, count: u64) -> Self {
        let bar = if count == 1 {
            ProgressBar::new(count).with_style(
                ProgressStyle::with_template("{prefix} {spinner} {elapsed}: {msg}")
                    .expect("Invalid progress bar style"),
            )
        } else {
            ProgressBar::new(count).with_style(
                ProgressStyle::with_template("{prefix} {bar} {pos}/{len}: {msg}")
                    .expect("Invalid progress bar style"),
            )
        };

        bar.set_prefix(name);

        Self {
            bar,
            in_flight: Default::default(),
        }
    }

    pub fn set_in_flight(&self, s: &'a str) {
        let mut in_flight = self.in_flight.borrow_mut();
        in_flight.insert(s);
        self.bar.set_message(in_flight.iter().join(", "));
    }

    pub fn remove_in_flight(&self, s: &'a str) {
        let mut in_flight = self.in_flight.borrow_mut();
        assert!(in_flight.remove(s));
        self.bar.set_message(in_flight.iter().join(", "));
    }

    pub(super) fn println(&self, f: fmt::Arguments<'_>) {
        self.bar.suspend(|| {
            println!("{}", f);
        });
    }

    pub(super) fn inc(&self) {
        self.bar.inc(1)
    }

    pub(super) fn finish(&self) {
        self.bar.finish_and_clear();
    }

    pub fn enable_steady_tick(&self, interval: Duration) {
        self.bar.enable_steady_tick(interval)
    }
}
