use syn::ItemTrait;

use crate::analyze::{Method, Model};

pub struct Ir {
    pub ref_schema_tys: Vec<syn::Type>,
    pub methods: Vec<Method>,
    pub item_trait: ItemTrait,
}

pub fn lower(model: Model) -> Ir {
    let Model {
        ref_schema_tys,
        methods,
        item_trait,
    } = model;
    Ir {
        ref_schema_tys,
        methods,
        item_trait,
    }
}

#[cfg(test)]
mod tests {}
