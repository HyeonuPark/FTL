FTL
=============================

[<img alt="badge-github"    src="https://img.shields.io/badge/github.com-HyeonuPark/ftl-green">](https://github.com/HyeonuPark/ftl)
[<img alt="badge-crates.io" src="https://img.shields.io/crates/v/ftl.svg">](https://crates.io/crates/ftl)
[<img alt="badge-docs.rs"   src="https://docs.rs/ftl/badge.svg">](https://docs.rs/ftl)
[<img alt="badge-ci"        src="https://img.shields.io/github/workflow/status/HyeonuPark/ftl/CI/main">](https://github.com/HyeonuPark/ftl/actions?query=branch%3Amain)

DISCLAIMER: features below may not be implemented yet

FTL engine for the HTTP servers, with hyperdrive.

This crate aims to provide HTTP+JSON API with OpenAPI support, with minimal boilerplate.

# Examples

```rust
use ftl::{api, Schema, BaseError};
use serde::{Serialize, Deserialize};

#[derive(Schema, Serialize, Deserialize)]
pub struct FooRequest {
    /// Description about this field
    #[example(42)]
    number: i32,
    #[example("some text here")]
    text: Option<String>,
}

#[derive(Schema, Serialize, Deserialize)]
pub struct FooResponse {
    #[example(["foo", "bar"])]
    names: Vec<String>,
}

#[derive(ftl::Error, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "reason")]
pub enum Error {
    #[error("I'm not a lawyer but - {issue}")]
    #[status(UNAVAILABLE_FOR_LEGAL_REASONS)]
    Legal { issue: String },

    #[error("Because it's a teapot!")]
    #[status(IM_A_TEAPOT)]
    Teapot,

    #[error("Base error - {error}")]
    Other {
        #[status]
        error: BaseError,
    },
}

/// OpenAPI spec description.
#[api(alloc_cors)]
#[async_trait]
pub trait OpenApiSpecTitle {
    /// Description about this endpoint.
    ///
    /// When the request is arrived to the server,
    /// it's checked against the path constraint of each methods of this trait
    /// in the declaration order, from top to bottom.
    /// Only static segments and length of the path are considered.
    /// Parse fail triggers the `#[fallback]` method.
    #[post("/foo/{some_path_arg}?fromQuery={some_query_param}")]
    #[header(req_host = "Host")]
    async fn handle_foo(
        // It must be `Arc<Self>`
        self: Arc<Self>,
        /// Description about this parameter.
        /// Will be strippted by the `#[api]` macro
        /// since the doc comments are not allowed on the function parameter.
        ///
        /// Both path, query and header params are parsed
        /// with the FromStr trait.
        some_path_arg: IpAddr,
        /// Parameters are always required unless the type is Option.
        req_host: Option<String>,
        /// If the query param type is bool,
        /// the query value should be empty or not exist.
        some_query_param: bool,
        /// The request body. It must have the name `body`.
        /// If the type is String, the Content-Type header should be `text/plain`.
        /// Otherwise it's `application/json`, and parsed as JSON.
        /// Same applies to the Ok part of the response type.
        body: FooRequest,
        // Return type should be the `Result<T, E> where E: ftl::Error`.
        // If the T is String, the Content-Type header becomes `text/plain`.
        // Otherwise it must be `serde::Serialize + ftl::Spec`
        // and the Content-Type becomes `application/json`.
    ) -> Result<FooResponse, Error>;

    // Fallback method for the requests which failed to trigger
    // any other methods defined.
    // It takes `Arc<Self>` and the `ftl::BaseError`
    // and returns type which `impl ftl::Error`.
    //
    // If `#[fallback]` is missing the `ftl::BaseError` itself is used
    // as a result since it also `impl ftl::Error`.
    #[fallback]
    async fn handle_fallback(
        self: Arc<Self>,
        base: BaseError,
    ) -> Error;
}

struct App {...}

impl OpenApiSpecTitle for App {...}

#[tokio::main]
async fn main() {
    let app = Arc::new(App::new());
    let spec = serde_json::to_string(&app.openapi_spec()).unwrap();

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    app.server().run(addr).await.expect("HTTP server terminated with error");
}
```

# Non goals

- HTML or templating
- Static file serving
- Anything about the implementation, not API
- Formats other than the JSON or the plain text
- Nested routers
- Define endpoints from multiple files

# License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
