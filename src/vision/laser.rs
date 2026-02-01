use image::RgbImage;
use imageproc::point::Point;

pub fn find_red_laser(img: &RgbImage) -> Option<Point<f32>> {
    let (width, height) = img.dimensions();
    let mut sum_x = 0.0f32;
    let mut sum_y = 0.0f32;
    let mut count = 0u32;

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            // Detect bright red pixels (laser point is very bright)
            if r > 245.0 {
                sum_x += x as f32;
                sum_y += y as f32;
                count += 1;
            }
        }
    }

    if count > 10 {
        Some(Point::new(sum_x / count as f32, sum_y / count as f32))
    } else {
        None
    }
}
