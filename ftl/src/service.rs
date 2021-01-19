use std::convert::Infallible;
use std::convert::TryInto;
use std::io::Cursor;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::future::{ready, BoxFuture, Ready};
use http::header::{self, HeaderMap};
use http::request::{self, Request};
use http::{Response, StatusCode};
use hyper::body::{Body, Bytes};
use hyper::service::Service as HyperService;
use strum::IntoEnumIterator;

use crate::error::{BaseError, DynError};
use crate::method::SupportedMethod;
use crate::router::Router;
use crate::BoxError;

#[derive(Debug)]
pub struct Service<T, H>
where
    T: Send + Sync + 'static + ?Sized,
    H: for<'a> Fn(
            Arc<T>,
            Request<Result<&'a str, Box<BaseError>>>,
        ) -> BoxFuture<'a, Result<Response<String>, BoxError>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    router: Router<T, H>,
    config: Arc<Config>,
}

#[derive(Debug, Default)]
pub struct Builder {
    config: Config,
}

#[derive(Debug, Default)]
struct Config {
    max_request_length: Option<usize>,
    #[cfg(feature = "tokio-runtime")]
    request_read_timeout: Option<Duration>,
}

#[derive(Debug, Clone, Default)]
pub struct OutBuffer {
    inner: Option<String>,
}

impl<T, H> Service<T, H>
where
    T: Send + Sync + 'static + ?Sized,
    H: for<'a> Fn(
            Arc<T>,
            Request<Result<&'a str, Box<BaseError>>>,
        ) -> BoxFuture<'a, Result<Response<String>, BoxError>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub fn new(router: Router<T, H>) -> Self {
        Self::builder().build(router)
    }

    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn app(&self) -> Arc<T> {
        Arc::clone(&self.router.app)
    }
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn max_reqeust_length(mut self, length: usize) -> Self {
        self.config.max_request_length = Some(length);
        self
    }

    #[cfg(feature = "tokio-runtime")]
    pub fn request_read_timeout(mut self, timeout: Duration) -> Self {
        self.config.request_read_timeout = Some(timeout);
        self
    }

    pub fn build<T, H>(self, router: Router<T, H>) -> Service<T, H>
    where
        T: Send + Sync + 'static + ?Sized,
        H: for<'a> Fn(
                Arc<T>,
                Request<Result<&'a str, Box<BaseError>>>,
            ) -> BoxFuture<'a, Result<Response<String>, BoxError>>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        Service {
            router,
            config: Arc::new(self.config),
        }
    }
}

impl<'c, C, T, H> HyperService<&'c C> for Service<T, H>
where
    T: Send + Sync + 'static + ?Sized,
    H: for<'a> Fn(
            Arc<T>,
            Request<Result<&'a str, Box<BaseError>>>,
        ) -> BoxFuture<'a, Result<Response<String>, BoxError>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    type Response = Self;
    type Error = Infallible;
    type Future = Ready<Result<Self, Infallible>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: &'c C) -> Self::Future {
        ready(Ok(self.clone()))
    }
}

impl<T, H> HyperService<Request<Body>> for Service<T, H>
where
    T: Send + Sync + 'static + ?Sized,
    H: for<'a> Fn(
            Arc<T>,
            Request<Result<&'a str, Box<BaseError>>>,
        ) -> BoxFuture<'a, Result<Response<String>, BoxError>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    type Response = Response<OutBuffer>;
    type Error = BoxError;
    // TODO: apply existential type when available
    type Future = BoxFuture<'static, Result<Response<OutBuffer>, BoxError>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let router = self.router.clone();
        let config = Arc::clone(&self.config);

        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let mut buf = Bytes::new();
            let body = parse_request(&parts, body, Arc::clone(&config), &mut buf).await;
            let resp = (router.handler)(router.app, Request::from_parts(parts, body)).await?;
            Ok(resp.map(From::from))
        })
    }
}

// Two lifetimes to workaround clippy bug: https://github.com/rust-lang/rust-clippy/issues/5787
async fn parse_request<'a, 'b>(
    parts: &'a request::Parts,
    body: Body,
    conf: Arc<Config>,
    buf: &'b mut Bytes,
) -> Result<&'b str, Box<BaseError>> {
    let method: SupportedMethod =
        parts
            .method
            .clone()
            .try_into()
            .map_err(|_| BaseError::MethodNotAllowed {
                allowed: SupportedMethod::iter().collect(),
            })?;

    if !method.request_has_body() {
        return Ok("");
    }

    // variable to satisfy clippy
    let content_length_header = header::CONTENT_LENGTH;
    let content_length: usize = parts
        .headers
        .get(&content_length_header)
        .ok_or(BaseError::LengthRequired)?
        .to_str()
        .map_err(|_| BaseError::LengthRequired)?
        .parse()
        .map_err(|_| BaseError::LengthRequired)?;

    if let Some(max_length) = conf.max_request_length {
        if content_length > max_length {
            return Err(BaseError::PayloadTooLarge.into());
        }
    }

    #[cfg(feature = "tokio-runtime")]
    let buffer = if let Some(timeout) = conf.request_read_timeout {
        tokio::time::timeout(timeout, hyper::body::to_bytes(body))
            .await
            .map_err(|_| BaseError::RequestTimeout)?
    } else {
        hyper::body::to_bytes(body).await
    };

    #[cfg(not(feature = "tokio-runtime"))]
    let buffer = hyper::body::to_bytes(body).await;

    *buf = buffer.map_err(|err| {
        BaseError::Other(DynError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: Some(err.into()),
        })
    })?;

    let body = std::str::from_utf8(&**buf).map_err(|_| BaseError::BodyNotUtf8)?;

    Ok(body)
}

impl<T, H> Clone for Service<T, H>
where
    T: Send + Sync + 'static + ?Sized,
    H: for<'a> Fn(
            Arc<T>,
            Request<Result<&'a str, Box<BaseError>>>,
        ) -> BoxFuture<'a, Result<Response<String>, BoxError>>
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn clone(&self) -> Self {
        Service {
            router: self.router.clone(),
            config: Arc::clone(&self.config),
        }
    }
}

impl OutBuffer {
    pub fn empty() -> Self {
        String::new().into()
    }
}

impl From<String> for OutBuffer {
    fn from(s: String) -> Self {
        Self { inner: Some(s) }
    }
}

impl hyper::body::HttpBody for OutBuffer {
    type Data = Cursor<Vec<u8>>;
    type Error = Infallible;

    fn poll_data(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        Poll::Ready(self.inner.take().map(|v| Ok(Cursor::new(v.into_bytes()))))
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_none()
    }
}
