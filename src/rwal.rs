use std::io::{Write, Read};
use std::path::Path;
use std::fs;

use image;
use image::DynamicImage;
use image::imageops::Nearest;
use image::GenericImageView;

#[derive(Clone, Eq, PartialEq)]
enum Color {
    RGB(RGB),
    HEX(HEX),
}

#[derive(Clone, Eq, PartialEq)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Clone, Eq, PartialEq)]
struct HEX {
    r: String,
    g: String,
    b: String,
}

impl Color {
    fn to_hex(&self) -> Color {
        match self {
            Color::RGB(rgb) => Color::HEX(HEX {
                r: format!("{:X}", rgb.r),
                g: format!("{:X}", rgb.g),
                b: format!("{:X}", rgb.b),
            }),
            Color::HEX(hex) => self.clone(),
        }
    }

    fn to_rgb(&self) -> Color {
        match self {
            Color::RGB(rgb) => self.clone(),
            Color::HEX(hex) => Color::RGB(RGB {
                r: u8::from_str_radix(&hex.r, 16).unwrap(),
                g: u8::from_str_radix(&hex.g, 16).unwrap(),
                b: u8::from_str_radix(&hex.b, 16).unwrap(),
            }),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Color::RGB(rgb) => {
                return  format!("{} {} {}", rgb.r, rgb.g, rgb.b);
            },
            Color::HEX(hex) => {
                let mut r = hex.r.clone();
                let mut g = hex.g.clone();
                let mut b = hex.b.clone();

                if r.len() == 1 {r = format!("0{}", r); }
                if g.len() == 1 {g = format!("0{}", g); }
                if b.len() == 1 {b = format!("0{}", b); }
                
                return  format!("#{}{}{}", r, g, b);
            }
        }
    }

    fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        Color::RGB(RGB { r, g, b })
    }

    fn new_hex(r: &str, g: &str, b: &str) -> Self {
        Color::HEX(HEX {
            r: r.to_string(),
            g: g.to_string(),
            b: b.to_string(),
        })
    }

    fn in_color_limit(&self, start: u8, end: u8) -> bool {
        match self {
            Color::RGB(rgb) => {
                let r: u8 = rgb.r;
                let g: u8 = rgb.g;
                let b: u8 = rgb.b;

                if r < start || r > end {return false;}
                if g < start || g > end {return false;}
                if b < start || b > end {return false;}
                return true;
            },
            Color::HEX(hex) => {
                let r = u8::from_str_radix(&hex.r, 16).unwrap();
                let g = u8::from_str_radix(&hex.g, 16).unwrap();
                let b = u8::from_str_radix(&hex.b, 16).unwrap();

                if r < start || r > end {return false;}
                if g < start || g > end {return false;}
                if b < start || b > end {return false;}
                return true;
            }
        }
    }

    fn color_sum(&self) -> u16 {
        match self {
            Color::RGB(rgb) => {
                let r: u16 = rgb.r as u16;
                let g: u16 = rgb.g as u16;
                let b: u16 = rgb.b as u16;

                return r + g + b;
            },
            Color::HEX(hex) => {
                let r = u16::from_str_radix(&hex.r, 16).unwrap();
                let g = u16::from_str_radix(&hex.g, 16).unwrap();
                let b = u16::from_str_radix(&hex.b, 16).unwrap();

                return r + g + b;
            }
        }
    }
}

#[derive(Debug)]
enum Error {
    TooLarge,
}

type PalletteReadResult<T, E> = Result<T, E>;

pub struct Rwal {
    image: DynamicImage,
    image_name: String,
    cache_dir: String,
    chunk_size: u32,
    chunk_count: u32,
    dark_b: u8,
    darl_t: u8,
    light_b: u8,
    light_t: u8,
    wanted_color_sum: u16,
}

impl Rwal {
    pub fn new(
        image_path: &str, 
        image_name: &str,
        cache_dir: &str, 
        chunk_size: u32, 
        chunk_count: u32,
        dark_b: u8,
        darl_t: u8,
        light_b: u8,
        light_t: u8,
        wanted_color_sum: u16,
    ) -> Self {
        let image = image::open(image_path).unwrap();

        Rwal {
            image,
            image_name: image_name.to_string(),
            cache_dir: cache_dir.to_string(),
            chunk_size,
            chunk_count,
            dark_b,
            darl_t,
            light_b,
            light_t,
            wanted_color_sum,
        }
    }
    pub fn from_dynamic_image(
        image: &DynamicImage, 
        image_name: &str,
        cache_dir: &str, 
        chunk_size: u32, 
        chunk_count: u32,
        dark_b: u8,
        darl_t: u8,
        light_b: u8,
        light_t: u8,
        wanted_color_sum: u16,
    ) -> Self {
        Rwal {
            image: image.clone(),
            image_name: image_name.to_string(),
            cache_dir: cache_dir.to_string(),
            chunk_size,
            chunk_count,
            dark_b,
            darl_t,
            light_b,
            light_t,
            wanted_color_sum,
        }
    }

    fn get_colors(
        &self, 
        dark_b: u8, 
        darl_t: u8, 
        light_b: u8, 
        light_t: u8, 
        wanted_color_sum: u16, 
        chunk_size: u32, 
        chunk_count: u32
    ) -> (Vec<Color>, Color, Color) {
    
        let scaled_image = self.image.resize_exact(
            chunk_size*chunk_count, 
            chunk_size*chunk_count, 
            Nearest
        );
        let mut colors: Vec<Color> = Vec::new();

        for i in 0..chunk_count {
            for j in 0..chunk_count {
                let chunk_size = chunk_size;
                let mut chunk = scaled_image.crop_imm(j*chunk_size, i*chunk_size, chunk_size, chunk_size);
                chunk = chunk.resize_exact(1, 1, Nearest);
                let pixel = chunk.get_pixel(0, 0);

                colors.push(Color::new_rgb(pixel[0], pixel[1], pixel[2]));                  
            }
        }

        let mut another_colors: Vec<Color> = Vec::new();

        let mut l_dark_color: Color = Color::new_rgb(0, 0, 0);
        let mut l_dark_color_sum: u16 = l_dark_color.color_sum();
        let mut d_light_color: Color = Color::new_rgb(255, 255, 255);
        let mut d_light_color_sum: u16 = d_light_color.color_sum();

        for _color in &colors {
            if _color.in_color_limit(dark_b, darl_t) {
                let _color_sum = _color.color_sum();
                if _color_sum > l_dark_color_sum {
                    l_dark_color_sum = _color_sum;
                    l_dark_color = _color.clone();
                }
            }
            else if _color.in_color_limit(light_b, light_t) {
                let _color_sum = _color.color_sum();
                if _color_sum < d_light_color_sum {
                    d_light_color_sum = _color_sum;
                    d_light_color = _color.clone();
                }
            }
            else if _color.in_color_limit(darl_t, light_b) {
                let _color_sum = _color.color_sum();
                if _color_sum <= wanted_color_sum {
                    another_colors.push(_color.clone());
                }
            }
        }

        (another_colors, l_dark_color, d_light_color)
    }


    fn get_pallete(&self) -> PalletteReadResult<Vec<Color>, Error> {
        let mut colors: Vec<Color> = Vec::new();
        let mut dark_color: Color = Color::new_rgb(0, 0, 0);
        let mut light_color: Color = Color::new_rgb(255, 255, 255);

        let mut i = 0;
        let max_iterations: usize = 10;

        let dark_b = self.dark_b;
        let darl_t = self.darl_t;
        let light_b = self.light_b;
        let light_t = self.light_t;
        let wanted_color_sum = self.wanted_color_sum;
        let chunk_size = self.chunk_size;
        let mut chunk_count = self.chunk_count;

        while i < max_iterations {
            let (_colors, _dark_color, _light_color) = self.get_colors(
                dark_b, 
                darl_t, 
                light_b, 
                light_t, 
                wanted_color_sum, 
                chunk_size, 
                chunk_count
            );

            i += 1;
            
            if _colors.len() < 6 {
                chunk_count += 1;
                continue;
            } 
            
            colors = _colors;
            dark_color = _dark_color;
            light_color = _light_color;
            break;
        }
        if i >= max_iterations {
            return Err(Error::TooLarge);
        }

        colors.sort_by(|a, b| b.color_sum().cmp(&a.color_sum()));

        if colors.len() >= 6 {
            let pallete = vec![
                dark_color, 
                colors[0].clone(), 
                colors[1].clone(), 
                colors[2].clone(), 
                colors[3].clone(), 
                colors[4].clone(), 
                colors[5].clone(), 
                light_color
                ];
                return Ok(pallete);  
        }

        return Err(Error::TooLarge);
    }

    fn get_pallete_cache_path(&self) -> String {
        format!("{}/{}", self.cache_dir, self.image_name)
    }

    fn is_cached(&self) -> bool {
        let cache_path = self.get_pallete_cache_path();

        Path::new(&cache_path).exists()
    }

    fn read_from_cache(&self) -> String {
        let cache_path = self.get_pallete_cache_path();

        fs::read_to_string(cache_path).unwrap()
    }

    fn cache(&self, pallete: &str) {
        let cache_path = self.get_pallete_cache_path();

        fs::File::create(&cache_path).unwrap();
        fs::write(cache_path, pallete).unwrap();
    }

    pub fn run(&self) -> String {
        let mut res: String = String::new();
        let pallete: Vec<Color>;

        if self.is_cached() {
            res = self.read_from_cache();
        } else {
            pallete = self.get_pallete().unwrap();
            for i in 0..2 {
                for el in &pallete {
                    res += &format!("{}\n", el.to_hex().to_string());
                }
            }
            self.cache(&res);
        }

        return res;
    }
}
