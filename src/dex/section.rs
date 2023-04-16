use scroll::{ctx::TryFromCtx, Pread};
use std::cmp::Ordering;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("attempted to read from bad/nonexistent section: {0}")]
    BadSection(&'static str),
}

pub struct Section<'a> {
    inner: &'a [u8],
    type_size: usize,
}

impl<'a> Section<'a> {
    pub(crate) fn new(inner: &'a [u8], type_size: usize) -> Self {
        Section { inner, type_size }
    }

    pub(crate) fn index<Ctx: Copy, N>(&self, index: usize, ctx: Ctx) -> Result<N, N::Error>
    where
        N: scroll::ctx::TryFromCtx<'a, Ctx>,
        N::Error: From<scroll::Error>,
    {
        let mut offset = index * self.type_size;
        if offset >= self.inner.len() {
            return Err(scroll::Error::BadOffset(offset).into());
        }
        // manual impl of pread_with to work around trait bound issues
        N::try_from_ctx(&self.inner[offset..], ctx).map(|(n, size)| {
            offset += size;
            n
        })
    }

    /// Binary search the contents of this section.
    /// * The items in the section should be of fixed size.
    /// * The items must be sorted in the order that predicate expects.
    ///
    /// Taken from: https://github.com/letmutx/dex-parser/blob/c3bc1fc/src/search.rs
    pub(crate) fn binary_search<'b, F, T, S, C: Copy, E>(
        &self,
        element: &'b S,
        ctx: C,
        predicate: F,
    ) -> Result<Option<usize>, E>
    where
        F: Fn(&T, &S) -> Result<Ordering, E>,
        T: TryFromCtx<'a, C, Error = scroll::Error>,
        E: From<scroll::Error>,
    {
        if self.inner.is_empty() {
            return Ok(None);
        }
        // Figure out the size of one item, all items must be of fixed size
        let mut size = 0;
        let _: T = self.inner.gread_with(&mut size, ctx)?;
        // Number of elements  = Size of buffer / Item size
        let len = self.inner.len() / size;
        let (mut start, mut end) = (0, len - 1);
        while start < end {
            let mid = start + (end - start) / 2;
            let mid_offset = mid * size;
            let item = self.inner.pread_with(mid_offset, ctx)?;
            let result = predicate(&item, element)?;
            match result {
                Ordering::Equal => return Ok(Some(mid)),
                Ordering::Less => end = mid - 1,
                Ordering::Greater => start = mid + 1,
            }
        }
        let start_offset = start * size;
        let item = self.inner.pread_with(start_offset, ctx)?;
        Ok(if predicate(&item, element)? == Ordering::Equal {
            Some(start)
        } else {
            None
        })
    }
}

impl<'a> AsRef<[u8]> for Section<'a> {
    fn as_ref(&self) -> &[u8] {
        self.inner
    }
}
