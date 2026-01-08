#[derive(Clone)]
pub enum HelenyFile {
    Text(String),
    Image(Vec<u8>),
}

#[derive(Clone)]
pub enum HelenyFileType {
    Text,
    Image,
    Unknown,
}
