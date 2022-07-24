use image::io::Reader as ImageReader;
use image::{DynamicImage, ImageError};

use mozjpeg::{ColorSpace, Compress, ScanMode};
use webp::{Encoder, WebPMemory};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::time::{Duration, Instant};

use crate::format_meta;

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ConvertStatus {
    Initialized,
    Pending,
    Success,
    Failed,
    Unsupported,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CovertResult {
    pub size: u64,
    pub path: String,
    pub elapsed: u64,
    pub extention: String,
}

pub fn covert_to_target_extention(
    file_path: &str,
    target_extention: &str,
    quality: f32,
) -> Result<CovertResult, ConvertError> {
    let start = Instant::now();

    let decoded: DynamicImage = ImageReader::open(file_path)?.decode()?;

    let input_extention = format_meta::get_format_from_path(&file_path)?;

    let output_extention = image::ImageFormat::from_extension(&target_extention).unwrap();

    let confirmed_extention = if input_extention == output_extention {
        // overwrite
        std::path::Path::new(file_path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap()
    } else {
        // alter extention
        target_extention
    };

    let result = match input_extention {
        image::ImageFormat::WebP => {
            let output_file_path = set_file_to_same_dir(&file_path, confirmed_extention);

            let encoder: Encoder = Encoder::from_image(&decoded).unwrap();

            let encoded: WebPMemory = encoder.encode(quality);

            std::fs::write(&output_file_path, &*encoded)?;

            let end = start.elapsed();

            CovertResult {
                size: std::fs::metadata(&output_file_path)?.len(),
                path: output_file_path.clone(),
                elapsed: end.as_millis() as u64,
                extention: confirmed_extention.to_string(),
            }
        }
        image::ImageFormat::Jpeg => {
            let output_file_path = set_file_to_same_dir(&file_path, confirmed_extention);

            let mut comp = Compress::new(ColorSpace::JCS_RGB);
            let width = decoded.width() as usize;
            let height = decoded.height() as usize;
            comp.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
            comp.set_quality(quality);

            comp.set_size(width, height);

            comp.set_mem_dest();
            comp.start_compress();

            let pixels = decoded.as_bytes();
            assert!(comp.write_scanlines(pixels));

            comp.finish_compress();
            let contents = comp.data_to_vec().unwrap();

            std::fs::write(&output_file_path, contents)?;

            let end = start.elapsed();

            CovertResult {
                size: std::fs::metadata(&output_file_path)?.len(),
                path: output_file_path.clone(),
                elapsed: end.as_millis() as u64,
                extention: confirmed_extention.to_string(),
            }
        }
        _ => {
            unimplemented!()
        }
    };

    Ok(result)
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
