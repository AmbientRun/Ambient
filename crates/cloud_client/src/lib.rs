use anyhow::{anyhow, Context};

use ambient_asset_cache::{AssetCache, SyncAssetKeyExt};
use ambient_native_std::download_asset::ReqwestClientKey;
use ambient_sys::task::wasm_nonsend;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum DeploySource {
    /// Deploy directly from an URL
    Url { deploy_url: String },
    /// Deploy the latest deployment of a package
    Package { package_id: String },
    /// Deploy from a specific deployment
    Deployment { deployment_id: String },
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ServerSpec {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub context: String,
    #[serde(flatten)]
    pub source: DeploySource,
}

impl ServerSpec {
    pub fn new_with_deployment(deployment_id: String) -> Self {
        Self {
            name: Default::default(),
            context: Default::default(),
            source: DeploySource::Deployment { deployment_id },
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = context;
        self
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Server {
    pub name: String,
    pub context: String,
    pub deploy_url: String,
    pub host: String,
}

/// Ensure that a server is running, and return its host and port.
pub async fn ensure_server_running(
    assets: &AssetCache,
    api_server: &str,
    auth_token: String,
    spec: ServerSpec,
) -> anyhow::Result<Server> {
    request(
        assets,
        reqwest::Method::POST,
        format!("{}/servers/ensure-running", api_server),
        auth_token,
        spec,
    )
    .await
}

/// List all servers running for a given deployment URL.
pub async fn list_servers(
    assets: &AssetCache,
    api_server: &str,
    auth_token: String,
    deploy_url: &str,
) -> anyhow::Result<Vec<Server>> {
    request(
        assets,
        reqwest::Method::GET,
        format!("{}/servers/list", api_server),
        auth_token,
        [(String::from("deploy_url"), deploy_url.to_owned())],
    )
    .await
}

pub(crate) async fn request<Req, Resp>(
    assets: &AssetCache,
    method: reqwest::Method,
    url: impl reqwest::IntoUrl,
    auth_token: String,
    req: Req,
) -> anyhow::Result<Resp>
where
    Req: serde::Serialize + 'static + Send,
    Resp: serde::de::DeserializeOwned + 'static + Send,
{
    let url_str = url.as_str().to_string();
    let url = url.into_url()?;
    let assets = assets.clone();

    // reqwest::Client is not Send on wasm
    wasm_nonsend(move || async move {
        let client = ReqwestClientKey.get(&assets);

        log::debug!("Requesting {}", url_str);
        let mut builder = client
            .request(method.clone(), url.clone())
            .bearer_auth(auth_token);

        match method {
            reqwest::Method::GET => {
                builder = builder.query(&req);
            }
            reqwest::Method::POST => {
                builder = builder.json(&req);
            }
            _ => {}
        }

        let resp = builder
            .send()
            .await
            .with_context(|| format!("Failed to send request to \"{url_str}\""))?;

        if !resp.status().is_success() {
            log::warn!("Request for {} failed: {:?}", url_str, resp.status());
            return Err(anyhow!(
                "Requesting {url_str} failed, bad status code: {:?}",
                resp.status()
            ));
        }

        resp.json::<Resp>()
            .await
            .with_context(|| format!("Failed to fetch body of {}", url_str))
    })
    .await
}
