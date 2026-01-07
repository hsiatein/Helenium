use image::DynamicImage;

#[derive(Clone)]
pub enum HelenyFile {
    Text(String),
    Image(DynamicImage),
}

#[derive(Clone)]
pub enum HelenyFileType {
    Text,
    Image,
    Unknown
}