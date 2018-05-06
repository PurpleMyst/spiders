use futures::{Future, Stream};
use hyper::{client::HttpConnector, Client, Error as HyperError, Uri};
use hyper_tls::{Error as HyperTlsError, HttpsConnector};
use num_cpus;
use select::{document::Document, node::Node};
use tokio_core::reactor::Handle;

use std::collections::HashSet;

fn relative_uri_to_absolute(base_uri: &Uri, uri: Uri) -> Option<Uri> {
    if uri.is_absolute() {
        Some(uri)
    } else {
        let new_url = if uri.path().starts_with("//") {
            format!("{}:{}", base_uri.scheme()?, uri)
        } else if uri == "#" {
            // TODO: implement a better check for this.
            // We could just return `base_uri.to_string()`, but we've obviously already visited that so we don't care.
            return None;
        } else {
            format!(
                "{}://{}{}",
                uri.scheme().or_else(|| base_uri.scheme())?,
                uri.authority().or_else(|| base_uri.host())?,
                uri.path(),
            )
        };

        new_url.parse::<Uri>().ok()
    }
}

#[derive(Debug)]
pub(crate) struct Visitor {
    handle: Handle,
    visited: HashSet<Uri>,
    http_client: Client<HttpsConnector<HttpConnector>>,
}

impl Visitor {
    pub(crate) fn new(handle: &Handle) -> Result<Self, HyperTlsError> {
        let cpus = num_cpus::get();

        Ok(Self {
            handle: handle.clone(),
            visited: HashSet::new(),
            http_client: Client::configure()
                .connector(HttpsConnector::new(cpus, handle)?)
                .build(handle),
        })
    }

    pub(crate) fn visit(
        &mut self,
        base_uri: Uri,
    ) -> Option<impl Future<Item = (Document, Vec<Uri>), Error = HyperError>> {
        if self.visited.contains(&base_uri) {
            return None;
        } else {
            self.visited.insert(base_uri.clone());
        }

        Some(
            self.http_client
                .get(base_uri.clone())
                .and_then(|response| response.body().collect())
                .map(move |chunks| {
                    // TODO: Use something smarter than reading the whole body into memory.
                    let body = chunks
                        .into_iter()
                        .map(|chunk| String::from_utf8_lossy(&chunk).into_owned())
                        .collect::<String>();

                    let document = Document::from(body.as_str());

                    let uris = document
                        .find(|node: &Node| node.html().starts_with("<a"))
                        .filter_map(|node| node.attr("href"))
                        .filter_map(|link| {
                            let uri = link.parse::<Uri>().ok()?;
                            relative_uri_to_absolute(&base_uri, uri)
                        })
                        .collect::<Vec<_>>();

                    (document, uris)
                }),
        )
    }
}
