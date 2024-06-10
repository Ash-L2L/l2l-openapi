//! Macros for deriving OpenAPI schemas from jsonrpsee RPC APIs.

pub use l2l_openapi_macros::open_api;

#[doc(hidden)]
pub use jsonrpsee as __jsonrpsee;

#[doc(hidden)]
pub use utoipa as __utoipa;
