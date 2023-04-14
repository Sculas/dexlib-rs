macro_rules! section {
    ($struct:ident, $iden:ident, $size:stmt) => {
        paste::paste! {
          fn [<raw_ $iden _section>]<'a>(src: &'a [u8], header: &Header<'a>) -> section::Section<'a> {
              let start = header.[<$iden _off>] as usize;
              let end = start + header.[<$iden _size>] as usize * $size;
              section::Section::new(&src[start..end])
          }
        }
        impl<'a> $struct<'a> {
            paste::paste! {
                pub fn [<$iden _section>](&self) -> section::Section {
                    [<raw_ $iden _section>](self.src, &self.header)
                }
            }
        }
    };
}
