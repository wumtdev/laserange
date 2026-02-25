use image::{GenericImageView, RgbImage, SubImage, math::Rect};

#[derive(Debug, Clone, Copy)]
pub struct Stencil {
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
}

impl Stencil {
    pub fn new(start_x: f32, start_y: f32, end_x: f32, end_y: f32) -> Self {
        Self {
            start_x,
            start_y,
            end_x,
            end_y,
        }
    }

    pub fn crop<'a>(&self, img: &'a RgbImage) -> SubImage<&'a RgbImage> {
        let (width, height) = img.dimensions();
        let r = self.rect(width, height);
        img.view(r.x, r.y, r.width, r.height)
    }

    pub fn rect(&self, width: u32, height: u32) -> Rect {
        let (width, height) = (width as f32, height as f32);
        Rect {
            x: (self.start_x * width) as u32,
            y: (self.start_y * height) as u32,
            width: ((self.end_x - self.start_x) * width) as u32,
            height: ((self.end_y - self.start_y) * height) as u32,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Stencil {
    fn from(v: (f32, f32, f32, f32)) -> Self {
        Stencil {
            start_x: v.0,
            start_y: v.1,
            end_x: v.2,
            end_y: v.3,
        }
    }
}

impl Default for Stencil {
    fn default() -> Self {
        Self {
            start_x: 0.0,
            start_y: 0.0,
            end_x: 1.0,
            end_y: 1.0,
        }
    }
}
