extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate select;
extern crate tokio_core;

use futures::Future;

mod crawler;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let mut crawler = crawler::Crawler::new(&core.handle());

    core.run(
        crawler
            .visit("https://xkcd.com".parse().unwrap())
            .unwrap()
            .map(|uris| println!("{:#?}", uris)),
    ).unwrap();
}
