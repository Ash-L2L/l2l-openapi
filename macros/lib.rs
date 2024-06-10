use proc_macro::TokenStream;

mod analyze;
mod codegen;
mod lower;
mod parse;

#[proc_macro_attribute]
pub fn open_api(args: TokenStream, item: TokenStream) -> TokenStream {
    let ast = match parse::parse(args.into(), item.into()) {
        Ok(ast) => ast,
        Err(err) => return err.into_compile_error().into(),
    };
    let model = match analyze::analyze(ast) {
        Ok(model) => model,
        Err(err) => return err.into_compile_errors().into(),
    };
    let ir = lower::lower(model);
    let rust = codegen::codegen(ir);
    rust.into()
}
