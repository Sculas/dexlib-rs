macro_rules! section {
    ($iden:ident, $size:stmt) => {
        paste::paste! {
          pub fn [<$iden _section>](&self) -> section::Section {
              let start = self.header.[<$iden _off>] as usize;
              let end = start + self.header.[<$iden _size>] as usize * $size;
              section::Section::new(&self.src[start..end])
          }
        }
    };
}
