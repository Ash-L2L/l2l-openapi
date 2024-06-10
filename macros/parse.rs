use proc_macro2::TokenStream;
use syn::{parse::Parser, punctuated::Punctuated, spanned::Spanned, Item, ItemTrait};

pub struct Ast {
    /// Reference schema types
    pub ref_schema_tys: Option<Punctuated<syn::Type, syn::token::Comma>>,
    pub item_trait: ItemTrait,
}

pub fn parse(args: TokenStream, item: TokenStream) -> syn::Result<Ast> {
    let mut ref_schema_tys: Option<Punctuated<syn::Type, syn::token::Comma>> = None;
    let args_parser =
        syn::meta::parser(
            |meta| match meta.path.require_ident()?.to_string().as_str() {
                "ref_schemas" => {
                    if ref_schema_tys.is_some() {
                        let err_msg = "ref_schemas cannot be set more than once";
                        return Err(meta.error(err_msg));
                    }
                    let schema_tys_expr;
                    syn::bracketed!(schema_tys_expr in meta.input);
                    ref_schema_tys = Some(
                        <Punctuated<syn::Type, syn::token::Comma>>::parse_terminated(
                            &schema_tys_expr,
                        )?,
                    );
                    Ok(())
                }
                ident => {
                    let err_msg = format!("unexpected key: {ident}");
                    Err(syn::Error::new(meta.path.span(), err_msg))
                }
            },
        );
    let () = args_parser.parse2(args)?;

    match syn::parse2::<Item>(item) {
        Ok(Item::Trait(item_trait)) => Ok(Ast {
            ref_schema_tys,
            item_trait,
        }),
        Ok(_item) => {
            // ../tests/ui/item-is-not-a-function.rs
            panic!("item is not a trait definition")
        }
        Err(_) => unreachable!(), // ?
    }
}
