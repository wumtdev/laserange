use image::{GrayImage, Luma};
use imageproc::{
    contours::find_contours, distance_transform::Norm, drawing::draw_polygon_mut, edges::canny,
    filter::gaussian_blur_f32, morphology::dilate_mut, point::Point,
};

const BLUR_SIGMA: f32 = 1.5;
const CANNY_LOW: f32 = 7.0;
const CANNY_HIGH: f32 = 8.0;
const DILATE_RADIUS: u8 = 1;
const MIN_CONTOUR_LEN: usize = 40;

fn find_zones(img: &GrayImage) -> (GrayImage, u8) {
    let (width, height) = img.dimensions();

    // Apply blur
    let preprocessed = if BLUR_SIGMA > 0.0 {
        gaussian_blur_f32(img, BLUR_SIGMA)
    } else {
        img.clone()
    };

    // Highlight edges
    let mut edges = canny(&preprocessed, CANNY_LOW, CANNY_HIGH);

    // Dilate to merge neighbor edges
    dilate_mut(&mut edges, Norm::LInf, DILATE_RADIUS);

    let contours = find_contours::<i32>(&edges);

    let mut sorted_indices: Vec<usize> = (0..contours.len()).collect();
    sorted_indices.sort_by_cached_key(|i| {
        let mut depth = 0;
        let mut current = *i;
        loop {
            match contours[current].parent {
                Some(p) => {
                    depth += 1;
                    current = p;
                }
                None => break,
            }
        }
        depth
    });

    let mut zone_map = GrayImage::new(width, height);
    let zone_count = sorted_indices.len() + 1; // +1 background zero zone
    let last_zone_id = zone_count - 1;
    for (i, contour_id) in sorted_indices.iter().enumerate() {
        let zone_id = last_zone_id - i;
        draw_polygon_mut(
            &mut zone_map,
            &contours[*contour_id].points,
            Luma([zone_id as u8]),
        );
    }

    (zone_map, zone_count as u8)
}

/// Maps positions on image to zone
pub struct ZoneMap {
    map: GrayImage,
    count: u8,
}

impl ZoneMap {
    /// Recognize zones on image and build zone map
    pub fn recognize(img: &GrayImage) -> Self {
        let (map, count) = find_zones(img);
        Self { map, count }
    }

    /// Load zone map from zone mapping image
    pub fn load(map: GrayImage) -> Self {
        let count = map.pixels().map(|Luma([v])| *v).max().unwrap_or(0) + 1;
        Self { map, count }
    }

    /// Zone mapping image
    pub fn map(&self) -> &GrayImage {
        &self.map
    }

    /// Zone count (including background zero zone)
    pub fn count(&self) -> u8 {
        self.count
    }

    /// Map pos on target to zone id
    pub fn at(&self, p: Point<u32>) -> u8 {
        self.map.get_pixel(p.x, p.y).0[0]
    }
}
