#[derive(Clone, Copy)]
pub enum OrderBy {
    Hue,
    Saturation,
    Brightness,
    Semantic,
}

#[derive(Clone)]
pub struct RwalParams {
    pub thumb_range: (u32, u32),
    pub clamp_range: (f32, f32),
    pub accent_color: u32,
    pub colors: u32,
    pub order: OrderBy,
}

impl RwalParams {
    pub fn new(
        thumb_range: (u32, u32),
        clamp_range: (f32, f32),
        accent_color: u32,
        colors: u32,
        order: OrderBy,
    ) -> Self {
        RwalParams {
            thumb_range,
            clamp_range,
            accent_color,
            colors,
            order,
        }
    }
}
