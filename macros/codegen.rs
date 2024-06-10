use proc_macro2::TokenStream;
use proc_macro_roids::IdentExt;
use quote::quote;
use syn::Ident;

use crate::{
    analyze::{MethodOutput, MethodParam, SchemaSource},
    lower::Ir,
};

pub type Rust = TokenStream;

/// Expression to generate a schema from a source and type
fn schema_expr(source: &SchemaSource, ty: &syn::Type) -> TokenStream {
    match source {
        SchemaSource::Partial(Some(ty)) => quote! {
            <#ty as l2l_openapi::__utoipa::PartialSchema>::schema()
        },
        SchemaSource::Partial(None) => quote! {
            <#ty as l2l_openapi::__utoipa::PartialSchema>::schema()
        },
        SchemaSource::ToSchema(Some(ty)) => quote! {
            <#ty as l2l_openapi::__utoipa::ToSchema>::schema().1
        },
        SchemaSource::ToSchema(None) => quote! {
            <#ty as l2l_openapi::__utoipa::ToSchema>::schema().1
        },
    }
}

/// Expression to generate a schema from a method param
fn method_param_schema_expr(method_param: &MethodParam) -> TokenStream {
    schema_expr(&method_param.schema_source, &method_param.ty)
}

/// Expression to generate a schema from a method output
fn method_output_schema_expr(method_output: &MethodOutput) -> TokenStream {
    let MethodOutput { ty, schema_source } = method_output;
    let inner_ty_expr = quote! { <#ty as l2l_openapi::__jsonrpsee::IntoResponse>::Output };
    let inner_ty: syn::Type = syn::parse2(inner_ty_expr).unwrap();
    schema_expr(schema_source, &inner_ty)
}

fn gen_doc(ir: &Ir) -> Rust {
    let Ir {
        ref_schema_tys,
        methods,
        item_trait,
    } = ir;

    let add_paths: TokenStream = methods.iter().map(|method| {
        // TODO: Do not use rust method name
        let ident_str_lit = &method.ident.to_string();

        let operation = {
            let set_description = method.description.as_ref().map(|description| quote! {
                operation.description = Some(#description.to_owned());
            });
            let set_request_body = if !method.params.is_empty() {
                // TODO: set name
                let content_schema = if method.params.len() == 1 {
                    method_param_schema_expr(&method.params[0])
                } else {
                    let set_properties: TokenStream =
                        method.params.iter().map(|method_param| {
                            let ident_str_lit = method_param.ident.to_string();
                            let schema_expr = method_param_schema_expr(method_param);
                            quote! {
                                schema.properties.insert(
                                    #ident_str_lit.to_owned(),
                                    #schema_expr
                                );
                            }
                        }).collect();
                    quote! {
                        {
                            let mut schema = utoipa::openapi::Object::new();
                            #set_properties
                            l2l_openapi::__utoipa::openapi::Schema::Object(schema)
                        }
                    }
                };
                Some(quote! {
                    operation.request_body = {
                        let mut request_body =
                        l2l_openapi::__utoipa::openapi::request_body::RequestBody::new();
                        let content_schema = #content_schema;
                        let content = l2l_openapi::__utoipa::openapi::ContentBuilder::new()
                            .schema(content_schema)
                            .build();
                        request_body.content.insert("application/json".to_owned(), content);
                        Some(request_body)
                    };
                })
            } else {
                None
            };
            let set_responses =
                // TODO: Handle errors
                method.output.as_ref().map(|output| {
                    let schema_expr = method_output_schema_expr(output);
                    quote! {
                        let response = {
                            let content = l2l_openapi::__utoipa::openapi::ContentBuilder::new()
                                .schema(#schema_expr)
                                .build();
                            l2l_openapi::__utoipa::openapi::ResponseBuilder::new()
                                .content("application/json".to_owned(), content)
                                .build()
                        };
                        operation.responses.responses.insert(
                            "200".to_owned(),
                            l2l_openapi::__utoipa::openapi::RefOr::T(response)
                        );
                    }
                });
            quote! {
                {
                    let mut operation = l2l_openapi::__utoipa::openapi::path::Operation::new();
                    #set_description
                    operation.operation_id = Some(#ident_str_lit.to_owned());
                    #set_request_body
                    #set_responses
                    operation
                }
            }
        };

        let path_item = quote! {
            {
                let mut path_item = utoipa::openapi::PathItem::default();
                let operation = #operation;
                path_item.operations.insert(l2l_openapi::__utoipa::openapi::PathItemType::Post, operation);
                path_item
            }
        };

        quote! { .path(#ident_str_lit, #path_item) }
    }).collect();

    let add_ref_schemas: TokenStream = ref_schema_tys
        .iter()
        .map(|ref_schema_ty| {
            quote! { .schema_from::<#ref_schema_ty>() }
        })
        .collect();

    // ident of the generated struct
    let ident = &item_trait.ident;
    let struct_ident_suffix = Ident::new("Doc", ident.span());
    let struct_ident = ident.append(struct_ident_suffix);
    let struct_vis = &item_trait.vis;

    quote! {
        #struct_vis struct #struct_ident;

        impl utoipa::OpenApi for #struct_ident {
            fn openapi() -> l2l_openapi::__utoipa::openapi::OpenApi {
                let paths = l2l_openapi::__utoipa::openapi::PathsBuilder::new()
                    #add_paths
                    .build();
                let components = l2l_openapi::__utoipa::openapi::ComponentsBuilder::new()
                    #add_ref_schemas
                    .build();
                l2l_openapi::__utoipa::openapi::OpenApiBuilder::new()
                .paths(paths)
                .components(Some(components))
                .build()
            }
        }
    }
}

pub fn codegen(ir: Ir) -> Rust {
    let Ir {
        ref_schema_tys: _,
        methods: _,
        ref item_trait,
    } = ir;
    let doc_item = gen_doc(&ir);
    quote!(
        #item_trait

        #doc_item
    )
}

#[cfg(test)]
mod tests {}
