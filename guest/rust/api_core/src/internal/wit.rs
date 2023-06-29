#![allow(warnings, unused, clippy)]

include!("bindings.rs");

pub(crate) use self::{ambient::bindings::*, exports::ambient::bindings::guest};
pub(crate) use export_bindings;
