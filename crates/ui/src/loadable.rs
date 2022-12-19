#[derive(Debug, Clone)]
pub enum Loadable<T> {
    Loading,
    Loaded(T),
    Error(String),
}
impl<T> From<anyhow::Result<T>> for Loadable<T> {
    fn from(res: anyhow::Result<T>) -> Self {
        match res {
            Ok(value) => Loadable::Loaded(value),
            Err(err) => Loadable::Error(format!("{:#}", err)),
        }
    }
}
#[macro_export]
macro_rules! unwrap_loadables {
    ( $($loadable:ident),+ ) => {
        $(
            let $loadable = match $loadable {
                Loadable::Loading => return $crate::Throbber.el(),
                Loadable::Error(err) => return $crate::Text::el(err),
                Loadable::Loaded(value) => value
            };
        )*
    };
}
