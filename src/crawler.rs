use futures::{Future, Stream};
use hyper::{client::HttpConnector, Client, Error as HyperError, Uri};
use hyper_tls::HttpsConnector;
use select::{document::Document, node::Node};
use tokio_core::reactor::Handle;

use std::collections::HashSet;

const VISIT_LIMIT: usize = 10;

#[derive(Debug)]
pub struct Crawler {
    handle: Handle,
    visited: HashSet<Uri>,
    http_client: Client<HttpsConnector<HttpConnector>>,
}

impl Crawler {
    pub fn new(handle: &Handle) -> Self {
        Self {
            handle: handle.clone(),
            visited: HashSet::new(),
            http_client: Client::configure()
                // TODO Not use unwrap + not hardcode 4
                .connector(HttpsConnector::new(4, handle).unwrap())
                .build(handle),
        }
    }

    pub fn visit(
        &mut self,
        base_uri: Uri,
    ) -> Option<impl Future<Item = Vec<Uri>, Error = HyperError>> {
        if self.visited.len() >= VISIT_LIMIT || self.visited.contains(&base_uri) {
            return None;
        } else {
            self.visited.insert(base_uri.clone());
        }

        Some(
            self.http_client
                // XXX: Can we remove this clone?
                .get(base_uri.clone())
                .and_then(|response| response.body().collect())
                .map(move |chunks| {
                    // TODO: Use something smarter than reading the whole body into memory.
                    let body = chunks
                        .into_iter()
                        .map(|chunk| String::from_utf8_lossy(&chunk).into_owned())
                        .collect::<String>();

                    let document = Document::from(body.as_str());

                    let links = document
                        .find(|node: &Node| node.html().starts_with("<a"))
                        .filter_map(|node| node.attr("href"));

                    links
                        .filter_map(|link| link.parse::<Uri>().ok())
                        .filter_map(|uri| {
                            if uri.is_absolute() {
                                Some(uri)
                            } else {
                                let new_url = 
                                    if uri.path().starts_with("//") {
                                        format!("{}:{}", base_uri.scheme()?, uri)
                                    } else if uri.to_string() == "#" {
                                        // TODO: implement a better check for this.
                                        // We could just return `base_uri.to_string()`, but we've obviously already visited that so we don't care.
                                        return None;
                                    } else {
                                        format!(
                                            "{}://{}{}",
                                            uri.scheme().or(base_uri.scheme())?,
                                            uri.authority().or(base_uri.host())?,
                                            uri.path(),
                                        )
                                    };

                                new_url.parse::<Uri>().ok()
                            }
                        })
                        .collect::<Vec<_>>()
                }),
        )
    }
}
