use std::path::PathBuf;

use clap::{ArgAction, Parser};
use serde::Serialize;

pub const CSS_REJECT_REQUEST_PATTERN: &str = "/^.*\\.(css)/";

const HELP_TEMPLATE: &str = "\
[NAME]
{name} {version}

[SUMMARY]
{about}

[USAGE]
{usage}

[ARGUMENTS]
{positionals}

[OPTIONS]
{options}

[ENVIRONMENT]
  CF_ACCOUNT_ID  Cloudflare アカウント ID（必須）
  CF_API_TOKEN   Cloudflare API トークン（必須）

[MODES]
  標準出力モード: --out-dir 未指定時に Markdown を標準出力へ表示
  保存モード:     --out-dir 指定時に Markdown ファイルを保存

[EXIT_CODES]
  0  成功
  1  入力/設定/API/ファイル出力のエラー

[EXAMPLES]
  page2md https://example.com
  page2md https://example.com --out-dir ./out
  page2md https://example.com --out-dir ./out --filename example.md
  page2md https://example.com --out-dir ./out --overwrite
  page2md --help-json
";

#[derive(Debug, Clone, Parser)]
#[command(
    name = "page2md",
    version,
    about = "Cloudflare Markdown endpoint を使ってページを Markdown に変換します。",
    help_template = HELP_TEMPLATE
)]
pub struct Cli {
    #[arg(
        value_name = "URL",
        required_unless_present = "help_json",
        help = "変換対象ページのURL"
    )]
    pub url: Option<String>,

    #[arg(long, value_name = "DIR", help = "Markdownを保存する出力ディレクトリ")]
    pub out_dir: Option<PathBuf>,

    #[arg(
        long,
        value_name = "NAME",
        help = "保存ファイル名を指定（未指定時はURL由来の名前）"
    )]
    pub filename: Option<String>,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "既存ファイルがある場合に上書きする"
    )]
    pub overwrite: bool,

    #[arg(
        long,
        action = ArgAction::SetTrue,
        help = "CLI仕様を機械可読JSONで出力（API呼び出しなし）"
    )]
    pub help_json: bool,
}

#[derive(Debug, Serialize)]
pub struct HelpJson {
    pub name: String,
    pub version: String,
    pub summary: String,
    pub usage: String,
    pub arguments: Vec<HelpItem>,
    pub options: Vec<HelpItem>,
    pub environment: Vec<HelpItem>,
    pub modes: Vec<HelpItem>,
    pub exit_codes: Vec<ExitCodeItem>,
    pub examples: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HelpItem {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct ExitCodeItem {
    pub code: i32,
    pub description: String,
}

fn string(value: &str) -> String {
    value.to_owned()
}

fn help_item(name: &str, description: &str) -> HelpItem {
    HelpItem {
        name: string(name),
        description: string(description),
    }
}

pub fn build_help_json() -> HelpJson {
    HelpJson {
        name: string("page2md"),
        version: string(env!("CARGO_PKG_VERSION")),
        summary: string("Cloudflare Markdown endpoint を使ってページを Markdown に変換します。"),
        usage: string("page2md <URL> [--out-dir <DIR>] [--filename <NAME>] [--overwrite]"),
        arguments: vec![help_item("URL", "変換対象ページのURL")],
        options: vec![
            help_item("-h, --help", "ヘルプを表示"),
            help_item(
                "--help-json",
                "CLI仕様を機械可読JSONで出力（API呼び出しなし）",
            ),
            help_item(
                "--out-dir <DIR>",
                "保存モードを有効にし、出力先ディレクトリを指定",
            ),
            help_item("--filename <NAME>", "保存モード時のファイル名を指定"),
            help_item("--overwrite", "同名ファイルが存在する場合に上書き"),
        ],
        environment: vec![
            help_item("CF_ACCOUNT_ID", "Cloudflare アカウント ID（必須）"),
            help_item("CF_API_TOKEN", "Cloudflare API トークン（必須）"),
        ],
        modes: vec![
            help_item(
                "標準出力モード",
                "--out-dir 未指定時に Markdown を標準出力へ表示",
            ),
            help_item("保存モード", "--out-dir 指定時に Markdown ファイルを保存"),
        ],
        exit_codes: vec![
            ExitCodeItem {
                code: 0,
                description: string("成功"),
            },
            ExitCodeItem {
                code: 1,
                description: string("入力/設定/API/ファイル出力のエラー"),
            },
        ],
        examples: vec![
            string("page2md https://example.com"),
            string("page2md https://example.com --out-dir ./out"),
            string("page2md https://example.com --out-dir ./out --filename example.md"),
            string("page2md https://example.com --out-dir ./out --overwrite"),
            string("page2md --help-json"),
        ],
    }
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::{build_help_json, Cli};

    #[test]
    fn parse_basic_arguments() {
        let cli = Cli::try_parse_from(["page2md", "https://example.com"]).expect("parse failed");

        assert_eq!(cli.url.as_deref(), Some("https://example.com"));
        assert!(cli.out_dir.is_none());
        assert!(!cli.overwrite);
    }

    #[test]
    fn parse_save_mode_arguments() {
        let cli = Cli::try_parse_from([
            "page2md",
            "https://example.com",
            "--out-dir",
            "./out",
            "--filename",
            "example.md",
            "--overwrite",
        ])
        .expect("parse failed");

        assert_eq!(cli.url.as_deref(), Some("https://example.com"));
        assert_eq!(cli.filename.as_deref(), Some("example.md"));
        assert!(cli.out_dir.is_some());
        assert!(cli.overwrite);
    }

    #[test]
    fn fail_when_url_is_missing() {
        let err = Cli::try_parse_from(["page2md"]).expect_err("should fail");

        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn allow_help_json_without_url() {
        let cli = Cli::try_parse_from(["page2md", "--help-json"]).expect("parse failed");

        assert!(cli.help_json);
        assert!(cli.url.is_none());
    }

    #[test]
    fn help_contains_required_sections() {
        let mut cmd = Cli::command();
        let mut help_bytes = Vec::new();

        cmd.write_long_help(&mut help_bytes)
            .expect("help write failed");
        let help = String::from_utf8(help_bytes).expect("utf8 conversion failed");

        for section in [
            "[NAME]",
            "[SUMMARY]",
            "[USAGE]",
            "[ARGUMENTS]",
            "[OPTIONS]",
            "[ENVIRONMENT]",
            "[MODES]",
            "[EXIT_CODES]",
            "[EXAMPLES]",
        ] {
            assert!(help.contains(section), "missing section: {section}");
        }
    }

    #[test]
    fn help_json_contains_required_keys() {
        let value = serde_json::to_value(build_help_json()).expect("to_value failed");

        for key in [
            "name",
            "version",
            "summary",
            "usage",
            "arguments",
            "options",
            "environment",
            "modes",
            "exit_codes",
            "examples",
        ] {
            assert!(value.get(key).is_some(), "missing key: {key}");
        }
    }
}
