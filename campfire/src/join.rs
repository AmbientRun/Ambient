use ambient_shared_types::urls;
use clap::Parser;

use crate::{package::RunParams, util::run_ambient};

#[derive(Parser, Clone, Debug)]
#[clap(trailing_var_arg = true)]
/// Join a server by various means
pub struct Join {
    /// The URL to join
    #[arg(long, short)]
    pub url: Option<String>,
    /// The deployment to join
    #[arg(long, short)]
    pub deployment: Option<String>,
    /// The package to join
    #[arg(long, short)]
    pub package: Option<String>,
    /// The context ID to use while joining
    #[arg(long, short)]
    pub context: Option<String>,

    #[command(flatten)]
    pub params: RunParams,
}

pub fn main(join: &Join) -> anyhow::Result<()> {
    let mut args = vec!["join"];

    let mut url = match (&join.url, &join.deployment, &join.package) {
        (Some(url), None, None) => Some(url.to_string()),
        (None, Some(deployment), None) => Some(urls::ensure_running_url(
            urls::ServerSelector::Deployment(deployment),
        )),
        (None, None, Some(package)) => Some(urls::ensure_running_url(
            urls::ServerSelector::Package(package),
        )),

        (None, None, None) => {
            tracing::info!("no join method specified, joining local server");
            None
        }
        _ => {
            anyhow::bail!("only one of `url`, `deployment`, or `package` can be specified")
        }
    };

    if let Some(url) = url.as_mut() {
        if let Some(context) = &join.context {
            if join.url.is_some() {
                anyhow::bail!("`context` cannot be specified when `url` is specified");
            }

            url.push_str(&format!("&context={context}"));
        }
        args.push(url);
    }

    if !join.params.args.is_empty() {
        args.extend(join.params.args.iter().map(|s| s.as_str()));
    }

    run_ambient(&args, join.params.release, false)
}
