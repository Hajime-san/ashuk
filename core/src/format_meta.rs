use std::ffi::OsStr;
use std::path::Path;
use thiserror::Error;

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

    pub fn can_read(&self) -> bool {
        match self {
            ImageFormat::Png => true,
            ImageFormat::Jpeg => true,
            ImageFormat::WebP => true,
        }
    }

    pub fn can_write(&self) -> bool {
        match self {
            ImageFormat::Png => true,
            ImageFormat::Jpeg => true,
            ImageFormat::WebP => true,
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
}
