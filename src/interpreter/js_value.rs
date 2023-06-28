use std::fmt::Display;
use crate::node::NodeKind;
use super::environment::Environment;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc, cell::RefMut};

#[derive(Debug, Clone, PartialEq)]
pub enum JsValue {
    Undefined,
    Null,
    String(String),
    Number(f64),
    Boolean(bool),
    Function(JsFunction),
    Object(Rc<RefCell<JsObject>>),
}

pub fn create_js_object(value: JsObject) -> JsValue {
    JsValue::Object(Rc::new(RefCell::new(value)))
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsObject {
    pub properties: HashMap<String, JsValue>,
}

impl JsObject {
    pub fn new_empty() -> Self {
        Self {
            properties: HashMap::new()
        }
    }

    pub fn new_with_properties<T: Into<HashMap<String, JsValue>>>(value: T) -> Self {
        Self {
            properties: value.into(),
        }
    }

    pub fn add_property(&mut self, key: &str, value: JsValue) {
//        println!("add_property {key} {value:?}");
        self.properties.insert(key.to_string(), value);
    }

    pub fn get_value_property(&self, key: &str) -> JsValue {
//        println!("get_value_property {key} {:#?}", self.properties);
        return self.properties.get(key).map_or(JsValue::Undefined, |x| x.clone());
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsFunction {
//    pub name: String,
    pub arguments: Vec<JsFunctionArg>,
    pub body: Box<NodeKind>,
    pub environment: Box<Environment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsFunctionArg {
    pub name: String,
    pub default_value: JsValue,
}

impl JsValue {
    pub fn get_type_as_str(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::String(_) => "string".to_string(),
            JsValue::Number(_) => "number".to_string(),
            JsValue::Boolean(_) => "boolean".to_string(),
            JsValue::Function(_) => "function".to_string(),
            JsValue::Object(_) => "object".to_string(),
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            JsValue::Undefined => false,
            JsValue::Null => false,
            JsValue::String(value) => value.len() != 0,
            JsValue::Number(value) => *value != 0.0,
            JsValue::Boolean(value) => *value,
            JsValue::Function(_) | JsValue::Object(_) => true,
        }
    }

    pub fn to_bool_js_value(&self) -> Self {
        JsValue::Boolean(self.to_bool())
    }
}

impl Display for JsValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsValue::Undefined => write!(f, "\x1b[37mundefined\x1b[0m"),
            JsValue::Null => write!(f, "null"),
            JsValue::String(str) => write!(f, "\x1b[93m\"{}\"\x1b[0m", str),
            JsValue::Number(number) => write!(f, "\x1b[36m{}\x1b[0m", number),
            JsValue::Boolean(value) => write!(f, "{}", if *value { "true" } else { "false" }),
            JsValue::Function(js_function) => write!(f, "[function]"),
            JsValue::Object(_) => write!(f, "[object Object]"),
        }
    }
}
