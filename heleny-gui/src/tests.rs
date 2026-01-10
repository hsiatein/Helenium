#[test]
fn test_base64() {
    use base64::prelude::*;
    use image::DynamicImage;
    use slint::Rgba8Pixel;
    use slint::SharedPixelBuffer;
    use std::path::Path;

    let image_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(".exchange")
        .join("5D8BAF8E29A29C419FED28A964645206.png");

    let image_bytes = std::fs::read(&image_path).expect("read test image");
    let img: DynamicImage = image::open(&image_path).expect("read");

    let b64 = BASE64_STANDARD.encode(&image_bytes);
    let decoded = BASE64_STANDARD.decode(b64).expect("decode base64");
    let img2: DynamicImage = image::load_from_memory(&decoded).expect("decode image from base64");

    assert_eq!(img.width(), img2.width());
    assert_eq!(img.height(), img2.height());

    let rgba = img2.to_rgba8();
    let (w, h) = rgba.dimensions();
    let _ = slint::Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        rgba.as_raw(),
        w,
        h,
    ));
}
