use image::{GenericImageView, RgbImage};

pub fn crop_image(img: &RgbImage, stencil: &(f32, f32, f32, f32)) -> RgbImage {
    let (width, height) = img.dimensions();
    let (width, height) = (width as f32, height as f32);
    img.view(
        (stencil.0 * width) as u32,
        (stencil.1 * height) as u32,
        ((stencil.2 - stencil.0) * width) as u32,
        ((stencil.3 - stencil.1) * height) as u32,
    )
    .to_image()
}
