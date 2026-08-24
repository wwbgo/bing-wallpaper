use std::fs::{self, File};
use std::io::{self, Read};
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use windows::Win32::Storage::FileSystem::{
    MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
};
use windows::core::PCWSTR;

use crate::cli::Resolution;

const MAX_IMAGE_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct BingResponse {
    images: Vec<BingImage>,
}

#[derive(Debug, Clone)]
pub struct BingImage {
    pub startdate: String,
    pub urlbase: String,
    pub copyright: String,
}

impl<'de> Deserialize<'de> for BingImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawImage {
            startdate: String,
            urlbase: String,
            copyright: String,
        }

        let raw = RawImage::deserialize(deserializer)?;
        Ok(Self {
            startdate: raw.startdate,
            urlbase: raw.urlbase,
            copyright: raw.copyright,
        })
    }
}

pub fn fetch_today(host: &str, mkt: &str) -> Result<BingImage> {
    let host = host.trim_end_matches('/');
    let url = format!("{host}/HPImageArchive.aspx");

    let agent = http_agent();
    let request = agent
        .get(&url)
        .query("format", "js")
        .query("idx", "0")
        .query("n", "1")
        .query("mkt", mkt);
    let mut response = request
        .call()
        .with_context(|| format!("failed to request Bing metadata from {url}"))?;
    let body = response
        .body_mut()
        .read_to_string()
        .with_context(|| format!("failed to read Bing metadata from {url}"))?;
    let parsed: BingResponse =
        serde_json::from_str(&body).with_context(|| "failed to parse Bing metadata")?;

    parsed
        .images
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("Bing metadata response did not contain any images"))
}

pub fn download_image(
    host: &str,
    image: &BingImage,
    resolution: Resolution,
    destination: &Path,
) -> Result<()> {
    let tmp = destination.with_extension("jpg.part");
    let agent = http_agent();
    let mut last_error = None;

    for url in candidate_image_urls(host, image, resolution) {
        let result = (|| -> Result<()> {
            let mut response = agent
                .get(&url)
                .call()
                .with_context(|| format!("failed to request image from {url}"))?;
            let mut reader = response.body_mut().as_reader().take(MAX_IMAGE_BYTES + 1);
            let mut file = File::create(&tmp)
                .with_context(|| format!("failed to create temporary file {}", tmp.display()))?;
            let copied = io::copy(&mut reader, &mut file)
                .with_context(|| format!("failed to download image to {}", tmp.display()))?;
            if copied > MAX_IMAGE_BYTES {
                return Err(anyhow!(
                    "image is larger than the {} byte limit",
                    MAX_IMAGE_BYTES
                ));
            }
            file.sync_all()
                .with_context(|| format!("failed to flush temporary file {}", tmp.display()))?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                atomic_replace(&tmp, destination)?;
                return Ok(());
            }
            Err(error) => {
                last_error = Some(error);
                let _ = fs::remove_file(&tmp);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow!("no image download candidate was available")))
}

fn candidate_image_urls(host: &str, image: &BingImage, resolution: Resolution) -> Vec<String> {
    let host = host.trim_end_matches('/');
    let base = &image.urlbase;

    match resolution {
        Resolution::Uhd => vec![
            format!("{host}{base}_UHD.jpg"),
            format!("{host}{base}_1920x1080.jpg"),
        ],
        Resolution::Fhd => vec![
            format!("{host}{base}_1920x1080.jpg"),
            format!("{host}{base}_UHD.jpg"),
        ],
    }
}

fn atomic_replace(tmp: &Path, destination: &Path) -> Result<()> {
    let tmp_wide = wide_path(tmp);
    let destination_wide = wide_path(destination);

    unsafe {
        MoveFileExW(
            PCWSTR(tmp_wide.as_ptr()),
            PCWSTR(destination_wide.as_ptr()),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
        .with_context(|| {
            format!(
                "failed to replace image {} with {}",
                destination.display(),
                tmp.display()
            )
        })?;
    }

    Ok(())
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect()
}

fn http_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .user_agent(concat!("bing-wallpaper/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fhd_prefers_1080_then_uhd() {
        let image = BingImage {
            startdate: "20260823".to_string(),
            urlbase: "/th?id=test".to_string(),
            copyright: "test".to_string(),
        };

        let urls = candidate_image_urls("https://www.bing.com/", &image, Resolution::Fhd);
        assert_eq!(urls[0], "https://www.bing.com/th?id=test_1920x1080.jpg");
        assert_eq!(urls[1], "https://www.bing.com/th?id=test_UHD.jpg");
    }
}
