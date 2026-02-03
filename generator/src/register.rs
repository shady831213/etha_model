use super::EnumVariant;
#[derive(Debug)]
pub enum RegisterAccess {
    RO,
    WO,
    RW,
    RW1S,
    RW1C,
}

#[derive(Debug)]
pub struct RegisterField {
    pub name: &'static str,
    pub lsb: usize,
    pub msb: usize,
    pub volatile: bool,
    pub access: RegisterAccess,
    pub enums: Option<Vec<EnumVariant>>,
}

#[derive(Debug)]
pub struct Register {
    pub name: &'static str,
    pub ty: &'static str,
    pub offset: usize,
    pub access: RegisterAccess,
    pub fields: Vec<RegisterField>,
}
