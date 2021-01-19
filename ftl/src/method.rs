use std::convert::TryFrom;

use http::Method;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    strum::AsRefStr,
    strum::Display,
    strum::EnumIter,
)]
#[strum(serialize_all = "UPPERCASE")]
pub enum SupportedMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

#[test]
fn supported_method_strum_impl() {
    use SupportedMethod::*;

    let fixtures = [
        (Get, "GET"),
        (Post, "POST"),
        (Put, "PUT"),
        (Delete, "DELETE"),
        (Head, "HEAD"),
        (Options, "OPTIONS"),
        (Patch, "PATCH"),
    ];

    for &(method, name) in &fixtures {
        assert_eq!(method.as_ref(), name);
        assert_eq!(format!("{}", method), name);
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("HTTP method {0} is not supported by this server")]
pub struct UnsupportedMethod(pub Method);

impl SupportedMethod {
    pub const ALLOW_HEADER: &'static str = "GET, POST, PUT, DELETE, HEAD, OPTIONS, PATCH";

    pub fn new(method: Method) -> Result<Self, UnsupportedMethod> {
        Ok(match method {
            Method::GET => Self::Get,
            Method::POST => Self::Post,
            Method::PUT => Self::Put,
            Method::DELETE => Self::Delete,
            Method::HEAD => Self::Head,
            Method::OPTIONS => Self::Options,
            Method::PATCH => Self::Patch,
            other => return Err(UnsupportedMethod(other)),
        })
    }

    pub fn request_has_body(self) -> bool {
        matches!(self, Self::Get | Self::Delete | Self::Head | Self::Options)
    }

    pub fn response_has_body(self) -> bool {
        matches!(self, Self::Head)
    }
}

impl TryFrom<Method> for SupportedMethod {
    type Error = UnsupportedMethod;

    fn try_from(method: Method) -> Result<Self, Self::Error> {
        Self::new(method)
    }
}
