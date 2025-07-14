use std::net::SocketAddr;

use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use l2l_openapi::open_api;
use serde::Serialize;
use utoipa::{
    openapi::{self, RefOr, Schema},
    PartialSchema, ToSchema,
};

#[derive(Clone, Serialize, ToSchema)]
pub struct Inner0 {
    pub inner0_bool: bool,
    pub inner0_u64: u64,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Inner1 {
    pub inner1_bool: bool,
    pub inner1_u32: u32,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct InnerRefs {
    pub inner0: Inner0,
    pub inner1: Inner1,
}

struct SocketAddrSchema;

impl PartialSchema for SocketAddrSchema {
    fn schema() -> RefOr<Schema> {
        let obj = utoipa::openapi::Object::with_type(openapi::Type::String);
        RefOr::T(Schema::Object(obj))
    }
}

#[open_api(ref_schemas [Inner0, Inner1])]
#[rpc(server)]
pub trait TestRpc {
    /// Doc comment
    #[method(name = "test_rpc0")]
    async fn test_rpc0(&self, some_u64: u64) -> RpcResult<u64>;

    /** Block doc comment */
    #[method(name = "test_rpc1")]
    async fn test_rpc1(&self, some_u32: u32) -> RpcResult<u32>;

    #[method(name = "no_doc_comment")]
    async fn no_doc_comment(&self, some_u32: u32) -> RpcResult<u32>;

    /// Different method name
    #[method(name = "jsonrpsee_method_name")]
    async fn rust_method_name(&self, some_u32: u32) -> RpcResult<u32>;

    /// No params
    #[method(name = "no_params")]
    async fn no_params(&self) -> RpcResult<u32>;

    /// Multiple params
    #[method(name = "multiple_params")]
    async fn multiple_params(&self, some_u64: u64, some_u32: u32) -> RpcResult<u32>;

    /// No response
    #[method(name = "no_respose")]
    async fn no_response(&self, some_u32: u32);

    /// Result unit (requires ToSchema)
    #[open_api_method(output_schema(ToSchema))]
    #[method(name = "result_unit")]
    async fn result_unit(&self, some_u32: u32) -> RpcResult<()>;

    /// Result socketaddr (requires output_schema)
    #[open_api_method(output_schema(PartialSchema = "SocketAddrSchema"))]
    #[method(name = "result_socketaddr")]
    async fn result_socketaddr(&self, some_u32: u32) -> RpcResult<SocketAddr>;

    /// Socketaddr param (requires arg_schema)
    #[method(name = "socketaddr_param")]
    async fn socketaddr_param(
        &self,
        #[open_api_method_arg(schema(PartialSchema = "SocketAddrSchema"))] socket_addr: SocketAddr,
    ) -> RpcResult<u32>;

    /// Result has inner refs
    #[open_api_method(output_schema(ToSchema))]
    #[method(name = "result_inner_ref")]
    async fn result_inner_ref(&self, some_u32: u32) -> RpcResult<InnerRefs>;

    /// Subscription
    #[subscription(name = "subscribe", item = ())]
    async fn subscribe(&self) -> jsonrpsee::core::SubscriptionResult;
}

#[test]
fn test_print_openapi() -> anyhow::Result<()> {
    use utoipa::OpenApi;
    let api_str = TestRpcDoc::openapi().to_pretty_json().unwrap();
    println!("{api_str}");
    Ok(())
}
