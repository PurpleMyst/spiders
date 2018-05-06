//! `spiders` is a simple crate that provides a web crawler that works with tokio + select.
//!
//! This currently uses `futures` version `0.1`, mostly because `hyper` and `tokio` only support
//! that version. But also because I don't like version `0.2`.
#![warn(missing_docs)]

#[macro_use]
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate num_cpus;
extern crate select;
extern crate tokio_core;

mod crawler;
mod visitor;

pub use crawler::Crawler;
