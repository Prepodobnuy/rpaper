//pub struct Rgb {
//    r: u32, // 0-255
//    g: u32, // 0-255
//    b: u32, // 0-255
//}
//
//pub struct Rgba {
//    r: u32, // 0-255
//    g: u32, // 0-255
//    b: u32, // 0-255
//    a: u32, // 0-255
//}
//
//pub struct Hsv {
//    h: u32, // 0-360
//    s: u32, // 0-100
//    v: u32, // 0-100
//}
//
//pub struct Hsva {
//    h: u32, // 0-360
//    s: u32, // 0-100
//    v: u32, // 0-100
//    a: u32, // 0-255
//}
//
//impl Rgb {
//    pub fn new(r: u32, g: u32, b: u32) -> Self {
//        Rgb {
//            r,
//            g,
//            b,
//        }
//    }
//}
//
//impl Rgba {
//    pub fn new(r: u32, g: u32, b: u32, a: u32) -> Self {
//        Rgba {
//            r,
//            g,
//            b,
//            a,
//        }
//    }
//}
//
//impl Hsv {
//    pub fn new(h: u32, s: u32, v: u32) -> Self {
//        Hsv {
//            h,
//            s,
//            v,
//        }
//    }
//}
//
//impl Hsva {
//    pub fn new(h: u32, s: u32, v: u32, a: u32) -> Self {
//        Hsva {
//            h,
//            s,
//            v,
//            a,
//        }
//    }
//}
//
//pub enum Color {
//    Rgb(Rgb),
//    Rgba(Rgba),
//    Hsv(Hsv),
//    Hsva(Hsva),
//}
//
//impl Color {
//    pub fn into_rgb(color: Color) -> Rgb {
//        match color {
//            Color::Rgb(rgb) => {rgb},
//            Color::Rgba(rgba) => {Rgb::new(rgba.r, rgba.g, rgba.b)},
//            Color::Hsv(hsv) => {todo!()},
//            Color::Hsva(hsva) => {todo!()},
//        }
//    }
//
//    pub fn into_rgba(color: Color) -> Rgba {
//        match color {
//            Color::Rgb(rgb) => {Rgba::new(rgb.r, rgb.g, rgb.b, 0)},
//            Color::Rgba(rgba) => {rgba},
//            Color::Hsv(hsv) => {todo!()},
//            Color::Hsva(hsva) => {todo!()},
//        }
//    }
//
//    pub fn into_hsv(color: Color) -> Hsv {
//        todo!()
//    }
//
//    pub fn into_hsva(color: Color) -> Hsva {
//        todo!()
//    }
//}
//
//