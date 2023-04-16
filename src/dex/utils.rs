macro_rules! section {
    ($struct:ident, $iden:ident, $size:stmt) => {
        paste::paste! {
          fn [<raw_ $iden _section>]<'a>(src: &'a [u8], header: &Header<'a>) -> Result<section::Section<'a>, section::Error> {
              let start = header.[<$iden _off>] as usize;
              let end = start + header.[<$iden _size>] as usize * $size;
              Ok(section::Section::new(&src[start..end], $size))
          }
        }
        impl<'a> $struct<'a> {
            paste::paste! {
                pub fn [<$iden _section>](&self) -> Result<section::Section, section::Error> {
                    [<raw_ $iden _section>](self.src, &self.header)
                }
            }
        }
    };
    (map($item:stmt): $struct:ident, $iden:ident, $size:stmt) => {
        paste::paste! {
          fn [<raw_ $iden _section>]<'a>(src: &'a [u8], map_list: &MapList) -> Result<section::Section<'a>, section::Error> {
              let err = || section::Error::BadSection(stringify!($iden));
              let item_ty = crate::raw::map_list::ItemType::$item;
              let item_off = map_list.get_offset(item_ty).ok_or_else(err)?;
              let item_size = map_list.get_len(item_ty).ok_or_else(err)?;
              let start = item_off as usize;
              let end = start + item_size as usize * $size;
              Ok(section::Section::new(&src[start..end], $size))
          }
        }
        impl<'a> $struct<'a> {
            paste::paste! {
                pub fn [<$iden _section>](&self) -> Result<section::Section, section::Error> {
                    [<raw_ $iden _section>](self.src, &self.map_list)
                }
            }
        }
    };
}
