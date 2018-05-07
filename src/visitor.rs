use futures::{Future, Stream};
use hyper::{client::HttpConnector, Client, Error as HyperError, Uri};
use hyper_tls::{Error as HyperTlsError, HttpsConnector};
use num_cpus;
use select::{document::Document, node::Node};
use tokio_core::reactor::Handle;

use std::collections::HashSet;

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
}
