use super::DexClass;
use crate::dex::DexFile;

pub struct ClassIterator<'a> {
    dex: &'a DexFile<'a>,
    index: usize,
    count: usize,
}

impl<'a> ClassIterator<'a> {
    pub fn new(dex: &'a DexFile<'a>, count: usize) -> Self {
        Self {
            dex,
            index: 0,
            count,
        }
    }

    pub fn size(&self) -> usize {
        self.count
    }
}

impl<'a> Iterator for ClassIterator<'a> {
    type Item = crate::Result<DexClass<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }
        let class_def = match self.dex.class_def(self.index) {
            Ok(class_def) => class_def,
            Err(err) => return Some(Err(err)),
        };
        let class = match DexClass::new(self.dex, class_def) {
            Ok(class) => class,
            Err(err) => return Some(Err(err)),
        };
        self.index += 1;
        Some(Ok(class))
    }
}
