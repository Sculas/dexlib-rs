use std::marker::PhantomData;

#[must_use = "this is a lazy set and will not be evaluated until iterated over"]
pub struct LazySet<'a, T> {
    getter: Box<dyn Fn(usize) -> T + 'a>,
    size: usize,
    phantom: PhantomData<&'a T>,
}

impl<'a, T> LazySet<'a, T> {
    pub fn new<F>(size: usize, getter: F) -> Self
    where
        F: Fn(usize) -> T + 'a,
    {
        Self {
            getter: Box::new(getter),
            size,
            phantom: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            getter: Box::new(|_| unreachable!("empty set should not be accessed")),
            size: 0,
            phantom: PhantomData,
        }
    }

    pub fn get(&self, index: usize) -> T {
        if index >= self.size() {
            panic!("index out of bounds, index: {index}, size: {}", self.size);
        }
        (self.getter)(index)
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<'a, T> IntoIterator for LazySet<'a, T> {
    type Item = T;
    type IntoIter = ListIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ListIterator {
            list: self,
            index: 0,
        }
    }
}

pub struct ListIterator<'a, T> {
    list: LazySet<'a, T>,
    index: usize,
}

impl<'a, T> Iterator for ListIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.list.size() {
            return None;
        }
        let item = self.list.get(self.index);
        self.index += 1;
        Some(item)
    }
}
