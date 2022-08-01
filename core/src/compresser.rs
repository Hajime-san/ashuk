use image::io::Reader as ImageReader;
use image::ImageError;

use mozjpeg::{ColorSpace, Compress, ScanMode};
use oxipng;
use webp;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::borrow;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path;
use std::result;
use std::time::Instant;

use crate::format_meta::ImageFormat;

#[derive(Error, Debug)]
pub enum CompressError {
    #[error("This operation from {0} to {1} is not supported")]
    Unsupported(ImageFormat, ImageFormat),
    #[error(transparent)]
    PngError(#[from] oxipng::PngError),
    #[error("file io error: {0}")]
    Disconnect(#[from] io::Error),
    #[error("unknown error: {0}")]
    Unknown(#[from] ImageError),
}

fn set_file_to_same_dir(file_path: &str, extension: &str) -> String {
    let path = path::Path::new(file_path);

    let ext = ".".to_string() + extension;

    let output_file_path =
        // dirname
        path.parent().unwrap().to_string_lossy()
        // add separator
        + borrow::Cow::from(path::MAIN_SEPARATOR.to_string())
        // add filename without extension
        + path.file_stem().unwrap().to_string_lossy()
        // add extension
        + path::Path::new(&ext).to_string_lossy();

    output_file_path.to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Status {
    Initialized,
    Pending,
    Success,
    Failed,
    Unsupported,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Result {
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

pub fn compress_to_target_extension(
    file_path: &str,
    options: CompressOptions,
) -> result::Result<Result, CompressError> {
    let start = Instant::now();

    let options = options.clone();

    let input_extension = ImageFormat::from_path(&file_path).unwrap();

    let output_extension = ImageFormat::from_extension(&options.clone().extension).unwrap();

    if let false = input_extension.can_compress(&output_extension) {
        return Err(CompressError::Unsupported(
            input_extension,
            output_extension,
        ));
    };

    let confirmed_extension = if input_extension == output_extension {
        // overwrite
        path::Path::new(file_path)
            .extension()
            .and_then(OsStr::to_str)
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

            fs::write(&output_file_path, &*contents)?;

            let end = start.elapsed();

            Result {
                size: fs::metadata(&output_file_path)?.len(),
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

            fs::write(&output_file_path, contents)?;

            let end = start.elapsed();

            Result {
                size: fs::metadata(&output_file_path)?.len(),
                path: output_file_path.clone(),
                elapsed: end.as_millis() as u64,
                extension: confirmed_extension.to_string(),
            }
        }
        ImageFormat::Png => {
            // don't use multi process outside this function, because of oxipng process image with multithreading
            let output_file_path = set_file_to_same_dir(&file_path, confirmed_extension);

            let input = path::PathBuf::from(&file_path);
            let output = path::PathBuf::from(&output_file_path);

            oxipng::optimize(
                &oxipng::InFile::Path(input),
                &oxipng::OutFile::Path(Some(output)),
                &oxipng::Options::from_preset(options.clone().quality.unwrap_or(6.0) as u8),
            )?;

            let end = start.elapsed();

            Result {
                size: fs::metadata(&output_file_path)?.len(),
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
