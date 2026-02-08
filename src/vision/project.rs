use image::RgbImage;
use imageproc::geometric_transformations::{Projection, warp_into};
use imageproc::point::Point;

use crate::util::point::MyPoint;

pub fn unwarp_rectangle(
    img: &RgbImage,
    vertices: &[MyPoint<f32>; 4],
    output_width: u32,
    output_height: u32,
) -> Option<RgbImage> {
    let [top_left, top_right, bottom_right, bottom_left] = vertices;

    // Destination corners (straight rectangle)
    let dst = [
        (0.0, 0.0),
        (output_width as f32, 0.0),
        (output_width as f32, output_height as f32),
        (0.0, output_height as f32),
    ];

    // Source corners (distorted quadrilateral)
    let src = [
        (top_left.x, top_left.y),
        (top_right.x, top_right.y),
        (bottom_right.x, bottom_right.y),
        (bottom_left.x, bottom_left.y),
    ];

    let projection = Projection::from_control_points(src, dst)?;
    let mut output = RgbImage::new(output_width, output_height);

    warp_into(
        img,
        &projection,
        imageproc::geometric_transformations::Interpolation::Bilinear,
        image::Rgb([255u8, 255, 255]),
        &mut output,
    );

    Some(output)
}
