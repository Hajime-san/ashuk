use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageError};
use webp::{Encoder, WebPMemory};
use serde::{ Serialize, Deserialize };

use thiserror::Error;

use std::time::{Duration, Instant};

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("file io error")]
    Disconnect(#[from] std::io::Error),
    #[error("unknown image relative error")]
    Unknown(#[from] ImageError),
}

fn set_file_to_same_dir(file_path: &str, extention: &str) -> String {
    let path = std::path::Path::new(file_path);

    let ext = ".".to_string() + extention;

    let output_file_path =
        // dirname
        path.parent().unwrap().to_string_lossy()
        // add separator
        + std::borrow::Cow::from(std::path::MAIN_SEPARATOR.to_string())
        // add filename without extention
        + path.file_stem().unwrap().to_string_lossy()
        // add extention
        + std::path::Path::new(&ext).to_string_lossy();

    output_file_path.to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConvertStatus {
    Success,
    Failed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageContext {
    pub size: u64,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElapsedTime {
    pub sec: u64,
    pub nano_sec: u32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CovertResult {
    pub status: ConvertStatus,
    pub input: Option<ImageContext>,
    pub output: Option<ImageContext>,
    pub elapsed: Option<ElapsedTime>,
}

pub async fn covert_to_webp(file_path: &str, quality: f32) -> Result<CovertResult, ConvertError> {
    let start = Instant::now();

    let decoded: DynamicImage = ImageReader::open(file_path)?.decode()?;

    let encoder: Encoder = Encoder::from_image(&decoded).unwrap();

    let encoded: WebPMemory = encoder.encode(quality);

    let output_file_path = set_file_to_same_dir(&file_path, "webp");

    std::fs::write(&output_file_path, &*encoded)?;

    let end = start.elapsed();
    // println!("{}.{:03}s took.", end.as_secs(), end.subsec_nanos() / 1_000_000);

    Ok(CovertResult {
        status: ConvertStatus::Success,
        input: Some(ImageContext {
            size: std::fs::metadata(file_path)?.len(),
            path: Some(file_path.to_string()),
        }),
        output: Some(ImageContext {
            size: std::fs::metadata(&output_file_path)?.len(),
            path: Some(output_file_path.clone()),
        }),
        elapsed: Some(ElapsedTime {
            sec: end.as_secs(),
            nano_sec: end.subsec_nanos() / 1_000_000
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_set_file_to_same_dir() {
        let file_path = "src/assets/New-York-street-scene-with-steam-and-billboards.jpg";
        let output_file_path = set_file_to_same_dir(&file_path, "jpg");
        assert_eq!(file_path, &output_file_path);
    }
}
