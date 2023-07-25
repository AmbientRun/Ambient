#[macro_export]
#[doc(hidden)]
/// Create a unique identifier from the current file, line, and column
macro_rules! line_uid {
    (($s:expr),*) => {{
        format!("{}:{}:{}:{:?}",file!(),line!(), column!(), $($s),*)
    }};
    () => {{
        concat!(file!(), ":" , line!(), ":", column!())
    }}
}
