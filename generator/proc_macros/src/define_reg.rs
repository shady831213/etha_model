use crate::common::expand_call;
use crate::common::RegisterAccess;
use proc_macro2::*;
use quote::format_ident;
use quote::quote;
use syn::parse::*;
use syn::punctuated::*;
use syn::*;
pub fn expand(input: TokenStream) -> TokenStream {
    let reg: Reg = expand_call!(parse2(input));
    let output = expand_call!(reg.expand());
    quote! {
        #output
    }
}

mod kw {
    syn::custom_keyword!(fields);
    syn::custom_keyword!(volatile);
}

#[derive(Debug)]
struct Reg {
    name: Ident,
    fields: Fields,
}
impl Reg {
    fn expand(&self) -> Result<TokenStream> {
        let name = &self.name;
        let name_s = name.to_string();
        let iter_name = format_ident!("{}Iter", name);
        let (fields, iters) = self.fields.expand()?;
        Ok(quote! {
            define_csr! {
                #name {
                    fields {
                        #fields
                    }
                }
            }
            pub struct #iter_name(usize);
            impl #iter_name {
                pub fn new() -> Self {
                    #iter_name(0)
                }
            }
            impl Iterator for #iter_name {
                type Item = RegisterField;
                fn next(&mut self) -> Option<Self::Item> {
                    match self.0 {
                        #iters
                        _ => None
                    }
                }
            }
            impl #name {
                pub fn fields() -> #iter_name {
                    #iter_name::new()
                }
            }
            impl GenHeader for #name {
                fn render_name() -> &'static str {
                    #name_s
                }
                fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()>  {
                    for f in Self::fields() {
                        let field_name = format!("{}_{}", Self::render_name().from_case(Case::UpperCamel).to_case(Case::UpperSnake), f.name.from_case(Case::Snake).to_case(Case::UpperSnake));
                        writeln!(header,"")?;
                        writeln!(header,"#define {}_POS {}", &field_name, f.lsb)?;
                        writeln!(header,"#define {}_FLAGS {:#x}", &field_name, if f.msb - f.lsb == 31 {
                            0xffffffff
                        } else {
                            (1 << (f.msb - f.lsb + 1)) as u32 - 1
                        })?;
                        if let Some(vs) = f.enums {
                            write!(header,"//Enum:")?;
                            for v in vs.iter() {
                                write!(header, " {} : {:#x};", v.name, v.value)?;
                            }
                            write!(header,"\n")?;
                        }
                        writeln!(header,"#define {}(x) (((x) >> {}_POS) & {}_FLAGS)", &field_name, &field_name, &field_name)?;
                        writeln!(header,"#define SET_{}(x) (((x) & {}_FLAGS) << {}_POS)", &field_name, &field_name, &field_name)?;
                        writeln!(header,"")?;
                    }
                    Ok(())
                }
            }
        })
    }
}
impl Parse for Reg {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse::<Ident>()?;
        let content: ParseBuffer;
        braced!(content in input);
        content.parse::<kw::fields>()?;
        let content1: ParseBuffer;
        braced!(content1 in content);
        Ok(Reg {
            name,
            fields: content1.parse()?,
        })
    }
}

#[derive(Debug)]
struct Fields {
    fields: Punctuated<Field, Token![;]>,
}
impl Fields {
    fn expand(&self) -> Result<(TokenStream, TokenStream)> {
        let fields = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| Ok((f.expand()?, f.expand_iter(i)?)))
            .reduce(|acc: Result<(TokenStream, TokenStream)>, e| {
                let (acc_csr, acc_iter) = acc?;
                let (e_csr, e_iter) = e?;
                Ok((
                    quote! {
                        #acc_csr
                        #e_csr
                    },
                    quote! {
                        #acc_iter
                        #e_iter
                    },
                ))
            })
            .unwrap()?;
        Ok(fields)
    }
}
impl Parse for Fields {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Fields {
            fields: input.parse_terminated(Field::parse)?,
        })
    }
}

#[derive(Debug)]
struct Field {
    name: Ident,
    access: RegisterAccess,
    volatile: bool,
    enums: Option<Punctuated<RegEnum, Token![,]>>,
    msb: LitInt,
    lsb: LitInt,
}
impl Field {
    fn expand_iter(&self, i: usize) -> Result<TokenStream> {
        let name_s = self.name.to_string();
        let lsb = &self.lsb;
        let msb = &self.msb;
        let volatile = self.volatile;
        let access = self.access.expand();
        let enums = if let Some(es) = &self.enums {
            let enums = es
                .iter()
                .map(|e| {
                    let name_s = e.name.to_string();
                    let value = &e.value;
                    quote! {
                        EnumVariant {name: #name_s, value: #value},
                    }
                })
                .reduce(|acc, a| {
                    quote! {
                        #acc
                        #a
                    }
                })
                .unwrap();
            quote! {
                Some(vec![#enums])
            }
        } else {
            quote! {None}
        };
        Ok(quote! {
            #i => {
                self.0 += 1;
                Some(RegisterField {name: #name_s,
                lsb: #lsb,
                msb: #msb,
                volatile:#volatile,
                access: #access,
                enums: #enums})
            }
        })
    }
    fn expand(&self) -> Result<TokenStream> {
        let name = &self.name;
        let access = self.access.to_define_csr();
        let msb = &self.msb;
        let lsb = &self.lsb;
        Ok(quote! {
            #name(#access) : #msb, #lsb;
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        let content: ParseBuffer;
        parenthesized!(content in input);
        let access = content.parse::<RegisterAccess>()?;
        let volatile = if content.parse::<Token![,]>().is_ok() {
            content.parse::<kw::volatile>()?;
            true
        } else {
            false
        };
        let enums = if input.peek(token::Brace) {
            let content: ParseBuffer;
            braced!(content in input);
            let enums: Punctuated<RegEnum, Token![,]> = content.parse_terminated(RegEnum::parse)?;
            Some(enums)
        } else {
            None
        };
        input.parse::<Token![:]>()?;
        let msb = input.parse::<LitInt>()?;
        input.parse::<Token![,]>()?;
        let lsb = input.parse::<LitInt>()?;
        Ok(Field {
            name,
            access,
            volatile,
            enums,
            msb,
            lsb,
        })
    }
}

#[derive(Debug)]
struct RegEnum {
    name: Ident,
    value: LitInt,
}
impl Parse for RegEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let value = input.parse::<LitInt>()?;
        Ok(RegEnum { name, value })
    }
}
