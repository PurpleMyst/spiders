use std::net::SocketAddr;
use std::rc::Rc;

use failure::Error;
use futures::{future, Async, Future, IntoFuture, Stream};
use hyper::{client::HttpConnector, Client, Uri};
use hyper_tls::HttpsConnector;
use num_cpus;
use redis_async::client::paired::{paired_connect as redis_paired_connect,
                                  PairedConnection as RedisPairedConnection};
use select::{document::Document, node::Node};
use tokio_core::reactor::Handle;

const URLS_KEY: &str = "crawler:urls";

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

/// The main crawler struct.
///
/// Usage of this must be done as a [`::futures::Stream`], and this will not stop on its own unless
/// it exhausts its list of URLs to visit (which is unlikely). It is reccommended then that you use
/// [`::futures::Stream::take`] to not overload the server you're crawling.
pub struct Crawler {
    handle: Handle,
    http_client: Rc<Client<HttpsConnector<HttpConnector>>>,
    redis_client: RedisPairedConnection,

    host: Option<String>,

    maybe_future: Option<Box<Future<Item = (Document, Vec<Uri>), Error = Error>>>,
    maybe_visiting: Option<Uri>,
}

impl Crawler {
    /// Creates a new `Crawler` instance.
    pub fn new(handle: Handle) -> impl Future<Item = Self, Error = Error> {
        let addr = SocketAddr::new("127.0.0.1".parse().unwrap(), 6379);

        redis_paired_connect(&addr)
            .map_err(Error::from)
            .join(
                HttpsConnector::new(num_cpus::get(), &handle.clone())
                    .map_err(Error::from)
                    .into_future(),
            )
            .map(move |(redis_client, https_connector)| Self {
                handle: handle.clone(),

                http_client: Rc::new(
                    Client::configure()
                        .connector(https_connector)
                        .build(&handle),
                ),

                redis_client,

                host: None,

                maybe_future: None,
                maybe_visiting: None,
            })
    }

    #[allow(missing_docs)]
    pub fn queue(&mut self, uri: Uri) -> impl Future<Item = usize, Error = Error> {
        self.redis_client
            .send::<usize>(resp_array!["SADD", URLS_KEY, uri.to_string()])
            .map_err(Error::from)
    }

    /// Limit the URLs this crawler crawls to the one which match the host given. This returns the
    /// previous host value, if there was one.
    pub fn limit_host(&mut self, host: String) -> Option<String> {
        let old_host = self.host.take();
        self.host = Some(host);
        old_host
    }

    fn add_to_visited(&mut self, uri: &Uri) -> impl Future<Item = bool, Error = Error> {
        self.redis_client
            .send::<usize>(resp_array!["SADD", URLS_KEY, uri.to_string()])
            .map(|b| match b {
                0 => false,
                1 => true,
                _ => unreachable!(),
            })
            .map_err(Error::from)
    }

    pub fn fetch(
        &mut self,
        base_uri: Uri,
    ) -> impl Future<Item = Option<(Document, Vec<Uri>)>, Error = Error> {
        let http_client = Rc::clone(&self.http_client);

        self.add_to_visited(&base_uri)
            .map_err(Error::from)
            .and_then(
            move |already_visited| -> Box<Future<Item = Option<(Document, Vec<Uri>)>, Error = Error>> {
                if already_visited {
                    Box::new(future::ok(None))
                } else {
                    Box::new(
                        http_client
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

                                Some((document, uris))
                            })
                            .map_err(Error::from),
                    )
                }
            },
        )
    }
}

impl Stream for Crawler {
    type Item = (Uri, Document);
    type Error = Error;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        unimplemented!("Crawler::poll");
    }
}
