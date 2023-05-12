use crate::internal::wit;

/// Watch a script file.
pub fn watch(url: impl AsRef<str>) {
    println!("watching script: {:?}", &url.as_ref());
    wit::script::watch(url.as_ref());
}

// pub fn url(path: impl AsRef<str>) -> Result<String, UrlError> {
//     Ok(wit::asset::url(path.as_ref())?)
// }