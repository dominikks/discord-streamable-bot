use chrono::Local;
use futures::stream::TryStreamExt;
use lazy_static::lazy_static;
use reqwest::Url;
use sanitize_filename::sanitize;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{debug, instrument};

lazy_static! {
    pub static ref DOWNLOADS_FOLDER: &'static Path = Path::new("./clips");
}

#[derive(Debug, Deserialize)]
struct StreamableResponse {
    title: String,
    files: HashMap<String, StreamableFile>,
}

#[derive(Debug, Deserialize)]
struct StreamableFile {
    url: Option<String>,
}

#[derive(Debug)]
pub enum DownloadError {
    Parse(),
    Fetch(#[allow(dead_code)] reqwest::Error),
    Api(),
    Filesystem(#[allow(dead_code)] std::io::Error),
}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        DownloadError::Fetch(e)
    }
}
impl From<std::io::Error> for DownloadError {
    fn from(e: std::io::Error) -> Self {
        DownloadError::Filesystem(e)
    }
}

#[instrument]
pub async fn download_clip(shortcode: &str, filename_prefix: &str) -> Result<(), DownloadError> {
    download_clip_internal(
        shortcode,
        filename_prefix,
        "https://api.streamable.com/videos/",
        &DOWNLOADS_FOLDER,
    )
    .await
}

#[instrument]
async fn download_clip_internal(
    shortcode: &str,
    filename_prefix: &str,
    base_url: &str,
    download_folder: &Path,
) -> Result<(), DownloadError> {
    let url = Url::parse(base_url)
        .and_then(|url| url.join(shortcode))
        .map_err(|_e| DownloadError::Parse())?;
    let res: StreamableResponse = reqwest::get(url).await?.error_for_status()?.json().await?;

    let title = res.title;
    let url = res
        .files
        .get("mp4")
        .and_then(|f| f.url.clone())
        .ok_or(DownloadError::Api())?;
    debug!(?url, ?title, "Found clip metadata");

    // https://github.com/seanmonstar/reqwest/issues/482#issuecomment-586508535
    let mut res = reqwest::get(&url)
        .await?
        .error_for_status()?
        .bytes_stream()
        .map_err(futures::io::Error::other)
        .into_async_read()
        .compat();
    let mut out = File::create(download_folder.join(sanitize(format!(
        "{} {} - {}.mp4",
        Local::now().format("%F %R"),
        filename_prefix,
        title,
    ))))
    .await?;
    io::copy(&mut res, &mut out).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_download_clip_full_flow() {
        // Setup mock HTTP server
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();

        // Mock the API endpoint that returns video metadata
        Mock::given(method("GET"))
            .and(path("/testcode"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "title": "Test Video",
                "files": {
                    "mp4": {
                        "url": format!("{}/video.mp4", mock_server.uri())
                    }
                }
            })))
            .mount(&mock_server)
            .await;

        // Mock the video file download endpoint
        let test_content = b"fake video content";
        Mock::given(method("GET"))
            .and(path("/video.mp4"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(test_content))
            .mount(&mock_server)
            .await;

        // Test the download
        let result = download_clip_internal(
            "testcode",
            "testuser",
            &format!("{}/", mock_server.uri()),
            temp_dir.path(),
        )
        .await;

        assert!(result.is_ok());

        // Verify file was created
        let files: Vec<_> = std::fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        assert_eq!(files.len(), 1);
        let file_path = files[0].path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        // Check file name contains expected parts
        assert!(file_name.contains("testuser"));
        assert!(file_name.contains("Test Video"));
        assert!(file_name.ends_with(".mp4"));

        // Verify file content
        let content = std::fs::read(&file_path).unwrap();
        assert_eq!(content, test_content);
    }

    #[tokio::test]
    async fn test_download_clip_api_error() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();

        // Mock API returning error
        Mock::given(method("GET"))
            .and(path("/badcode"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let result = download_clip_internal(
            "badcode",
            "testuser",
            &format!("{}/", mock_server.uri()),
            temp_dir.path(),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_download_clip_missing_mp4() {
        let mock_server = MockServer::start().await;
        let temp_dir = TempDir::new().unwrap();

        // Mock API returning response without mp4 file
        Mock::given(method("GET"))
            .and(path("/nomp4"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "title": "Test Video",
                "files": {}
            })))
            .mount(&mock_server)
            .await;

        let result = download_clip_internal(
            "nomp4",
            "testuser",
            &format!("{}/", mock_server.uri()),
            temp_dir.path(),
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DownloadError::Api()));
    }
}
