#![no_std]
#![allow(static_mut_refs)]
#![allow(static_mut_refs)]

mod logger;

pub use crate::logger::{builder, init, init_with_filter, Builder, Formatter};