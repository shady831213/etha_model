use crate::common::expand_call;
use crate::common::RegisterAccess;
use proc_macro2::*;
use quote::format_ident;
use quote::quote;
use syn::parse::*;
use syn::punctuated::*;
use syn::*;
pub fn expand(input: TokenStream) -> TokenStream {
    let reg: RegMap = expand_call!(parse2(input));
    let output = expand_call!(reg.expand());
    quote! {
        #output
    }
}

#[derive(Debug)]
struct RegMap {
    is_pub: bool,
    name: Ident,
    size: LitInt,
    regs: Regs,
}

impl RegMap {
    fn expand(&self) -> Result<TokenStream> {
        let name = &self.name;
        let name_s = name.to_string();
        let size = &self.size;
        let hi = self.size.base10_parse::<usize>()? - 1;
        let vis = if self.is_pub {
            quote! {pub}
        } else {
            quote! {}
        };
        let iter_name = format_ident!("{}Iter", name);
        let (regs, iters) = self.regs.expand()?;
        Ok(quote! {
            csr_map! {
                #vis #name (0, #hi) {
                    #regs
                }
            }
            pub struct #iter_name(usize);
            impl #iter_name {
                pub fn new() -> Self {
                    #iter_name(0)
                }
            }
            impl Iterator for #iter_name {
                type Item = Register;
                fn next(&mut self) -> Option<Self::Item> {
                    match self.0 {
                        #iters
                        _ => None
                    }
                }
            }
            impl #name {
                pub fn regs() -> #iter_name {
                    #iter_name::new()
                }
            }
            impl GenHeader for #name {
                fn render_name() -> &'static str {
                    #name_s
                }
                fn gen_c_header<W: std::io::Write>(header: &mut W) -> std::io::Result<()>  {
                    writeln!(header,"")?;
                    writeln!(header,"#define {}_SIZE {:#x}", Self::render_name().from_case(Case::UpperCamel).to_case(Case::UpperSnake), #size)?;
                    for r in Self::regs() {
                        writeln!(header,"// macros: {}*", r.ty.from_case(Case::UpperCamel).to_case(Case::UpperSnake))?;
                        writeln!(header,"#define {}_{}_OFFSET {:#x}", Self::render_name().from_case(Case::UpperCamel).to_case(Case::UpperSnake), r.name.from_case(Case::Snake).to_case(Case::UpperSnake), r.offset)?;
                    }
                    writeln!(header,"")?;
                    Ok(())
                }
            }
        })
    }
}

impl Parse for RegMap {
    fn parse(input: ParseStream) -> Result<Self> {
        let is_pub = if input.peek(Token![pub]) {
            input.parse::<Token![pub]>()?;
            true
        } else {
            false
        };
        let name: Ident = input.parse::<Ident>()?;
        let content: ParseBuffer;
        parenthesized!(content in input);
        let size = content.parse::<LitInt>()?;
        let content1: ParseBuffer;
        braced!(content1 in input);
        Ok(RegMap {
            is_pub,
            name,
            size,
            regs: content1.parse()?,
        })
    }
}

#[derive(Debug)]
struct Regs {
    regs: Punctuated<Reg, Token![;]>,
}

impl Regs {
    fn expand(&self) -> Result<(TokenStream, TokenStream)> {
        let regs = self
            .regs
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
        Ok(regs)
    }
}

impl Parse for Regs {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Regs {
            regs: input.parse_terminated(Reg::parse)?,
        })
    }
}

#[derive(Debug)]
struct Reg {
    name: Ident,
    access: RegisterAccess,
    ty: Ident,
    offset: LitInt,
}

impl Reg {
    fn expand_iter(&self, i: usize) -> Result<TokenStream> {
        let name_s = self.name.to_string();
        let ty = &self.ty;
        let ty_s = self.ty.to_string();
        let offset = &self.offset;
        let access = self.access.expand();
        Ok(quote! {
            #i => {
                self.0 += 1;
                Some(Register {name: #name_s,
                ty: #ty_s,
                offset: #offset,
                access: #access,
                fields: #ty::fields().collect::<Vec<_>>()})
            }
        })
    }
    fn expand(&self) -> Result<TokenStream> {
        let name = &self.name;
        let access = self.access.to_define_csr();
        let ty = &self.ty;
        let offset = &self.offset;
        Ok(quote! {
            #name(#access) : #ty, #offset;
        })
    }
}

impl Parse for Reg {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        let content: ParseBuffer;
        parenthesized!(content in input);
        let access = content.parse::<RegisterAccess>()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let offset = input.parse::<LitInt>()?;
        Ok(Reg {
            name,
            access,
            ty,
            offset,
        })
    }
}
