mod common;
mod define_reg;
mod desc;
mod reg_map;
#[proc_macro_attribute]
pub fn desc_gen(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);
    let output: proc_macro2::TokenStream = desc::expand(attr, item);
    proc_macro::TokenStream::from(output)
}

#[proc_macro]
pub fn define_reg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let output: proc_macro2::TokenStream = define_reg::expand(input);
    proc_macro::TokenStream::from(output)
}

#[proc_macro]
pub fn reg_map(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let output: proc_macro2::TokenStream = reg_map::expand(input);
    proc_macro::TokenStream::from(output)
}
