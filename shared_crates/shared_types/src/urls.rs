use std::fmt::Display;

pub const AMBIENT_WEB_APP_URL: &str = "https://ambient-733e7.web.app";
pub const ASSETS_URL: &str = "https://assets.ambient.run";
pub const API_URL: &str = "https://api.ambient.run";

/// The URL for a deployed package on the website.
///
/// What the user would visit to play this package on the website.
pub fn web_package_url(package_id: &str, deployment_id: Option<&str>) -> String {
    let mut output = format!("{AMBIENT_WEB_APP_URL}/packages/{package_id}");
    if let Some(deployment_id) = deployment_id {
        output.push_str(&format!("/deployment/{deployment_id}"));
    }
    output
}

/// The URL for a deployed package on the asset server.
///
/// What the user would use to run this package, or to get its assets.
pub fn deployment_url(deployment_id: &str) -> String {
    format!("{ASSETS_URL}/{deployment_id}")
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerSelector<'a> {
    Deployment(&'a str),
    Package(&'a str),
}
impl Display for ServerSelector<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerSelector::Deployment(id) => write!(f, "deployment_id={id}"),
            ServerSelector::Package(id) => write!(f, "package_id={id}"),
        }
    }
}

/// The URL for a ensure-running server for a deployed package.
///
/// When connecting to this URL, a server will be started if one is not already running.
pub fn ensure_running_url(selector: ServerSelector) -> String {
    format!("{API_URL}/servers/ensure-running?{selector}")
}

/// Replicated from `AmbientFbSchema::DbPackageContent`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PackageContent {
    Playable,
    Example,
    NotExample,
    Asset,
    Models,
    Animations,
    Textures,
    Materials,
    Fonts,
    Code,
    Schema,
    Audio,
    Other,
    Tool,
    Mod,
}

#[derive(Default, Copy, Clone)]
pub struct PackageListParams<'a> {
    /// ISO8601 date string
    pub created_after: Option<&'a str>,
    /// ISO8601 date string
    pub updated_after: Option<&'a str>,
    /// NOTE: This does not URI-encode the name (yet)
    pub name: Option<&'a str>,
    pub content_contains: &'a [PackageContent],
    pub owner_id: Option<&'a str>,
    pub for_playable: Option<&'a str>,
}

/// Endpoint to get a list of packages. Filters can be supplied as necessary.
pub fn package_list_url(params: PackageListParams) -> String {
    let url = format!("{API_URL}/packages/list");

    let mut segments = vec![];
    if let Some(created_after) = params.created_after {
        segments.push(format!("created_after={}", created_after));
    }

    if let Some(updated_after) = params.updated_after {
        segments.push(format!("updated_after={}", updated_after));
    }

    if let Some(name) = params.name {
        segments.push(format!("name={}", name));
    }

    if !params.content_contains.is_empty() {
        segments.push(format!(
            "content_contains={}",
            params
                .content_contains
                .iter()
                .map(|content| format!("{:?}", content))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }

    if let Some(owner_id) = params.owner_id {
        segments.push(format!("owner_id={}", owner_id));
    }

    if let Some(for_playable) = params.for_playable {
        segments.push(format!("for_playable={}", for_playable));
    }

    if segments.is_empty() {
        url
    } else {
        format!("{url}?{}", segments.join("&"))
    }
}
