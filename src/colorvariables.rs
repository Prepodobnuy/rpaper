use serde_json::Value;

pub struct ColorVariable {
    pub name: String,
    pub value: usize,
    pub brightness: i32,
}

pub fn get_color_variables(data: &Value) -> Vec<ColorVariable> {
    let mut colors: Vec<ColorVariable> = Vec::new();
    for raw_variable in data.as_array().unwrap() {
        colors.push(ColorVariable {
            name: String::from(raw_variable["name"].as_str().unwrap()),
            value: raw_variable["value"].as_u64().unwrap() as usize,
            brightness: raw_variable["brightness"].as_i64().unwrap() as i32,
        })
    }
    return colors;
}
