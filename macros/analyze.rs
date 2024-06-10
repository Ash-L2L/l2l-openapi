use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{spanned::Spanned, Attribute, Ident, ItemTrait, TraitItem, TraitItemFn};

use crate::parse::Ast;

#[derive(Clone, Debug)]
pub enum SchemaSource {
    /// Use [`utoipa::PartialSchema`]
    Partial(Option<syn::Type>),
    /// Use [`utoipa::ToSchema`]
    ToSchema(Option<syn::Type>),
}

impl Default for SchemaSource {
    fn default() -> Self {
        Self::Partial(None)
    }
}

/// Custom attribute for method params
#[derive(Debug, Default)]
pub struct MethodParamAttr {
    pub schema_source: Option<SchemaSource>,
    pub span: Option<Span>,
}

/// Error when parsing the custom method parameter attribute
#[derive(Debug)]
pub struct MethodParamAttrParseError(pub syn::Error);

impl MethodParamAttrParseError {
    fn into_compile_error(self) -> TokenStream {
        self.0.into_compile_error()
    }
}

fn parse_method_param_attr(
    attr: &Attribute,
) -> Option<Result<MethodParamAttr, MethodParamAttrParseError>> {
    if !attr.path().is_ident("open_api_method_arg") {
        return None;
    }
    let mut res = MethodParamAttr {
        schema_source: None,
        span: Some(attr.span()),
    };
    let parse_result =
        attr.parse_nested_meta(
            |meta| match meta.path.require_ident()?.to_string().as_str() {
                "schema" => {
                    if res.schema_source.is_some() {
                        let err_msg = "schema cannot be set more than once";
                        return Err(meta.error(err_msg));
                    }
                    meta.parse_nested_meta(|meta| {
                        match meta.path.require_ident()?.to_string().as_str() {
                            "PartialSchema" => {
                                let ty = if meta.input.is_empty() {
                                    None
                                } else {
                                    Some(meta.value()?.parse::<syn::LitStr>()?.parse()?)
                                };
                                res.schema_source = Some(SchemaSource::Partial(ty));
                                Ok(())
                            }
                            "ToSchema" => {
                                let ty = if meta.input.is_empty() {
                                    None
                                } else {
                                    Some(meta.value()?.parse::<syn::LitStr>()?.parse()?)
                                };
                                res.schema_source = Some(SchemaSource::ToSchema(ty));
                                Ok(())
                            }
                            ident => {
                                let err_msg = format!("unexpected value: {ident}");
                                Err(syn::Error::new(meta.path.span(), err_msg))
                            }
                        }
                    })
                }
                ident => {
                    let err_msg = format!("unexpected key: {ident}");
                    Err(syn::Error::new(meta.path.span(), err_msg))
                }
            },
        );
    match parse_result {
        Ok(()) => Some(Ok(res)),
        Err(syn_err) => Some(Err(MethodParamAttrParseError(syn_err))),
    }
}

pub struct MethodParam {
    pub ident: Ident,
    pub ty: Box<syn::Type>,
    pub schema_source: SchemaSource,
}

#[derive(Debug)]
pub enum ParamError {
    AttrParseError(MethodParamAttrParseError),
    DuplicateAttr(Span),
    ExpectedPatIdent(Box<syn::Pat>),
}

impl ParamError {
    fn into_compile_error(self) -> TokenStream {
        match self {
            Self::AttrParseError(err) => err.into_compile_error(),
            Self::DuplicateAttr(span) => {
                let err_msg = "open_api_method_arg attribute can be used at most once";
                syn::Error::new(span, err_msg).into_compile_error()
            }
            Self::ExpectedPatIdent(pat) => {
                let span = pat.span();
                let err_msg = format!("Expected identifier; found {}", pat.to_token_stream());
                syn::Error::new(span, err_msg).into_compile_error()
            }
        }
    }
}

pub struct ParamErrors(pub Vec<ParamError>);

fn analyze_param_attrs(errs: &mut Vec<ParamError>, pat_type: &mut syn::PatType) -> MethodParamAttr {
    let mut res_attr = None;
    pat_type.attrs.retain(|attr| {
        let Some(parse_res) = parse_method_param_attr(attr) else {
            return true;
        };
        match parse_res {
            Ok(attr) => {
                if res_attr.is_some() {
                    errs.push(ParamError::DuplicateAttr(attr.span.unwrap()))
                } else {
                    res_attr = Some(attr);
                }
            }
            Err(attr_parse_err) => errs.push(ParamError::AttrParseError(attr_parse_err)),
        };
        false
    });
    res_attr.unwrap_or_default()
}

fn analyze_param(pat_type: &mut syn::PatType) -> Result<MethodParam, ParamErrors> {
    let mut errs = Vec::new();
    let method_param_attr = analyze_param_attrs(&mut errs, pat_type);
    let syn::Pat::Ident(ident) = &*pat_type.pat else {
        let err = ParamError::ExpectedPatIdent(pat_type.pat.clone());
        errs.push(err);
        return Err(ParamErrors(errs));
    };
    if errs.is_empty() {
        Ok(MethodParam {
            ident: ident.ident.clone(),
            ty: pat_type.ty.clone(),
            schema_source: method_param_attr.schema_source.unwrap_or_default(),
        })
    } else {
        Err(ParamErrors(errs))
    }
}

/// Custom attribute for methods
#[derive(Debug, Default)]
pub struct MethodAttr {
    pub schema_source: Option<SchemaSource>,
    pub span: Option<Span>,
}

/// Error when parsing the custom method attribute
pub struct MethodAttrParseError(pub syn::Error);

impl MethodAttrParseError {
    fn into_compile_error(self) -> TokenStream {
        self.0.into_compile_error()
    }
}

fn parse_method_attr(attr: &Attribute) -> Option<Result<MethodAttr, MethodAttrParseError>> {
    if !attr.path().is_ident("open_api_method") {
        return None;
    }
    let mut res = MethodAttr {
        schema_source: None,
        span: Some(attr.span()),
    };
    let parse_result =
        attr.parse_nested_meta(
            |meta| match meta.path.require_ident()?.to_string().as_str() {
                "output_schema" => {
                    if res.schema_source.is_some() {
                        let err_msg = "output_schema cannot be set more than once";
                        return Err(meta.error(err_msg));
                    }
                    meta.parse_nested_meta(|meta| {
                        match meta.path.require_ident()?.to_string().as_str() {
                            "PartialSchema" => {
                                let ty = if meta.input.is_empty() {
                                    None
                                } else {
                                    Some(meta.value()?.parse::<syn::LitStr>()?.parse()?)
                                };
                                res.schema_source = Some(SchemaSource::Partial(ty));
                                Ok(())
                            }
                            "ToSchema" => {
                                let ty = if meta.input.is_empty() {
                                    None
                                } else {
                                    Some(meta.value()?.parse::<syn::LitStr>()?.parse()?)
                                };
                                res.schema_source = Some(SchemaSource::ToSchema(ty));
                                Ok(())
                            }
                            ident => {
                                let err_msg = format!("unexpected value: {ident}");
                                Err(syn::Error::new(meta.path.span(), err_msg))
                            }
                        }
                    })
                }
                ident => {
                    let err_msg = format!("unexpected key: {ident}");
                    Err(syn::Error::new(meta.path.span(), err_msg))
                }
            },
        );
    match parse_result {
        Ok(()) => Some(Ok(res)),
        Err(syn_err) => Some(Err(MethodAttrParseError(syn_err))),
    }
}

pub struct MethodOutput {
    pub ty: Box<syn::Type>,
    pub schema_source: SchemaSource,
}

pub struct Method {
    pub ident: Ident,
    pub params: Vec<MethodParam>,
    pub output: Option<MethodOutput>,
    pub description: Option<String>,
}

pub enum MethodError {
    AttrParseError(MethodAttrParseError),
    DuplicateAttr(Span),
    ParamError(ParamError),
}

impl MethodError {
    pub fn into_compile_error(self) -> TokenStream {
        match self {
            Self::AttrParseError(err) => err.into_compile_error(),
            Self::DuplicateAttr(span) => {
                let err_msg = "open_api_method attribute can be used at most once";
                syn::Error::new(span, err_msg).into_compile_error()
            }
            Self::ParamError(err) => err.into_compile_error(),
        }
    }
}

pub struct MethodErrors(pub Vec<MethodError>);

impl MethodErrors {
    pub fn into_compile_errors(self) -> TokenStream {
        self.0
            .into_iter()
            .map(|err| err.into_compile_error())
            .collect()
    }
}

fn get_doc_comment(attr: &Attribute) -> Option<String> {
    let namevalue = attr.meta.require_name_value().ok()?;
    if !namevalue.path.is_ident("doc") {
        return None;
    }
    let syn::Expr::Lit(ref lit) = namevalue.value else {
        return None;
    };
    let syn::Lit::Str(ref lit_str) = lit.lit else {
        return None;
    };
    Some(lit_str.value().trim().to_owned())
}

fn analyze_trait_item_fn_attrs(
    errs: &mut Vec<MethodError>,
    trait_item_fn: &mut TraitItemFn,
) -> MethodAttr {
    let mut res_attr = None;
    trait_item_fn.attrs.retain(|attr| {
        let Some(parse_res) = parse_method_attr(attr) else {
            return true;
        };
        match parse_res {
            Ok(attr) => {
                if res_attr.is_some() {
                    errs.push(MethodError::DuplicateAttr(attr.span.unwrap()))
                } else {
                    res_attr = Some(attr);
                }
            }
            Err(attr_parse_err) => errs.push(MethodError::AttrParseError(attr_parse_err)),
        };
        false
    });
    res_attr.unwrap_or_default()
}

fn analyze_trait_item_fn(trait_item_fn: &mut TraitItemFn) -> Result<Method, MethodErrors> {
    let mut errs = Vec::new();
    let ident = trait_item_fn.sig.ident.clone();
    let method_attr = analyze_trait_item_fn_attrs(&mut errs, trait_item_fn);
    let mut params = Vec::new();
    trait_item_fn
        .sig
        .inputs
        .iter_mut()
        .filter_map(|arg| match arg {
            syn::FnArg::Receiver(_) => None,
            syn::FnArg::Typed(pat_type) => Some(analyze_param(pat_type)),
        })
        .for_each(|res| match res {
            Ok(param) => params.push(param),
            Err(param_errs) => errs.extend(param_errs.0.into_iter().map(MethodError::ParamError)),
        });
    if !errs.is_empty() {
        return Err(MethodErrors(errs));
    }
    let output = match &trait_item_fn.sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(MethodOutput {
            ty: ty.clone(),
            schema_source: method_attr.schema_source.unwrap_or_default(),
        }),
    };
    let doc_comments: Vec<_> = trait_item_fn
        .attrs
        .iter()
        .filter_map(get_doc_comment)
        .collect();
    let description = if doc_comments.is_empty() {
        None
    } else {
        Some(doc_comments.join("\n"))
    };
    Ok(Method {
        ident,
        params,
        output,
        description,
    })
}

pub struct Model {
    pub ref_schema_tys: Vec<syn::Type>,
    pub methods: Vec<Method>,
    pub item_trait: ItemTrait,
}

pub struct Error(pub Vec<MethodErrors>);

impl Error {
    pub fn into_compile_errors(self) -> TokenStream {
        self.0
            .into_iter()
            .map(|err| err.into_compile_errors())
            .collect()
    }
}

pub fn analyze(mut ast: Ast) -> Result<Model, Error> {
    let ref_schema_tys = match ast.ref_schema_tys {
        Some(ref_schema_tys) => Vec::from_iter(ref_schema_tys),
        None => Vec::new(),
    };
    let (mut methods, mut method_errs) = (Vec::new(), Vec::new());
    ast.item_trait
        .items
        .iter_mut()
        .filter_map(|trait_item| match trait_item {
            TraitItem::Fn(trait_item_fn) => Some(analyze_trait_item_fn(trait_item_fn)),
            _ => None,
        })
        .for_each(|res| match res {
            Ok(method) => methods.push(method),
            Err(method_err) => method_errs.push(method_err),
        });
    if method_errs.is_empty() {
        Ok(Model {
            ref_schema_tys,
            methods,
            item_trait: ast.item_trait,
        })
    } else {
        Err(Error(method_errs))
    }
}

#[cfg(test)]
mod tests {}
