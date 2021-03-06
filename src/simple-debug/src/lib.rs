#![feature(associated_consts)]

extern crate mainsail;
extern crate rigging;
extern crate tokio_service as tokio;

use std::fmt::Debug;

use mainsail::{ResourceEndpoint, Error};
use rigging::http;
use rigging::format::Format;
use rigging::method::Method;
use rigging::present::{Present, PresentResource, PresentCollection, Template};
use rigging::receive::Receive;

const MIME: &'static str = "ext/plain; charset=utf-8";

#[derive(Default, Clone)]
pub struct SimpleDebug {
    _private: (),
}

impl<T> Format<T> for SimpleDebug
where
    T: ResourceEndpoint + Debug,
{
    type Presenter = Self;
    type Receiver = Self;
    const MIME_TYPES: &'static [&'static str] = &[MIME];
}

impl<T: ResourceEndpoint> Receive<T> for SimpleDebug { }

impl<T: ResourceEndpoint + Debug> Present<T> for SimpleDebug {
    type ResourcePresenter = ResourcePresenter;
    type CollectionPresenter = CollectionPresenter<T>;

    fn for_resource(&self) -> Self::ResourcePresenter {
        ResourcePresenter::default()
    }

    fn for_collection(&self) -> Self::CollectionPresenter {
        CollectionPresenter::default()
    }
}

#[derive(Default, Clone, Copy)]
pub struct ResourcePresenter;

impl<T: ResourceEndpoint + Debug> PresentResource<T> for ResourcePresenter {
    fn resource<M: ?Sized + Method<Response = T>>(self, resource: T, _: Option<Template>)
        -> http::Response
    {
        debug_response(resource, http::StatusCode::Ok)
    }

    fn error<M: ?Sized + Method<Response = T>>(self, error: Error, _: Option<Template>)
        -> http::Response
    {
        debug_response(error, http::StatusCode::InternalServerError)
    }
}

pub struct CollectionPresenter<T> {
    resources: Result<Vec<T>, Error>,
}

impl<T> Default for CollectionPresenter<T> {
    fn default() -> Self {
        Self { resources: Ok(vec![]) }
    }
}

impl<T> Clone for CollectionPresenter<T> {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl<T: ResourceEndpoint + Debug> PresentCollection<T> for CollectionPresenter<T> {
    fn append(&mut self, resource: T, _: Option<Template>) {
        let _ = self.resources.as_mut().map(|v| v.push(resource));
    }

    fn error(&mut self, error: Error, _: Option<Template>) {
        self.resources = Err(error);
    }

    fn finish(self) -> http::Response {
        match self.resources {
            Ok(resources)   => debug_response(resources, http::StatusCode::Ok),
            Err(error)      => debug_response(error, http::StatusCode::InternalServerError),
        }
    }
}

fn debug_response<T: Debug>(t: T, status: http::StatusCode) -> http::Response {
    let data = format!("{:?}", t);
    http::Response::new().with_status(status)
                         .with_header(http::headers::ContentLength(data.len() as u64))
                         .with_header(http::headers::ContentType(MIME.parse().unwrap()))
                         .with_body(data)
}
