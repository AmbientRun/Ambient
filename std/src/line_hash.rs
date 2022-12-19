/// Create a hash of the current file/line/column
#[macro_export]
#[doc(hidden)]
macro_rules! line_hash {
    ($s:expr) => {{
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let id = $s;

        let mut s = DefaultHasher::new();
        id.hash(&mut s);
        s.finish()
    }};
    () => {{
        let id = concat!(file!(), line!(), column!());
        line_hash!(id)
    }};
    ($($s:expr),*) => {{
        let mut s: u128 = 0;
        $(s += $crate::line_hash!($s) as u128;)*
        $crate::line_hash!(s)
    }};
}
