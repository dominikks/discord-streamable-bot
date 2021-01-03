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
  width: u32,
  height: u32,
  duration: f32,
}

#[derive(Debug)]
pub enum DownloadError {
  ParseError(),
  FetchError(reqwest::Error),
  ApiError(),
  FilesystemError(std::io::Error),
}

impl From<reqwest::Error> for DownloadError {
  fn from(e: reqwest::Error) -> Self {
    DownloadError::FetchError(e)
  }
}
impl From<std::io::Error> for DownloadError {
  fn from(e: std::io::Error) -> Self {
    DownloadError::FilesystemError(e)
  }
}

#[instrument]
pub async fn download_clip(shortcode: &str, filename_prefix: &str) -> Result<(), DownloadError> {
  let url = Url::parse("https://api.streamable.com/videos/")
    .and_then(|url| url.join(shortcode))
    .map_err(|_e| DownloadError::ParseError())?;
  let res: StreamableResponse = reqwest::get(url).await?.error_for_status()?.json().await?;

  let title = res.title;
  let url = res
    .files
    .get("mp4")
    .and_then(|f| f.url.clone())
    .ok_or(DownloadError::ApiError())?;
  debug!(?url, ?title, "Found clip metadata");

  // https://github.com/seanmonstar/reqwest/issues/482#issuecomment-586508535
  let mut res = reqwest::get(&url)
    .await?
    .error_for_status()?
    .bytes_stream()
    .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
    .into_async_read()
    .compat();
  let mut out = File::create(DOWNLOADS_FOLDER.join(sanitize(format!(
    "{} {} - {}.mp4",
    Local::now().format("%F %R"),
    filename_prefix,
    title,
  ))))
  .await?;
  io::copy(&mut res, &mut out).await?;

  Ok(())
}
