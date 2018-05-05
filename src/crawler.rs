use futures::{Future, Stream};
use hyper::{client::HttpConnector, Client, Error as HyperError, Uri};
use hyper_tls::HttpsConnector;
use select::{document::Document, predicate::Name};
use tokio_core::reactor::Handle;

const VISIT_LIMIT: usize = 10;

#[derive(Debug)]
pub struct Crawler {
    handle: Handle,
    visited: usize,
    http_client: Client<HttpsConnector<HttpConnector>>,
}

impl Crawler {
    pub fn new(handle: &Handle) -> Self {
        Self {
            handle: handle.clone(),
            visited: 0,
            http_client: Client::configure()
                // TODO Not use unwrap + not hardcode 4
                .connector(HttpsConnector::new(4, handle).unwrap())
                .build(handle),
        }
    }

    pub fn visit(&mut self, uri: Uri) -> Option<impl Future<Item = Vec<Uri>, Error = HyperError>> {
        if self.visited >= VISIT_LIMIT {
            return None;
        } else {
            self.visited += 1;
        }

        Some(
            self.http_client
                .get(uri)
                .and_then(|response| response.body().collect())
                .map(|chunks| {
                    // TODO: Use something smarter than reading the whole body into memory.
                    let body = chunks
                        .into_iter()
                        .map(|chunk| String::from_utf8_lossy(&chunk).into_owned())
                        .collect::<String>();

                    let document = Document::from(body.as_str());

                    let links = document
                        .find(Name("a"))
                        .filter_map(|node| node.attr("href"));

                    // TODO: Handle relative links.
                    links
                        .filter_map(|link| link.parse::<Uri>().ok())
                        .collect::<Vec<_>>()
                }),
        )
    }
}
