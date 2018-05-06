#[macro_use]
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate num_cpus;
extern crate select;
extern crate tokio_core;

mod crawler;
mod visitor;

use futures::{future, Stream};

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let crawler =
        crawler::Crawler::new(&core.handle(), "https://xkcd.com".parse().unwrap()).unwrap();

    core.run(crawler.for_each(|(uri, _document)| future::ok(println!("we visited {}", uri))))
        .unwrap();
}
