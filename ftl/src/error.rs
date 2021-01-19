use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use hyper::StatusCode;
use indexmap::IndexMap;
use openapiv3::{self as oa, Schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;

use crate::method::SupportedMethod;
use crate::schema::Schema as FtlSchema;
use crate::BoxError;

pub trait Error: FtlSchema {
    fn status(&self) -> StatusCode;

    fn error_schema() -> ErrorSchema;
}

#[derive(Debug)]
pub struct ErrorSchema {
    pub default_schema: Option<Schema>,
    pub schemas: HashMap<StatusCode, Schema>,
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
#[non_exhaustive]
pub enum BaseError {
    #[error("404 Not Found")]
    NotFound,
    #[error("405 Method Not Allowed")]
    MethodNotAllowed { allowed: Vec<SupportedMethod> },
    #[error("408 Request Timeout")]
    RequestTimeout,
    #[error("411 Length Required")]
    LengthRequired,
    #[error("413 Payload Too Lager")]
    PayloadTooLarge,
    #[error("415 Unsupported Media Type")]
    UnsupportedMediaType,
    #[error("Failed to decode request body as UTF-8")]
    BodyNotUtf8,
    #[error("Failed to parse request parameters")]
    InvalidParameter {
        query: Vec<InvalidParameter>,
        header: Vec<InvalidParameter>,
    },
    #[error("Other error - {0}")]
    Other(#[from] DynError),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InvalidParameter {
    pub name: Cow<'static, str>,
    pub value: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub struct DynError {
    pub status: StatusCode,
    #[source]
    pub error: Option<BoxError>,
}

#[derive(Serialize, Deserialize)]
struct DynErrorSerde {
    status: u16,
    error: Option<String>,
}

impl FtlSchema for BaseError {
    fn schema() -> openapiv3::Schema {
        todo!()
    }
}

impl Error for BaseError {
    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::MethodNotAllowed { .. } => StatusCode::METHOD_NOT_ALLOWED,
            Self::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
            Self::LengthRequired => StatusCode::LENGTH_REQUIRED,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::BodyNotUtf8 => StatusCode::BAD_REQUEST,
            Self::InvalidParameter { .. } => StatusCode::BAD_REQUEST,
            Self::Other(DynError { status, .. }) => *status,
        }
    }

    fn error_schema() -> ErrorSchema {
        todo!()
    }
}

impl Serialize for DynError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let repr = DynErrorSerde {
            status: self.status.into(),
            error: self.error.as_ref().map(|err| err.to_string()),
        };

        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DynError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let repr = DynErrorSerde::deserialize(deserializer)?;

        Ok(DynError {
            status: StatusCode::from_u16(repr.status).map_err(D::Error::custom)?,
            error: repr.error.map(From::from),
        })
    }
}

impl fmt::Display for DynError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.status)?;

        if let Some(err) = &self.error {
            write!(f, " - {}", err)?;
        }

        Ok(())
    }
}

impl Error for String {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_schema() -> ErrorSchema {
        ErrorSchema {
            default_schema: None,
            schemas: Some((StatusCode::INTERNAL_SERVER_ERROR, String::schema()))
                .into_iter()
                .collect(),
        }
    }
}

#[test]
fn parse_example_dyn_error() {
    crate::schema::parse_example::<DynError>()
}

impl FtlSchema for DynError {
    fn schema() -> oa::Schema {
        Schema {
            schema_data: oa::SchemaData {
                example: Some(json!({"status": 418, "error": "Honestly, it's a teapot"})),
                title: Some("DynError".into()),
                description: Some("Type erased error".into()),
                ..Default::default()
            },
            schema_kind: oa::SchemaKind::Type(oa::Type::Object(oa::ObjectType {
                properties: {
                    let mut map = IndexMap::new();
                    map.insert(
                        "status".into(),
                        oa::ReferenceOr::Item(Box::new(oa::Schema {
                            schema_data: oa::SchemaData {
                                title: Some("HTTP status code".into()),
                                ..Default::default()
                            },
                            schema_kind: oa::SchemaKind::Type(oa::Type::Integer(oa::IntegerType {
                                format: oa::VariantOrUnknownOrEmpty::Item(oa::IntegerFormat::Int32),
                                minimum: Some(100),
                                maximum: Some(999),
                                ..Default::default()
                            })),
                        })),
                    );
                    map.insert(
                        "error".into(),
                        oa::ReferenceOr::Item(Box::new(<Option<String>>::schema())),
                    );
                    map
                },
                required: vec!["status".into()],
                ..Default::default()
            })),
        }
    }
}

impl Error for DynError {
    fn status(&self) -> StatusCode {
        self.status
    }

    fn error_schema() -> ErrorSchema {
        ErrorSchema {
            default_schema: Some(Self::schema()),
            schemas: HashMap::new(),
        }
    }
}
