use scroll::{ctx::TryFromCtx, Pread};
use std::{cmp::Ordering, fmt::Debug};

// Taken from: https://github.com/letmutx/dex-parser/blob/c3bc1fc/src/search.rs
// Thank you, letmutx!

pub struct Section<'a> {
    inner: &'a [u8],
}

impl<'a> Section<'a> {
    pub(crate) fn new(inner: &'a [u8]) -> Self {
        Section { inner }
    }

    /// Binary search the contents of this section.
    /// * The items in the section should be of fixed size.
    /// * The items must be sorted in the order that predicate expects.
    pub(crate) fn binary_search<'b, F, T, S, C: Copy, E>(
        &self,
        element: &'b S,
        ctx: C,
        predicate: F,
    ) -> Result<Option<usize>, E>
    where
        F: Fn(&T, &S) -> Result<Ordering, E>,
        T: TryFromCtx<'a, C, Error = scroll::Error> + Debug,
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
