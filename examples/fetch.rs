extern crate futures;
extern crate select;
extern crate spiders;
extern crate tokio_core;

use futures::Future;

fn main() {
    let mut core = tokio_core::reactor::Core::new().unwrap();

    let crawler_future = spiders::Crawler::new(core.handle());
    core.run(crawler_future.and_then(|mut crawler| {
        crawler
            .fetch("https://httpbin.org/get".parse().unwrap())
            .map(|opt| opt.expect("no uri"))
            .map(|(document, _)| {
                println!(
                    "{}",
                    document
                        .find(select::predicate::Name("html"))
                        .next()
                        .expect("no html")
                        .text()
                )
            })
    })).unwrap();
}
