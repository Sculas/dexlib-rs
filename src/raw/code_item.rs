use crate::raw::{encoded_value::EncodedCatchHandlerList, simple::TryItem, *};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use scroll::{
    ctx::{TryFromCtx, TryIntoCtx},
    Pread, Pwrite,
};

type TriesPadding = ushort;

#[derive(Debug)]
pub struct CodeItem {
    pub registers_size: ushort,
    pub ins_size: ushort,
    pub outs_size: ushort,
    pub debug_info_off: uint,
    pub insns: Vec<ushort>,
    pub tries: Vec<TryItem>,
    pub handlers: Option<EncodedCatchHandlerList>,
}

impl<'a> TryFromCtx<'a, scroll::Endian> for CodeItem {
    type Error = scroll::Error;
    fn try_from_ctx(src: &'a [u8], ctx: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let registers_size = src.gread_with(offset, ctx)?;
        let ins_size = src.gread_with(offset, ctx)?;
        let outs_size = src.gread_with(offset, ctx)?;
        let tries_size: ushort = src.gread_with(offset, ctx)?;
        let debug_info_off = src.gread_with(offset, ctx)?;
        let insns_size: uint = src.gread_with(offset, ctx)?;
        let insns = try_gread_vec_with!(src, offset, insns_size, ctx);
        // 2 bytes of padding to make `tries` four-byte aligned.
        // This element is only present if `tries_size` is non-zero and `insns_size` is odd.
        if insns_size % 2 != 0 && tries_size != 0 {
            src.gread_with::<TriesPadding>(offset, ctx)?;
        }
        let tries = try_gread_vec_with!(src, offset, tries_size, ctx);
        let handlers = if tries_size != 0 {
            Some(src.gread(offset)?)
        } else {
            None
        };
        Ok((
            Self {
                registers_size,
                ins_size,
                outs_size,
                debug_info_off,
                insns,
                tries,
                handlers,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx<scroll::Endian> for CodeItem {
    type Error = scroll::Error;
    fn try_into_ctx(self, dst: &mut [u8], ctx: scroll::Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.registers_size, offset, ctx)?;
        dst.gwrite_with(self.ins_size, offset, ctx)?;
        dst.gwrite_with(self.outs_size, offset, ctx)?;
        dst.gwrite_with(self.tries.len() as ushort, offset, ctx)?;
        dst.gwrite_with(self.debug_info_off, offset, ctx)?;
        dst.gwrite_with(self.insns.len() as uint, offset, ctx)?;
        try_gwrite_vec_with!(dst, offset, &self.insns, ctx);
        // 2 bytes of padding to make `tries` four-byte aligned.
        // This element is only present if `tries_size` is non-zero and `insns_size` is odd.
        if self.insns.len() % 2 != 0 && !self.tries.is_empty() {
            dst.gwrite_with::<TriesPadding>(0, offset, ctx)?;
        }
        try_gwrite_vec_with!(dst, offset, self.tries, ctx);
        if let Some(handlers) = self.handlers {
            dst.gwrite(handlers, offset)?;
        }
        Ok(*offset)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DebugInfoError {
    #[error("invalid operation in debug info: {0}")]
    InvalidOperation(ubyte),
    #[error("read error: {0}")]
    Scroll(#[from] scroll::Error),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DebugInfoItem {
    /// The line number where the information in this item starts.
    pub line_start: ulong,
    /// The list of parameter names for this method.
    /// `Some` means the parameter has a name, `None` means it doesn't.
    pub parameter_names: Vec<Option<ulong>>,
}

/// See https://source.android.com/docs/core/runtime/dex-format#debug-info-item
#[derive(FromPrimitive, Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum DebugInfoOperation {
    EndSequence = 0x00,
    AdvancePc = 0x01,
    AdvanceLine = 0x02,
    StartLocal = 0x03,
    StartLocalExtended = 0x04,
    EndLocal = 0x05,
    RestartLocal = 0x06,
    SetPrologueEnd = 0x07,
    SetEpilogueBegin = 0x08,
    SetFile = 0x09,
}
const SPECIAL: std::ops::RangeInclusive<u8> = 0x0a..=0xff;

impl<'a> TryFromCtx<'a> for DebugInfoItem {
    type Error = DebugInfoError;
    fn try_from_ctx(src: &'a [u8], _: ()) -> Result<(Self, usize), Self::Error> {
        let offset = &mut 0;
        let line_start = uleb128::read(src, offset)?;
        let parameters_size = uleb128::read(src, offset)?;
        let mut parameter_names = Vec::with_capacity(parameters_size as usize);
        for _ in 0..parameters_size {
            let idx = uleb128::read(src, offset)?;
            parameter_names.push(if idx != NO_INDEX.into() {
                Some(idx + 1) // uleb128p1
            } else {
                None
            });
        }
        // TODO: Implement DWARF3 state machine.
        // Currently, we just discard the debug info.
        loop {
            let byte = src.gread_with::<u8>(offset, scroll::LE)?;
            match DebugInfoOperation::from_u8(byte) {
                Some(op) => match op {
                    DebugInfoOperation::EndSequence => {
                        break;
                    }
                    DebugInfoOperation::AdvancePc => {
                        uleb128::read(src, offset)?;
                    }
                    DebugInfoOperation::AdvanceLine => {
                        sleb128::read(src, offset)?;
                    }
                    DebugInfoOperation::StartLocal => {
                        uleb128::read(src, offset)?;
                        uleb128::read(src, offset)?;
                        uleb128::read(src, offset)?;
                    }
                    DebugInfoOperation::StartLocalExtended => {
                        uleb128::read(src, offset)?;
                        uleb128::read(src, offset)?;
                        uleb128::read(src, offset)?;
                        uleb128::read(src, offset)?;
                    }
                    DebugInfoOperation::EndLocal => {
                        uleb128::read(src, offset)?;
                    }
                    DebugInfoOperation::RestartLocal => {
                        uleb128::read(src, offset)?;
                    }
                    DebugInfoOperation::SetPrologueEnd => {}
                    DebugInfoOperation::SetEpilogueBegin => {}
                    DebugInfoOperation::SetFile => {
                        uleb128::read(src, offset)?;
                    }
                },
                None => {
                    if !SPECIAL.contains(&byte) {
                        return Err(DebugInfoError::InvalidOperation(byte));
                    }
                }
            }
        }
        Ok((
            Self {
                line_start,
                parameter_names,
            },
            *offset,
        ))
    }
}

impl TryIntoCtx for DebugInfoItem {
    type Error = DebugInfoError;
    fn try_into_ctx(self, dst: &mut [u8], _: ()) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        uleb128::write(dst, offset, self.line_start)?;
        uleb128::write(dst, offset, self.parameter_names.len() as u64)?;
        for idx in self.parameter_names {
            if let Some(idx) = idx {
                uleb128::write(dst, offset, idx - 1)?; // uleb128p1
            } else {
                uleb128::write(dst, offset, NO_INDEX.into())?;
            }
        }
        // TODO: Implement DWARF3 state machine.
        // Currently, we just immediately end the sequence.
        dst.gwrite_with::<u8>(DebugInfoOperation::EndSequence as u8, offset, scroll::LE)?;
        Ok(*offset)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn debug_info() {
        let v = super::DebugInfoItem {
            line_start: 256,
            parameter_names: vec![Some(256), None, Some(1)],
        };
        let mut buf = [0u8; 1024];
        let len = scroll::ctx::TryIntoCtx::try_into_ctx(v.clone(), &mut buf, ()).unwrap();
        let (v2, _) = scroll::ctx::TryFromCtx::try_from_ctx(&buf[..len], ()).unwrap();
        assert_eq!(v, v2);
    }
}
