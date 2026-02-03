use proc_macro2::*;
use quote::format_ident;
use quote::quote;
use syn::parse::*;
use syn::*;

macro_rules! expand_call {
    ($exp:expr) => {
        match $exp {
            Ok(result) => result,
            Err(err) => return err.to_compile_error(),
        }
    };
}
pub(crate) use expand_call;

mod reg_kw {
    syn::custom_keyword!(RO);
    syn::custom_keyword!(WO);
    syn::custom_keyword!(RW);
    syn::custom_keyword!(RW1S);
    syn::custom_keyword!(RW1C);
}

#[derive(Debug)]
pub(crate) enum RegisterAccess {
    RO,
    WO,
    RW,
    RW1S,
    RW1C,
}
impl RegisterAccess {
    pub(crate) fn to_define_csr(&self) -> TokenStream {
        match self {
            RegisterAccess::RO => quote! {RO},
            RegisterAccess::WO | RegisterAccess::RW1S | RegisterAccess::RW1C => quote! {WO},
            RegisterAccess::RW => quote! {RW},
        }
    }
    pub(crate) fn expand(&self) -> TokenStream {
        match self {
            RegisterAccess::RO => quote! {RegisterAccess::RO},
            RegisterAccess::WO => quote! {RegisterAccess::WO},
            RegisterAccess::RW1S => quote! {RegisterAccess::RW1S},
            RegisterAccess::RW1C => quote! {RegisterAccess::RW1C},
            RegisterAccess::RW => quote! {RegisterAccess::RW},
        }
    }
}

impl Parse for RegisterAccess {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.parse::<reg_kw::RO>().is_ok() {
            Ok(RegisterAccess::RO)
        } else if input.parse::<reg_kw::WO>().is_ok() {
            Ok(RegisterAccess::WO)
        } else if input.parse::<reg_kw::RW>().is_ok() {
            Ok(RegisterAccess::RW)
        } else if input.parse::<reg_kw::RW1S>().is_ok() {
            Ok(RegisterAccess::RW1S)
        } else if input.parse::<reg_kw::RW1C>().is_ok() {
            Ok(RegisterAccess::RW1C)
        } else {
            Err(input.error("Invalid access type!"))
        }
    }
}

#[derive(Debug)]
pub(crate) struct Enum {
    e: ItemEnum,
}

impl Parse for Enum {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Enum {
            e: input.parse::<ItemEnum>()?,
        })
    }
}

impl Enum {
    pub(crate) fn expand(&self) -> Result<TokenStream> {
        let name = &self.e.ident;
        let name_s = self.e.ident.to_string();
        let iter_name = format_ident!("{}Iter", name);
        let variants = self
            .e
            .variants
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let ident_s = v.ident.to_string();
                let value = if let Expr::Lit(ExprLit {
                    lit: Lit::Int(value),
                    ..
                }) = &v.discriminant.as_ref().unwrap().1
                {
                    value
                } else {
                    return Err(Error::new(
                        v.ident.span(),
                        "Variant must set value explictly!",
                    ));
                };
                Ok(quote! {
                    #i => {
                        self.0 += 1;
                        Some(EnumVariant {name: #ident_s, value: #value})
                    }
                })
            })
            .reduce(|acc, e| {
                let acc = acc?;
                let e = e?;
                Ok(quote! {
                    #acc
                    #e
                })
            })
            .unwrap()?;

        Ok(quote! {
            pub struct #iter_name(usize);
            impl #iter_name {
                pub fn new() -> Self {
                    #iter_name(0)
                }
            }

            impl Iterator for #iter_name {
                type Item = EnumVariant;
                fn next(&mut self) -> Option<Self::Item> {
                    match self.0 {
                        #variants
                        _ => None
                    }
                }
            }

            impl #name {
                pub fn variants() -> #iter_name {
                    #iter_name::new()
                }
            }

            impl GenHeader for #name {
                fn render_name() -> &'static str {
                    #name_s
                }
                fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()> {
                    writeln!(header,"typedef enum {{")?;
                    for f in Self::variants() {
                        writeln!(header,"    {} = {},", f.name, f.value)?;
                    }
                    writeln!(header,"}} {};", Self::render_name())?;
                    Ok(())
                }
            }
        })
    }
}
