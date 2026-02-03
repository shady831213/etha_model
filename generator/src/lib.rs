mod desc;
mod register;
pub use convert_case::*;
pub use desc::*;
pub use etha_model_generator_macros::*;
pub use register::*;
pub use terminus_vault::*;

#[derive(Debug)]
pub struct EnumVariant {
    pub name: &'static str,
    pub value: usize,
}

#[derive(Debug)]
pub enum StructFieldType {
    U64,
    U32,
    U16,
    U8,
    Type(&'static str),
}

#[derive(Debug)]
pub struct StructField {
    pub name: &'static str,
    pub ty: StructFieldType,
}

#[derive(Debug)]
pub enum HeaderType {
    C,
}

pub trait GenHeader {
    fn render_name() -> &'static str;
    fn gen_header<W: std::io::Write>(
        header_type: &HeaderType,
        header: &mut W,
    ) -> std::io::Result<()> {
        writeln!(header, "")?;
        match header_type {
            HeaderType::C => Self::gen_c_header(header),
        }?;
        writeln!(header, "")?;
        Ok(())
    }

    fn gen_c_header<W: std::io::Write>(_header: &mut W) -> std::io::Result<()> {
        panic!("{}: unsopport gen_c_header!", Self::render_name())
    }
}

pub trait ObjGenHeader {
    fn gen_header<W: std::io::Write>(
        &self,
        header_type: &HeaderType,
        header: &mut W,
    ) -> std::io::Result<()> {
        writeln!(header, "")?;
        match header_type {
            HeaderType::C => self.gen_c_header(header),
        }?;
        writeln!(header, "")?;
        Ok(())
    }

    fn gen_c_header<W: std::io::Write>(&self, _header: &mut W) -> std::io::Result<()> {
        panic!("unsopport gen_c_header!")
    }
}
