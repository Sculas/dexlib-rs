pub mod leb128;
pub(crate) mod nohash;

macro_rules! try_gread_vec_with {
    ($src:ident, $offset:ident, $cap:expr, $ctx:expr) => {{
        let mut vec = Vec::with_capacity($cap as usize);
        for _ in 0..$cap {
            vec.push($src.gread_with($offset, $ctx)?);
        }
        vec
    }};
    ($src:ident, $offset:ident, $cap:expr; ctx = offset) => {{
        let mut vec = Vec::with_capacity($cap as usize);
        for _ in 0..$cap {
            vec.push($src.gread_with($offset, *$offset)?);
        }
        vec
    }};
}

macro_rules! try_gwrite_vec_with {
    ($dst:ident, $offset:ident, $vec:expr, $ctx:expr) => {
        for item in $vec {
            $dst.gwrite_with(item, $offset, $ctx)?;
        }
    };
}

macro_rules! count_delim {
    ($src:ident, $offset:ident, $delim:literal) => {
        $src.iter()
            .skip(*$offset)
            .take_while(|c| **c != $delim)
            .count()
    };
}

#[cfg(debug_assertions)]
macro_rules! assert_sz {
    ($($const:ident; $struct:ty)+) => {
        $(const _: [(); /* SIZE VALIDATION FAILED! */ $const - std::mem::size_of::<$struct>()] = [];)+
    };
}

pub trait IntoArc<T> {
    fn into_arc(self) -> std::sync::Arc<T>;
}

impl<T> IntoArc<T> for T {
    fn into_arc(self) -> std::sync::Arc<T> {
        std::sync::Arc::new(self)
    }
}
