use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("入力URLが不正です: {0}")]
    InvalidUrl(String),

    #[error("環境変数 `{0}` が設定されていません")]
    MissingEnvVar(&'static str),

    #[error("Cloudflare APIへのHTTPリクエストに失敗しました: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Cloudflare APIレスポンスのJSON解析に失敗しました: {0}")]
    InvalidResponse(String),

    #[error("Cloudflare APIエラー (status={status:?}): {message}")]
    Api {
        status: Option<u16>,
        message: String,
    },

    #[error("出力先ファイルが既に存在します: {0}")]
    FileExists(PathBuf),

    #[error("ファイルI/Oエラー: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSONシリアライズエラー: {0}")]
    Json(#[from] serde_json::Error),
}
