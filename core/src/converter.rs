use image::io::Reader as ImageReader;
use image::ImageError;

use mozjpeg::{ColorSpace, Compress, ScanMode};
use oxipng;
use webp;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::time::Instant;

use crate::format_meta::ImageFormat;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("This operation from {0} to {1} is not supported")]
    Unsupported(String, String),
    #[error(transparent)]
    PngError(#[from] oxipng::PngError),
    #[error("file io error: {0}")]
    Disconnect(#[from] std::io::Error),
    #[error("unknown error: {0}")]
    Unknown(#[from] ImageError),
}

fn set_file_to_same_dir(file_path: &str, extension: &str) -> String {
    let path = std::path::Path::new(file_path);

    let ext = ".".to_string() + extension;

    let output_file_path =
        // dirname
        path.parent().unwrap().to_string_lossy()
        // add separator
        + std::borrow::Cow::from(std::path::MAIN_SEPARATOR.to_string())
        // add filename without extension
        + path.file_stem().unwrap().to_string_lossy()
        // add extension
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
    pub extension: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompressOptions {
    pub quality: Option<f32>,
    pub extension: String,
}

pub fn covert_to_target_extension(
    file_path: &str,
    options: CompressOptions,
) -> Result<CovertResult, ConvertError> {
    let start = Instant::now();

    let options = options.clone();

    let input_extension = ImageFormat::from_path(&file_path).unwrap();

    let output_extension = ImageFormat::from_extension(&options.clone().extension).unwrap();

    let confirmed_extension = if input_extension == output_extension {
        // overwrite
        std::path::Path::new(file_path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap()
    } else {
        // alter extension
        &options.extension
    };

    let result = match output_extension {
        ImageFormat::WebP => {
            let output_file_path = set_file_to_same_dir(&file_path, confirmed_extension);

            let decoded = ImageReader::open(file_path)?.decode()?;

            let encoder = webp::Encoder::from_image(&decoded).unwrap();

            let contents = encoder.encode(options.clone().quality.unwrap_or(75.0));

            std::fs::write(&output_file_path, &*contents)?;

            let end = start.elapsed();

            CovertResult {
                size: std::fs::metadata(&output_file_path)?.len(),
                path: output_file_path.clone(),
                elapsed: end.as_millis() as u64,
                extension: confirmed_extension.to_string(),
            }
        }
        ImageFormat::Jpeg => {
            let output_file_path = set_file_to_same_dir(&file_path, confirmed_extension);

            let decoded = ImageReader::open(file_path)?.decode()?;

            let mut comp = Compress::new(ColorSpace::JCS_RGB);
            let width = decoded.width() as usize;
            let height = decoded.height() as usize;
            comp.set_scan_optimization_mode(ScanMode::AllComponentsTogether);
            comp.set_quality(options.clone().quality.unwrap_or(75.0));

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
                extension: confirmed_extension.to_string(),
            }
        }
        ImageFormat::Png => {
            // don't use multi process outside this function, because of oxipng process image with multithreading
            let output_file_path = set_file_to_same_dir(&file_path, confirmed_extension);

            let input = std::path::PathBuf::from(&file_path);
            let output = std::path::PathBuf::from(&output_file_path);

            oxipng::optimize(
                &oxipng::InFile::Path(input),
                &oxipng::OutFile::Path(Some(output)),
                &oxipng::Options::from_preset(options.clone().quality.unwrap_or(6.0) as u8),
            )?;

            let end = start.elapsed();

            CovertResult {
                size: std::fs::metadata(&output_file_path)?.len(),
                path: output_file_path.clone(),
                elapsed: end.as_millis() as u64,
                extension: confirmed_extension.to_string(),
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
