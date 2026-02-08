use imageproc::point::Point;

// use image::RgbImage;
use imageproc::contours::Contour;

use crate::util::point::MyPoint;
// use imageproc::point::Point;

pub fn find_rectangle_vertices(contours: &[Contour<u32>]) -> Option<[MyPoint<f32>; 4]> {
    // Find largest contour (should be the rectangle)
    let largest = contours.iter().max_by_key(|c| c.points.len())?;

    if largest.points.len() < 4 {
        return None;
    }

    // Find 4 corner points from contour
    let points: Vec<Point<f32>> = largest
        .points
        .iter()
        .map(|p| Point::new(p.x as f32, p.y as f32))
        .collect();

    // Find extreme points
    let top_left = points
        .iter()
        .min_by(|a, b| (a.x + a.y).partial_cmp(&(b.x + b.y)).unwrap())?;
    let bottom_right = points
        .iter()
        .max_by(|a, b| (a.x + a.y).partial_cmp(&(b.x + b.y)).unwrap())?;
    let top_right = points
        .iter()
        .max_by(|a, b| (a.x - a.y).partial_cmp(&(b.x - b.y)).unwrap())?;
    let bottom_left = points
        .iter()
        .min_by(|a, b| (a.x - a.y).partial_cmp(&(b.x - b.y)).unwrap())?;

    Some([
        top_left.into(),
        top_right.into(),
        bottom_right.into(),
        bottom_left.into(),
    ])
}

// pub fn find_rectangle_vertices(img: &RgbImage) -> Option<[Point<f32>; 4]> {
//     let (width, height) = img.dimensions();
//     let mut corners = Vec::new();

//     // Find dark pixels (black frame)
//     for y in 5..height - 5 {
//         for x in 5..width - 5 {
//             let pixel = img.get_pixel(x, y);
//             let brightness = (pixel[0] as u32 + pixel[1] as u32 + pixel[2] as u32) / 3;

//             if brightness < 80 {
//                 // Check if this is a corner (two perpendicular edges)
//                 let mut dark_dirs = 0;

//                 // Check 4 directions
//                 let left_dark = (0..10)
//                     .filter(|&i| {
//                         let p = img.get_pixel(x.saturating_sub(i), y);
//                         (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3 < 80
//                     })
//                     .count()
//                     > 5;

//                 let right_dark = (0..10)
//                     .filter(|&i| {
//                         if x + i >= width {
//                             return false;
//                         }
//                         let p = img.get_pixel(x + i, y);
//                         (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3 < 80
//                     })
//                     .count()
//                     > 5;

//                 let top_dark = (0..10)
//                     .filter(|&i| {
//                         let p = img.get_pixel(x, y.saturating_sub(i));
//                         (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3 < 80
//                     })
//                     .count()
//                     > 5;

//                 let bottom_dark = (0..10)
//                     .filter(|&i| {
//                         if y + i >= height {
//                             return false;
//                         }
//                         let p = img.get_pixel(x, y + i);
//                         (p[0] as u32 + p[1] as u32 + p[2] as u32) / 3 < 80
//                     })
//                     .count()
//                     > 5;

//                 if left_dark {
//                     dark_dirs += 1;
//                 }
//                 if right_dark {
//                     dark_dirs += 1;
//                 }
//                 if top_dark {
//                     dark_dirs += 1;
//                 }
//                 if bottom_dark {
//                     dark_dirs += 1;
//                 }

//                 if dark_dirs >= 2 && ((left_dark || right_dark) && (top_dark || bottom_dark)) {
//                     corners.push(Point::new(x as f32, y as f32));
//                 }
//             }
//         }
//     }

//     if corners.len() < 4 {
//         return None;
//     }

//     // Find 4 extreme corners
//     corners.sort_by(|a, b| (a.x + a.y).partial_cmp(&(b.x + b.y)).unwrap());
//     let top_left = corners[0];
//     let bottom_right = *corners.last().unwrap();

//     corners.sort_by(|a, b| (a.x - a.y).partial_cmp(&(b.x - b.y)).unwrap());
//     let top_right = *corners.last().unwrap();
//     let bottom_left = corners[0];

//     Some([top_left, top_right, bottom_right, bottom_left])
// }

// use image::RgbImage;
// use imageproc::contrast::threshold;
// use imageproc::edges::canny;
// // use imageproc::point::Point;

// pub fn find_rectangle_vertices(img: &RgbImage) -> Option<[Point<f32>; 4]> {
//     let gray = image::imageops::grayscale(img);
//     let edges = canny(&gray, 50.0, 100.0);

//     let (width, height) = edges.dimensions();
//     let mut corners = Vec::new();

//     // Find corner points on edges
//     for y in 5..height - 5 {
//         for x in 5..width - 5 {
//             if edges.get_pixel(x, y)[0] == 0 {
//                 continue;
//             }

//             // Count edge directions around this pixel
//             let mut h_edges = 0;
//             let mut v_edges = 0;

//             for i in 1..6 {
//                 if x >= i && edges.get_pixel(x - i, y)[0] > 0 {
//                     h_edges += 1;
//                 }
//                 if x + i < width && edges.get_pixel(x + i, y)[0] > 0 {
//                     h_edges += 1;
//                 }
//                 if y >= i && edges.get_pixel(x, y - i)[0] > 0 {
//                     v_edges += 1;
//                 }
//                 if y + i < height && edges.get_pixel(x, y + i)[0] > 0 {
//                     v_edges += 1;
//                 }
//             }

//             // Corner has both horizontal and vertical edges
//             if h_edges >= 3 && v_edges >= 3 {
//                 corners.push(Point::new(x as f32, y as f32));
//             }
//         }
//     }

//     if corners.len() < 4 {
//         return None;
//     }

//     // Find 4 extreme corners
//     corners.sort_by(|a, b| (a.x + a.y).partial_cmp(&(b.x + b.y)).unwrap());
//     let top_left = corners[0];
//     let bottom_right = *corners.last().unwrap();

//     corners.sort_by(|a, b| (a.x - a.y).partial_cmp(&(b.x - b.y)).unwrap());
//     let top_right = *corners.last().unwrap();
//     let bottom_left = corners[0];

//     Some([top_left, top_right, bottom_right, bottom_left])
// }
