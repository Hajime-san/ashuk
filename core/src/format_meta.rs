use image;

pub struct ImageFormat(image::ImageFormat);

pub enum ProcessStrategy {
    Serial,
    Parallel
}

impl ImageFormat {
    pub fn can_read(format: &image::ImageFormat) -> bool {
        match format {
            image::ImageFormat::Png => true,
            image::ImageFormat::Jpeg => true,
            image::ImageFormat::WebP => true,
            _ => false,
        }
    }
    // override 'image' crate
    pub fn can_write(format: &image::ImageFormat) -> bool {
        match format {
            image::ImageFormat::Jpeg => true,
            image::ImageFormat::WebP => true,
            _ => false,
        }
    }
    pub fn process_strategy(format: &image::ImageFormat) -> ProcessStrategy {
        match format {
            image::ImageFormat::Png => ProcessStrategy::Serial,
            image::ImageFormat::Jpeg => ProcessStrategy::Parallel,
            image::ImageFormat::WebP => ProcessStrategy::Parallel,
            _ => unreachable!("unsupported format"),
        }
    }
}

pub fn get_formats() -> Vec<image::ImageFormat> {
    let jpg = image::ImageFormat::Jpeg;
    let png = image::ImageFormat::Png;
    let webp = image::ImageFormat::WebP;

    let formats = vec![
        jpg, png, webp,
    ];

    formats
}

pub fn get_format_from_path(path: &str) -> image::ImageResult<image::ImageFormat> {
    image::ImageFormat::from_path(path)
}
