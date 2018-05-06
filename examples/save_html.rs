extern crate futures;
extern crate hyper;
extern crate select;
extern crate spiders;
extern crate tokio_core;

use futures::{future, Stream};
use hyper::Uri;
use select::predicate::Name;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();

    let start_uri = "https://xkcd.com".parse::<Uri>().unwrap();
    let host = start_uri.host().unwrap().to_owned();

    let mut crawler = spiders::Crawler::new(&core.handle(), start_uri).unwrap();

    crawler.limit_host(host);

    core.run(crawler.take(1).for_each(|(_uri, document)| {
        future::ok(println!(
            "{}",
            document.find(Name("html")).next().unwrap().html()
        ))
    })).unwrap();
}
