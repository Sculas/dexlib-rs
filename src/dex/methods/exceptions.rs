use crate::{
    dex::strings::DexString,
    raw::{uint, ulong, ushort},
};

#[derive(Debug, Clone)]
pub struct TryCatchBlock {
    pub start_addr: uint,
    pub insn_count: ushort,
    pub catch_handlers: Vec<CatchHandler>,
}

#[derive(Debug, Clone)]
pub struct CatchHandler {
    pub exception: ExceptionType,
    pub addr: ulong,
}

#[derive(Debug, Clone)]
pub enum ExceptionType {
    Base,
    Type(DexString),
}
