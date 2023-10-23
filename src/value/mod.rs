pub mod object;
pub mod function;

use std::cell::Ref;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::ops;
use crate::bytecode::bytecode_interpreter::VM;
use crate::keywords::{NULL_KEYWORD, UNDEFINED_KEYWORD};
use crate::value::function::JsFunction;
use crate::value::object::{JsObject, JsObjectRef, ObjectKind};

#[derive(Debug, Clone, PartialEq)]
pub enum JsValue {
    Undefined,
    Null,
    String(String),
    Number(f64),
    Boolean(bool),
    Object(JsObjectRef),
}

impl JsValue {
    pub fn is_function(&self) -> bool {
        match self {
            JsValue::Object(obj) => matches!(obj.borrow().kind, ObjectKind::Function(_)),
            _ => false
        }
    }

    pub fn native_function(function: fn(&[JsValue]) -> Result<JsValue, String>) -> Self {
        JsFunction::native_function(function).into()
    }

    pub fn native_bytecode_function(function: fn(&VM, &[JsValue]) -> Result<JsValue, String>) -> Self {
        JsFunction::native_bytecode_function(function).into()
    }

    pub fn as_string(&self) -> &str {
        if let JsValue::String(string) = self {
            return string;
        }

        panic!("{}", format!("not a string, actual type: {}", self.get_type_as_str()))
    }

    pub fn as_object(&self) -> &JsObjectRef {
        if let JsValue::Object(object) = self {
            return object;
        }

        panic!("{}", format!("not an object, actual type: {}", self.get_type_as_str()))
    }

    pub fn object<T: Into<HashMap<String, JsValue>>>(properties: T) -> Self {
        JsObject::new(ObjectKind::Ordinary, properties).into()
    }

    pub fn get_type_as_str(&self) -> String {
        match self {
            JsValue::Undefined => UNDEFINED_KEYWORD.to_string(),
            JsValue::Null => NULL_KEYWORD.to_string(),
            JsValue::String(_) => "string".to_string(),
            JsValue::Number(_) => "number".to_string(),
            JsValue::Boolean(_) => "boolean".to_string(),
            JsValue::Object(_) => {
                if self.is_function() {
                    "function".to_string()
                } else {
                    "object".to_string()
                }
            },
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            JsValue::Undefined => false,
            JsValue::Null => false,
            JsValue::String(value) => value.len() != 0,
            JsValue::Number(value) => *value != 0.0,
            JsValue::Boolean(value) => *value,
            JsValue::Object(_) => true,
        }
    }

    pub fn to_bool_js_value(&self) -> Self {
        JsValue::Boolean(self.to_bool())
    }

    pub fn exponentiation(&self, rhs: &JsValue) -> Result<JsValue, String> {
        match (self, rhs) {
            (JsValue::Number(left_number), JsValue::Number(right_number)) => {
                Ok(JsValue::Number(left_number.powf(*right_number)))
            }
            _ => Err(format!(
                "exponentiation of types '{}' and '{}' is not possible",
                self.get_type_as_str(),
                rhs.get_type_as_str()
            )),
        }
    }

    pub fn display_with_no_colors(&self) -> String {
        match self {
            JsValue::Undefined => format!("{UNDEFINED_KEYWORD}"),
            JsValue::Null => format!("{NULL_KEYWORD}"),
            JsValue::String(str) => format!("\"{}\"", str),
            JsValue::Number(number) => format!("{}", number),
            JsValue::Boolean(value) => format!("{}", if *value { "true" } else { "false" }),
            JsValue::Object(object) => {
                match &object.borrow().kind {
                    ObjectKind::Ordinary => {
                        let result: Vec<String> = object.borrow().properties
                            .iter()
                            .map(|(key, value)| format!("{key}: {value}"))
                            .collect();
                        let result = result.join(", ");
                        format!("{{ {result} }}")
                    },
                    ObjectKind::Function(function) => {
                        match function {
                            JsFunction::Ordinary(_) => format!("[function]"),
                            JsFunction::Bytecode(function) => format!("[function {}]", function.name),
                            JsFunction::Native(_) | JsFunction::NativeBytecode(_) => format!("[native function]"),
                        }
                    },
                    ObjectKind::Array => {
                        let result: Vec<String> = object.borrow().properties
                            .values()
                            .map(|x| format!("{x}"))
                            .collect();
                        let result = result.join(", ");
                        format!("[{result}]")
                    }
                }
            },
        }
    }
}

impl From<f64> for JsValue {
    fn from(value: f64) -> Self {
        JsValue::Number(value)
    }
}

impl From<bool> for JsValue {
    fn from(value: bool) -> Self {
        JsValue::Boolean(value)
    }
}

impl From<String> for JsValue {
    fn from(value: String) -> Self {
        JsValue::String(value)
    }
}

impl PartialOrd for JsValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (JsValue::Number(left_number), JsValue::Number(right_number)) => {
                left_number.partial_cmp(right_number)
            },
            _ => None
        }
    }
}

impl ops::Add<&JsValue> for &JsValue {
    type Output = Result<JsValue, String>;

    fn add(self, rhs: &JsValue) -> Self::Output {
        match (self, rhs) {
            (JsValue::Number(first_number), JsValue::Number(second_number)) => Ok(JsValue::Number(first_number + second_number)),
            (JsValue::String(first_string), JsValue::String(second_string)) => Ok(JsValue::String(format!("{}{}", first_string, second_string.as_str()))),
            (JsValue::String(left_string), JsValue::Number(right_number)) => {
                Ok(JsValue::String(format!("{}{}", left_string, right_number.to_string())))
            }
            _ => Err(format!(
                "addition of types '{}' and '{}' is not possible",
                &self.get_type_as_str(),
                &rhs.get_type_as_str()
            ))
        }
    }
}

impl ops::Sub<&JsValue> for &JsValue {
    type Output = Result<JsValue, String>;

    fn sub(self, rhs: &JsValue) -> Self::Output {
        match (self, rhs) {
            (JsValue::Number(first_number), JsValue::Number(second_number)) => Ok(JsValue::Number(first_number - second_number)),
            _ => Err(format!(
                "subtraction of types '{}' and '{}' is not possible",
                &self.get_type_as_str(),
                &rhs.get_type_as_str()
            ))
        }
    }
}

impl ops::Mul<&JsValue> for &JsValue {
    type Output = Result<JsValue, String>;

    fn mul(self, rhs: &JsValue) -> Self::Output {
        match (self, rhs) {
            (JsValue::Number(first_number), JsValue::Number(second_number)) => Ok(JsValue::Number(first_number * second_number)),
            (JsValue::String(string), JsValue::Number(number)) => Ok(JsValue::String(string.repeat(*number as usize))),
            _ => Err(format!(
                "multiplication of types '{}' and '{}' is not possible",
                &self.get_type_as_str(),
                &rhs.get_type_as_str()
            ))
        }
    }
}

impl ops::Div<&JsValue> for &JsValue {
    type Output = Result<JsValue, String>;

    fn div(self, rhs: &JsValue) -> Self::Output {
        match (self, rhs) {
            (JsValue::Number(first_number), JsValue::Number(second_number)) => Ok(JsValue::Number(first_number / second_number)),
            _ => Err(format!(
                "division of types '{}' and '{}' is not possible",
                &self.get_type_as_str(),
                &rhs.get_type_as_str()
            ))
        }
    }
}

impl Display for JsValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JsValue::Undefined => write!(f, "\x1b[37m{}\x1b[0m", self.display_with_no_colors()),
            JsValue::Null => write!(f, "{NULL_KEYWORD}"),
            JsValue::String(str) => write!(f, "\x1b[93m\"{}\"\x1b[0m", str),
            JsValue::Number(number) => write!(f, "\x1b[36m{}\x1b[0m", number),
            JsValue::Boolean(value) => write!(f, "\x1b[35m{}\x1b[0m", if *value { "true" } else { "false" }),
            JsValue::Object(object) => {
                match &object.borrow().kind {
                    ObjectKind::Ordinary => {
                        let result: Vec<String> = object.borrow().properties
                            .iter()
                            .map(|(key, value)| format!("{key}: {value}"))
                            .collect();
                        let result = result.join(", ");
                        write!(f, "{{ {result} }}")
                    },
                    ObjectKind::Function(function) => {
                        match function {
                            JsFunction::Ordinary(_) => write!(f, "[function]"),
                            JsFunction::Bytecode(function) => write!(f, "[function {}]", function.name),
                            JsFunction::Native(_) | JsFunction::NativeBytecode(_) => write!(f, "[native function]"),
                        }
                    },
                    ObjectKind::Array => {
                        let result: Vec<String> = object.borrow().properties
                            .values()
                            .map(|x| format!("{x}"))
                            .collect();
                        let result = result.join(", ");
                        write!(f, "[{result}]")
                    }
                }
            },
        }
    }
}
