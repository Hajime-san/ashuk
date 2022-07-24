use image;

pub struct ImageFormat(image::ImageFormat);

impl ImageFormat {
    pub fn can_read(format: &image::ImageFormat) -> bool {
        match format {
            image::ImageFormat::Png => true,
            image::ImageFormat::Gif => true,
            image::ImageFormat::Jpeg => true,
            image::ImageFormat::WebP => true,
            image::ImageFormat::Tiff => true,
            image::ImageFormat::Tga => true,
            image::ImageFormat::Dds => false,
            image::ImageFormat::Bmp => true,
            image::ImageFormat::Ico => true,
            image::ImageFormat::Hdr => true,
            image::ImageFormat::OpenExr => true,
            image::ImageFormat::Pnm => true,
            image::ImageFormat::Farbfeld => true,
            image::ImageFormat::Avif => true,
            _ => false,
        }
    }
    // override 'image' crate
    pub fn can_write(format: &image::ImageFormat) -> bool {
        match format {
            image::ImageFormat::Gif => true,
            image::ImageFormat::Ico => true,
            image::ImageFormat::Jpeg => true,
            image::ImageFormat::Png => true,
            image::ImageFormat::Bmp => true,
            image::ImageFormat::Tiff => true,
            image::ImageFormat::Tga => true,
            image::ImageFormat::Pnm => true,
            image::ImageFormat::Farbfeld => true,
            image::ImageFormat::Avif => true,
            // add support
            image::ImageFormat::WebP => true,
            image::ImageFormat::Hdr => false,
            image::ImageFormat::OpenExr => true,
            image::ImageFormat::Dds => false,
            _ => false,
        }
    }
}

pub fn get_formats() -> Vec<image::ImageFormat> {
    let avif = image::ImageFormat::Avif;
    let bmp = image::ImageFormat::Bmp;
    let farbfeld = image::ImageFormat::Farbfeld;
    // unsupported read, write
    let dds = image::ImageFormat::Dds;
    let gif = image::ImageFormat::Gif;
    // unsupported write
    let hdr = image::ImageFormat::Hdr;
    let ico = image::ImageFormat::Ico;
    let jpg = image::ImageFormat::Jpeg;
    let exr = image::ImageFormat::OpenExr;
    let png = image::ImageFormat::Png;
    let pnm = image::ImageFormat::Pnm;
    let tga = image::ImageFormat::Tga;
    let tiff = image::ImageFormat::Tiff;
    let webp = image::ImageFormat::WebP;

    let formats = vec![
        avif, bmp, farbfeld, dds, gif, hdr, ico, jpg, exr, png, pnm, tga, tiff, webp,
    ];

    formats
}

pub fn get_format_from_path(path: &str) -> image::ImageResult<image::ImageFormat> {
    image::ImageFormat::from_path(path)
}
