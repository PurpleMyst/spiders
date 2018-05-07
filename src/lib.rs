//! `spiders` is a simple crate that provides a web crawler that works with tokio + select.
//!
//! This currently uses `futures` version `0.1`, mostly because `hyper` and `tokio` only support
//! that version. But also because I don't like version `0.2`.
#![warn(missing_docs)]

#[macro_use]
extern crate futures;
extern crate failure;
extern crate hyper;
extern crate hyper_tls;
extern crate num_cpus;
#[macro_use]
extern crate redis_async;
extern crate select;
extern crate tokio_core;

mod crawler;

pub use crawler::Crawler;
