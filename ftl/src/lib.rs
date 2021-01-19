//! FTL

pub use http::{header, Request, Response, StatusCode};
pub use hyper::server::Server;

pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub mod error;
pub mod router;
pub mod schema;
pub mod service;

mod method;

pub use error::{BaseError, Error};
pub use router::Router;
pub use schema::Schema;
