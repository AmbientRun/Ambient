/// Create a hash of the current file/line/column
#[macro_export]
#[doc(hidden)]
macro_rules! line_hash {
    (($s:expr),*) => {{
        format!("{:?}", $($s),*)
    }};
    () => {{
        concat!(file!(), line!(), column!())
    }}
}
