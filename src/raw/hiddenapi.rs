use crate::raw::uleb128;

bitflags::bitflags! {
  /// For more information, click [here][1].
  ///
  /// [1]: https://source.android.com/docs/core/runtime/dex-format#hiddenapi-class-data-item
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  pub struct RestrictionFlag: u64 {
      const Whitelist = 0;
      const Greylist = 1;
      const Blacklist = 2;
      const GreylistMaxO = 3;
      const GreylistMaxP = 4;
      const GreylistMaxQ = 5;
      const GreylistMaxR = 6;
  }
}

impl RestrictionFlag {
    pub fn try_from_uleb128(src: &[u8], offset: &mut usize) -> Result<Self, scroll::Error> {
        let flags = uleb128::read(src, offset)?;
        Ok(RestrictionFlag::from_bits_truncate(flags))
    }

    pub fn try_into_uleb128(
        &self,
        dst: &mut [u8],
        offset: &mut usize,
    ) -> Result<(), scroll::Error> {
        uleb128::write(dst, offset, self.bits())?;
        Ok(())
    }
}
