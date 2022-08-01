use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::ffi::OsStr;
use std::path::Path;

pub enum ProcessStrategy {
    Serial,
    Parallel,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
}

#[derive(Error, Debug)]
pub enum ImageFormatError {
    #[error("file is not supported")]
    Unsupported,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompressOptionsContext {
    pub extension: String,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub step: f32,
}

impl ImageFormat {
    pub fn from_extension<S>(ext: S) -> Option<Self>
    where
        S: AsRef<OsStr>,
    {
        fn inner(ext: &OsStr) -> Option<ImageFormat> {
            let ext = ext.to_str()?.to_ascii_lowercase();

            Some(match ext.as_str() {
                "jpg" | "jpeg" => ImageFormat::Jpeg,
                "png" => ImageFormat::Png,
                "webp" => ImageFormat::WebP,
                _ => return None,
            })
        }

        inner(ext.as_ref())
    }

    pub fn from_path<P>(path: P) -> Result<Self, ImageFormatError>
    where
        P: AsRef<Path>,
    {
        fn inner(path: &Path) -> Result<ImageFormat, ImageFormatError> {
            let exact_ext = path.extension();
            exact_ext
                .and_then(ImageFormat::from_extension)
                .ok_or_else(|| ImageFormatError::Unsupported)
        }

        inner(path.as_ref())
    }

    pub fn extensions_str(self) -> &'static [&'static str] {
        match self {
            ImageFormat::Png => &["png"],
            ImageFormat::Jpeg => &["jpg", "jpeg"],
            ImageFormat::WebP => &["webp"],
        }
    }

    pub fn get_representative_ext_str(&self) -> String {
        match self {
            ImageFormat::Png => <str as ToString>::to_string(&*self.extensions_str()[0]),
            ImageFormat::Jpeg => <str as ToString>::to_string(&*self.extensions_str()[0]),
            ImageFormat::WebP => <str as ToString>::to_string(&*self.extensions_str()[0]),
        }
    }

    pub fn process_strategy(&self) -> ProcessStrategy {
        match self {
            ImageFormat::Png => ProcessStrategy::Serial,
            ImageFormat::Jpeg => ProcessStrategy::Parallel,
            ImageFormat::WebP => ProcessStrategy::Parallel,
        }
    }

    pub fn get_formats() -> Vec<ImageFormat> {
        let jpg = ImageFormat::Jpeg;
        let png = ImageFormat::Png;
        let webp = ImageFormat::WebP;

        let formats = vec![jpg, png, webp];

        formats
    }

    pub fn get_compress_options_context(&self) -> CompressOptionsContext {
        match self {
            ImageFormat::Png => CompressOptionsContext {
                extension: self.get_representative_ext_str(),
                min: 0.0,
                max: 6.0,
                default: 6.0,
                step: 1.0,
            },
            ImageFormat::Jpeg => CompressOptionsContext {
                extension: self.get_representative_ext_str(),
                min: 0.0,
                max: 100.0,
                default: 75.0,
                step: 0.1,
            },
            ImageFormat::WebP => CompressOptionsContext {
                extension: self.get_representative_ext_str(),
                min: 0.0,
                max: 100.0,
                default: 75.0,
                step: 0.1,
            },
        }
    }
}
