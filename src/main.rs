mod cli;
mod client;
mod error;
mod output;

use clap::Parser;
use url::Url;

use crate::cli::{build_help_json, Cli};
use crate::client::MarkdownClient;
use crate::error::AppError;
use crate::output::{derive_filename, save_markdown};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    if cli.help_json {
        let json = serde_json::to_string_pretty(&build_help_json())?;
        println!("{json}");
        return Ok(());
    }

    let input_url = cli
        .url
        .as_deref()
        .ok_or_else(|| AppError::InvalidUrl("URLが指定されていません".to_string()))?;

    let parsed_url = Url::parse(input_url).map_err(|err| AppError::InvalidUrl(err.to_string()))?;

    let account_id = required_env("CF_ACCOUNT_ID")?;
    let api_token = required_env("CF_API_TOKEN")?;

    let client = MarkdownClient::new(&account_id, &api_token)?;
    let markdown = client.fetch_markdown(parsed_url.as_str()).await?;

    if let Some(out_dir) = cli.out_dir.as_deref() {
        let filename = derive_filename(&parsed_url, cli.filename.as_deref());
        let output_path = save_markdown(out_dir, &filename, &markdown, cli.overwrite)?;
        println!("{}", output_path.display());
    } else {
        println!("{markdown}");
    }

    Ok(())
}

fn required_env(key: &'static str) -> Result<String, AppError> {
    std::env::var(key).map_err(|_| AppError::MissingEnvVar(key))
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;

    use super::required_env;

    #[test]
    fn required_env_returns_error_when_variable_is_missing() {
        const KEY: &str = "PAGE2MD_TEST_MISSING_ENV";
        std::env::remove_var(KEY);

        let error = required_env(KEY).expect_err("should fail");
        assert!(matches!(
            error,
            AppError::MissingEnvVar("PAGE2MD_TEST_MISSING_ENV")
        ));
    }

    #[test]
    fn required_env_returns_value_when_variable_exists() {
        const KEY: &str = "PAGE2MD_TEST_EXISTING_ENV";
        std::env::set_var(KEY, "value");

        let value = required_env(KEY).expect("env should exist");

        assert_eq!(value, "value");
        std::env::remove_var(KEY);
    }
}
