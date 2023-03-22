use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum JsValue {
    Undefined,
    Null,
    String(String),
    Number(f64),
    Boolean(bool),
}

impl JsValue {
    pub fn get_type_as_str(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::String(_) => "string".to_string(),
            JsValue::Number(_) => "number".to_string(),
            JsValue::Boolean(_) => "boolean".to_string(),
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            JsValue::Undefined => false,
            JsValue::Null => false,
            JsValue::String(value) => value.len() != 0,
            JsValue::Number(value) => *value != 0.0,
            JsValue::Boolean(value) => *value,
        }
    }

    pub fn to_bool_js_value(&self) -> Self {
        JsValue::Boolean(self.to_bool())
    }
}

impl Display for JsValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsValue::Undefined => write!(f, "undefined"),
            JsValue::Null => write!(f, "null"),
            JsValue::String(str) => write!(f, "\"{}\"", str),
            JsValue::Number(number) => write!(f, "{}", number),
            JsValue::Boolean(value) => write!(f, "{}", if *value { "true" } else { "false" }),
        }
    }
}
