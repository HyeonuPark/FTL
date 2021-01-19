//! Routes reqeust to the matching handler.
//!
//! The router is a central type of the FTL workflow.
//! It can be generated from the api trait.
//! You can apply some tests on it, or create another router with some combinators.
//! At the end the [`Service`](crate::service::Service) can be generated from the router.

use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::future::BoxFuture;
use hyper::{Request, Response, Server};

use crate::error::BaseError;
use crate::service::Service;
use crate::BoxError;

pub type Handler<T> = for<'a> fn(
    Arc<T>,
    Request<Result<&'a str, Box<BaseError>>>,
) -> BoxFuture<'a, Result<Response<String>, BoxError>>;

pub struct Router<T, H = Handler<T>>
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
    pub app: Arc<T>,
    pub handler: H,
}

impl<T, H> Router<T, H>
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
    pub fn with<F, H2>(self, middleware: F) -> Router<T, H2>
    where
        F: FnOnce(H) -> H2,
        H2: for<'a> Fn(
                Arc<T>,
                Request<Result<&'a str, Box<BaseError>>>,
            ) -> BoxFuture<'a, Result<Response<String>, BoxError>>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        Router {
            app: self.app,
            handler: middleware(self.handler),
        }
    }

    pub fn call<'a>(
        &self,
        request: Request<Result<&'a str, Box<BaseError>>>,
    ) -> BoxFuture<'a, Result<Response<String>, BoxError>> {
        let app = Arc::clone(&self.app);

        (self.handler)(app, request)
    }

    pub async fn run(self, addr: SocketAddr) -> Result<(), BoxError> {
        let service = Service::new(self);
        Server::try_bind(&addr)?.serve(service).await?;
        Ok(())
    }
}

impl<T, H> Clone for Router<T, H>
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
        Self {
            app: Arc::clone(&self.app),
            handler: self.handler.clone(),
        }
    }
}

impl<T: fmt::Debug, H> fmt::Debug for Router<T, H>
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Router")
            .field("app", &self.app)
            .field("handler", &"fn { ... }")
            .finish()
    }
}
