use std::fs;
use std::path::{Path, PathBuf};

use slug::slugify;
use url::Url;

use crate::error::AppError;

pub fn derive_filename(url: &Url, filename_override: Option<&str>) -> String {
    let base_name =
        filename_override.map_or_else(|| default_name_from_url(url), sanitize_override_name);
    with_md_extension(base_name)
}

pub fn save_markdown(
    out_dir: &Path,
    filename: &str,
    markdown: &str,
    overwrite: bool,
) -> Result<PathBuf, AppError> {
    fs::create_dir_all(out_dir)?;

    let output_path = out_dir.join(filename);
    if output_path.exists() && !overwrite {
        return Err(AppError::FileExists(output_path));
    }

    fs::write(&output_path, markdown)?;
    Ok(output_path)
}

fn default_name_from_url(url: &Url) -> String {
    let base = url
        .path_segments()
        .and_then(|mut segments| segments.rfind(|segment| !segment.trim().is_empty()))
        .or_else(|| url.host_str())
        .unwrap_or("output");

    let slug = slugify(base);
    if slug.is_empty() {
        "output".to_string()
    } else {
        slug
    }
}

fn sanitize_override_name(name: &str) -> String {
    Path::new(name)
        .file_name()
        .and_then(|file| file.to_str())
        .map(str::trim)
        .filter(|file| !file.is_empty())
        .unwrap_or("output")
        .to_string()
}

fn with_md_extension(name: String) -> String {
    if name.to_ascii_lowercase().ends_with(".md") {
        name
    } else {
        format!("{name}.md")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;
    use url::Url;

    use crate::error::AppError;

    use super::{derive_filename, save_markdown};

    #[test]
    fn derive_filename_from_url_path() {
        let url = Url::parse("https://example.com/articles/hello-world").expect("url parse failed");
        let filename = derive_filename(&url, None);

        assert_eq!(filename, "hello-world.md");
    }

    #[test]
    fn derive_filename_uses_host_when_path_is_empty() {
        let url = Url::parse("https://example.com/").expect("url parse failed");
        let filename = derive_filename(&url, None);

        assert_eq!(filename, "example-com.md");
    }

    #[test]
    fn derive_filename_from_override_and_append_extension() {
        let url = Url::parse("https://example.com/ignored").expect("url parse failed");
        let filename = derive_filename(&url, Some("report"));

        assert_eq!(filename, "report.md");
    }

    #[test]
    fn save_markdown_creates_directory_and_writes_file() {
        let temp = tempdir().expect("tempdir failed");
        let out_dir = temp.path().join("nested").join("output");

        let path = save_markdown(&out_dir, "sample.md", "# Hello", false).expect("save failed");

        assert!(path.exists());
        let written = fs::read_to_string(path).expect("read failed");
        assert_eq!(written, "# Hello");
    }

    #[test]
    fn save_markdown_fails_when_file_exists_without_overwrite() {
        let temp = tempdir().expect("tempdir failed");
        let out_dir = temp.path();
        let file = out_dir.join("sample.md");
        fs::write(&file, "old").expect("seed write failed");

        let error = save_markdown(out_dir, "sample.md", "new", false).expect_err("should fail");

        match error {
            AppError::FileExists(path) => assert_eq!(path, file),
            _ => panic!("unexpected error: {error:?}"),
        }
    }

    #[test]
    fn save_markdown_overwrites_when_flag_is_true() {
        let temp = tempdir().expect("tempdir failed");
        let out_dir = temp.path();
        let file = out_dir.join("sample.md");
        fs::write(&file, "old").expect("seed write failed");

        let path = save_markdown(out_dir, "sample.md", "new", true).expect("save failed");

        assert_eq!(path, file);
        let written = fs::read_to_string(path).expect("read failed");
        assert_eq!(written, "new");
    }
}
