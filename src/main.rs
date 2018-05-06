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
use hyper::Uri;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();

    let start_uri = "https://xkcd.com".parse::<Uri>().unwrap();
    let host = start_uri.host().unwrap().to_owned();

    let mut crawler = crawler::Crawler::new(&core.handle(), start_uri).unwrap();

    crawler.limit_host(host);

    core.run(
        crawler
            .take(32)
            .for_each(|(uri, _document)| future::ok(println!("we visited {}", uri))),
    ).unwrap();
}
