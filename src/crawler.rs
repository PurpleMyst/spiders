use super::visitor::Visitor;

use futures::{Async, Future, Stream};
use hyper::{Error as HyperError, Uri};
use hyper_tls::Error as HyperTlsError;
use select::document::Document;
use tokio_core::reactor::Handle;

pub struct Crawler {
    visitor: Visitor,
    to_visit: Vec<Uri>,

    maybe_future: Option<Box<Future<Item = (Document, Vec<Uri>), Error = HyperError>>>,
    maybe_visiting: Option<Uri>,
}

impl Crawler {
    pub fn new(handle: &Handle, start_uri: Uri) -> Result<Self, HyperTlsError> {
        Ok(Self {
            visitor: Visitor::new(handle)?,
            to_visit: vec![start_uri],

            maybe_future: None,
            maybe_visiting: None,
        })
    }
}

impl Stream for Crawler {
    // TODO: Use `failure` to avoid `unwrap` and `expect`.

    type Item = (Uri, Document);
    type Error = HyperError;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        if self.maybe_future.is_none() {
            loop {
                let uri = match self.to_visit.pop() {
                    Some(uri) => uri,
                    None => return Ok(Async::Ready(None)),
                };

                self.maybe_visiting = Some(uri.clone());
                self.maybe_future = if let Some(future) = self.visitor.visit(uri) {
                    Some(Box::new(future))
                } else {
                    continue;
                };

                break;
            }
        }

        // We're sure `self.maybe_future` is Some, so we can unwrap it.
        let (document, new_urls) = {
            let future = self.maybe_future.as_mut().unwrap();
            try_ready!(future.poll())
        };

        self.maybe_future = None;
        self.to_visit.extend(new_urls);

        // We're also sure `self.maybe_visiting` is Some.
        Ok(Async::Ready(Some((
            self.maybe_visiting.take().unwrap(),
            document,
        ))))
    }
}
