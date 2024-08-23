use serde_json::Value;

#[derive(Clone)]
pub struct Display {
    pub width: u32,
    pub height: u32,
    pub margin_left: u32,
    pub margin_top: u32,
    pub name: String,
}

pub fn get_displays(data: &Value) -> Vec<Display> {
    let mut displays: Vec<Display> = Vec::new();
    for raw_display in data["displays"].as_array().unwrap() {
        displays.push(Display {
            width: raw_display["width"].as_u64().unwrap() as u32,
            height: raw_display["height"].as_u64().unwrap() as u32,
            margin_left: raw_display["margin-left"].as_u64().unwrap() as u32,
            margin_top: raw_display["margin-top"].as_u64().unwrap() as u32,
            name: String::from(raw_display["name"].as_str().unwrap()),
        })
    }
    return displays;
}

pub fn max_width(displays: &Vec<Display>) -> u32 {
    let mut res: u32 = 0;
    for display in displays {
        if display.width + display.margin_left > res {
            res = display.width + display.margin_left
        }
    }

    return res;
}

pub fn max_height(displays: &Vec<Display>) -> u32 {
    let mut res: u32 = 0;
    for display in displays {
        if display.height + display.margin_top > res {
            res = display.height + display.margin_top
        }
    }

    return res;
}
