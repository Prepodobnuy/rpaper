#[derive(Clone)]
pub struct Display {
    pub w: u32,
    pub h: u32,
    pub x: u32,
    pub y: u32,
    pub name: String,
}

impl Display {
    pub fn new(w: u32, h: u32, x: u32, y: u32, name: String) -> Self {
        Display { w, h, x, y, name }
    }
}

pub fn displays_max_width(displays: &Vec<Display>) -> u32 {
    let mut res: u32 = 0;
    for display in displays {
        if display.w + display.x > res {
            res = display.w + display.x
        }
    }

    res
}

pub fn displays_max_height(displays: &Vec<Display>) -> u32 {
    let mut res: u32 = 0;
    for display in displays {
        if display.h + display.y > res {
            res = display.h + display.y
        }
    }

    res
}
