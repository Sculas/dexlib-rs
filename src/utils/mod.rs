pub mod leb128;

macro_rules! try_gread_vec_with {
    ($src:ident, $offset:ident, $cap:expr, $ctx:expr) => {{
        let mut vec = Vec::with_capacity($cap as usize);
        for _ in 0..$cap {
            vec.push($src.gread_with($offset, $ctx)?);
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
