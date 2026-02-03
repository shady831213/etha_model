use crate::common::{expand_call, Enum};
use proc_macro2::*;
use quote::format_ident;
use quote::quote;
use syn::parse::*;
use syn::punctuated::*;
use syn::*;

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: Attr = expand_call!(parse2(attr));
    let desc: Desc = expand_call!(parse2(item.clone()));
    let output = expand_call!(desc.expand(&attr));
    quote! {
        #item
        #output
    }
}

mod kw {
    syn::custom_keyword!(padding_to);
}
#[derive(Debug)]
enum PaddingTo {
    Var(Ident),
    Int(LitInt),
}
impl PaddingTo {
    fn expand(&self) -> TokenStream {
        match self {
            PaddingTo::Var(ident) => quote! {#ident},
            PaddingTo::Int(int) => quote! {#int},
        }
    }
}

impl Parse for PaddingTo {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(ident) = input.parse::<Ident>() {
            Ok(PaddingTo::Var(ident))
        } else if let Ok(int) = input.parse::<LitInt>() {
            Ok(PaddingTo::Int(int))
        } else {
            Err(input.error("expected const var or int number"))
        }
    }
}

#[derive(Debug)]
struct Attr {
    padding_to: Option<PaddingTo>,
}
impl Parse for Attr {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(if input.parse::<kw::padding_to>().is_ok() {
            input.parse::<Token![=]>()?;
            Attr {
                padding_to: Some(input.parse::<PaddingTo>()?),
            }
        } else {
            Attr { padding_to: None }
        })
    }
}

#[derive(Debug)]
enum Desc {
    BitField(BitFiled),
    Enum(Enum),
    Struct(Struct),
}

impl Desc {
    fn expand(&self, attr: &Attr) -> Result<TokenStream> {
        match self {
            Desc::BitField(b) => b.expand(),
            Desc::Enum(e) => e.expand(),
            Desc::Struct(s) => s.expand(attr),
        }
    }
}
impl Parse for Desc {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(fields) = input.parse::<BitFiled>() {
            Ok(Desc::BitField(fields))
        } else if let Ok(e) = input.parse::<Enum>() {
            Ok(Desc::Enum(e))
        } else if let Ok(s) = input.parse::<Struct>() {
            Ok(Desc::Struct(s))
        } else {
            Err(input.error("expected bitfield!, enum or struct."))
        }
    }
}

#[derive(Debug)]
struct Struct {
    s: ItemStruct,
}

impl Parse for Struct {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Struct {
            s: input.parse::<ItemStruct>()?,
        })
    }
}

impl Struct {
    fn to_struct_field_ty(&self, ty: &Ident) -> TokenStream {
        let ty_s = ty.to_string();
        match ty_s.as_str() {
            "u64" => quote! {StructFieldType::U64},
            "u32" => quote! {StructFieldType::U32},
            "u16" => quote! {StructFieldType::U16},
            "u8" => quote! {StructFieldType::U8},
            _ => quote! {StructFieldType::Type(#ty_s)},
        }
    }
    fn expand(&self, attr: &Attr) -> Result<TokenStream> {
        let name = &self.s.ident;
        let name_s = self.s.ident.to_string();
        let iter_name = format_ident!("{}Iter", name);
        let fields = self
            .s
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let ident = f
                    .ident
                    .as_ref()
                    .ok_or(Error::new(self.s.ident.span(), "must be named fields!"))?;
                let ident_s = ident.to_string();
                let ty = if let Type::Path(TypePath { path: p, .. }) = &f.ty {
                    p.get_ident()
                        .ok_or(Error::new(ident.span(), "unknowned type!"))?
                } else {
                    return Err(Error::new(ident.span(), "Unsupport struct filed type!"));
                };
                let ty_token = self.to_struct_field_ty(ty);
                Ok(quote! {
                    #i => {
                        self.0 += 1;
                        Some(StructField {name: #ident_s, ty: #ty_token})
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
        let padding = attr.padding_to.as_ref().map(|size| {
            let size = size.expand();
            quote! {
                let self_size = std::mem::size_of::<Self>();
                if self_size < #size {
                    writeln!(header,"    uint8_t padding[{}];", #size - self_size)?;
                }
            }
        });
        Ok(quote! {
            pub struct #iter_name(usize);
            impl #iter_name {
                pub fn new() -> Self {
                    #iter_name(0)
                }
            }

            impl Iterator for #iter_name {
                type Item = StructField;
                fn next(&mut self) -> Option<Self::Item> {
                    match self.0 {
                        #fields
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
                    writeln!(header,"typedef struct {{")?;
                    for f in Self::fields() {
                        let ty = match f.ty {
                            StructFieldType::U64 => "uint64_t",
                            StructFieldType::U32 => "uint32_t",
                            StructFieldType::U16 => "uint16_t",
                            StructFieldType::U8 => "uint8_t",
                            StructFieldType::Type(s) => s,
                        };
                        writeln!(header,"    {} {};", ty, f.name)?;
                    }
                    #padding
                    writeln!(header,"}} {};", Self::render_name())?;
                    Ok(())
                }
            }
        })
    }
}

#[derive(Debug)]
struct BitFiled {
    fields: Fields,
}

impl Parse for BitFiled {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(BitFiled {
            fields: input.parse::<Macro>()?.parse_body()?,
        })
    }
}

impl std::ops::Deref for BitFiled {
    type Target = Fields;
    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

#[derive(Debug)]
struct Fields {
    name: Ident,
    slice_ty: Ident,
    fields: Punctuated<Field, Token![;]>,
}

impl Fields {
    fn expand(&self) -> Result<TokenStream> {
        let name = &self.name;
        let slice_ty = &self.slice_ty;
        let name_s = self.name.to_string();
        let iter_name = format_ident!("{}Iter", self.name);
        let fields = self
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let f_name = f.name.to_string();
                let f_lsb = &f.lsb;
                let f_msb = &f.msb;
                quote! {
                    #i => {
                        self.0 += 1;
                        Some(DescField {name: #f_name, lsb:#f_lsb, msb:#f_msb})
                    }
                }
            })
            .reduce(|acc, e| {
                quote! {
                    #acc
                    #e
                }
            });
        Ok(quote! {
            pub struct #iter_name(usize);
            impl #iter_name {
                pub fn new() -> Self {
                    #iter_name(0)
                }
            }

            impl Iterator for #iter_name {
                type Item = DescField;
                fn next(&mut self) -> Option<Self::Item> {
                    match self.0 {
                        #fields
                        _ => None
                    }
                }
            }

            impl<T: AsRef<[#slice_ty]>> #name<T> {
                pub fn fields() -> #iter_name {
                    #iter_name::new()
                }
            }

            impl<T: AsRef<[#slice_ty]>> GenHeader for #name<T> {
                fn render_name() -> &'static str {
                    #name_s
                }
                fn gen_c_header<W: std::io::Write>(header: &mut W)  -> std::io::Result<()>  {
                    writeln!(header,"typedef struct {{")?;
                    let mut pos = 0;
                    for f in Self::fields() {
                        if f.lsb > pos {
                            writeln!(header,"    uint32_t :{};", f.lsb - pos)?;
                            pos = f.lsb;
                        }
                        writeln!(header,"    uint32_t {}{};", f.name,
                        if f.msb - pos == 31 {
                            "".to_string()
                        } else {
                            format!(": {}", f.msb - pos + 1)
                        })?;
                        pos = f.msb + 1;
                    }
                    let bits_padding = 32 - (pos % 32);
                    if bits_padding != 32 {
                        writeln!(header,"    uint32_t :{};", bits_padding)?;
                        pos += bits_padding;
                    }
                    let word_cnt = pos >> 5;
                    let self_word_cnt = std::mem::size_of::<Self>() >> 2;
                    if word_cnt < self_word_cnt {
                        writeln!(header,"    uint32_t padding[{}];", self_word_cnt - word_cnt)?;
                    }
                    writeln!(header,"}} __attribute__((packed)) {};", Self::render_name())?;
                    Ok(())
                }
            }
        })
    }
}

impl Parse for Fields {
    fn parse(input: ParseStream) -> Result<Self> {
        let st = input.parse::<ItemStruct>()?;
        if input.peek(Token![impl]) {
            input.parse::<Token![impl]>()?;
            input.parse::<Ident>()?;
            input.parse::<Token![;]>()?;
        }
        let slice_ty = input.parse::<Ident>()?;
        input.parse::<Token![;]>()?;
        Ok(Fields {
            name: st.ident,
            slice_ty,
            fields: Punctuated::<Field, Token![;]>::parse_terminated(&input)?,
        })
    }
}

#[derive(Debug)]
struct Field {
    name: Ident,
    lsb: LitInt,
    msb: LitInt,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![pub]) {
            input.parse::<Token![pub]>()?;
        }
        let name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let _: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let msb: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let lsb: LitInt = input.parse()?;

        if msb.base10_parse::<usize>()? < lsb.base10_parse::<usize>()? {
            return Err(Error::new(
                msb.span(),
                format!(
                    "msb {} is smaller than lsb {} !",
                    msb.to_string(),
                    lsb.to_string()
                ),
            ));
        }
        Ok(Field { name, msb, lsb })
    }
}
