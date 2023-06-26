use crate::dex::{methods::exceptions::TryCatchBlock, traits, DexFile};
use crate::raw::code_item::{CodeItem, DebugInfoItem};
use crate::{error::Error, Result};
use scroll::Pread;

use super::exceptions::{CatchHandler, ExceptionType};

#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub struct DexMethodImplementation<'a> {
    #[derivative(Debug = "ignore")]
    dex: &'a DexFile<'a>,
    midx: usize,
    raw: CodeItem,
}

impl<'a> DexMethodImplementation<'a> {
    pub fn new(dex: &'a DexFile<'a>, midx: usize, raw: CodeItem) -> Self {
        Self { dex, midx, raw }
    }
}

impl<'a> traits::MethodImplementation for DexMethodImplementation<'a> {
    fn registers(&self) -> u16 {
        self.raw.registers_size
    }

    fn instructions(&self) -> Result<Vec<u16>> {
        todo!()
    }

    fn try_blocks(&self) -> Result<Vec<TryCatchBlock>> {
        let mut try_blocks = Vec::new();
        let raw_handlers = match self.raw.handlers {
            Some(ref handlers) => handlers,
            None => return Ok(try_blocks),
        };
        for item in &self.raw.tries {
            let mut catch_handlers = Vec::new();
            let eh = raw_handlers
                .find(item.handler_off)
                .ok_or_else(|| Error::InvalidExceptionHandler(item.handler_off))?;
            for handler in &eh.handlers {
                let id = self
                    .dex
                    .strings()
                    .id_at_type_idx(handler.type_id.try_into().expect("bad type idx?"))?;
                catch_handlers.push(CatchHandler {
                    exception: ExceptionType::Type(self.dex.strings().get(&id)?),
                    addr: handler.addr,
                });
            }
            if let Some(addr) = eh.catch_all_addr {
                catch_handlers.push(CatchHandler {
                    exception: ExceptionType::Base,
                    addr,
                });
            }
            try_blocks.push(TryCatchBlock {
                start_addr: item.start_addr,
                insn_count: item.insn_count,
                catch_handlers,
            })
        }
        Ok(try_blocks)
    }

    fn debug_info(&self) -> Result<DebugInfoItem> {
        Ok(self.dex.src.pread(self.raw.debug_info_off as usize)?)
    }
}
