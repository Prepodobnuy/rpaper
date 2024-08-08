use serde_json::Value;

pub struct Template {
    pub temp_path: String,
    pub conf_path: String,
    pub use_quotes: bool,
    pub use_sharps: bool,
    pub opacity: String,
    pub command: String,
}

pub fn get_templates(data: &Value) -> Vec<Template> {
    let mut res: Vec<Template> = Vec::new();
    for raw_template in data["templates"].as_array().unwrap() {
        res.push(Template {
            temp_path: String::from(raw_template["template_path"].as_str().unwrap()),
            conf_path: String::from(raw_template["config_path"].as_str().unwrap()),
            use_quotes: raw_template["use_quotes"].as_bool().unwrap(),
            use_sharps: raw_template["use_sharps"].as_bool().unwrap(),
            opacity: String::from(raw_template["opacity"].as_str().unwrap()),
            command: String::from(raw_template["command"].as_str().unwrap()),
        })
    }
    return res;
}