use std::fmt::{Debug, Display, Formatter};
use crate::node::{BlockStatementNode, AstStatement};
use super::environment::Environment;
use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc, ops};
use crate::interpreter::Interpreter;
use crate::scanner::TokenKind::Percent;

pub type JsObjectRef = Rc<RefCell<JsObject>>;

#[derive(Debug, Clone, PartialEq)]
pub enum JsValue {
    Undefined,
    Null,
    String(String),
    Number(f64),
    Boolean(bool),
    Function(Func),
    Object(JsObjectRef),
}

pub fn create_js_object(value: JsObject) -> JsValue {
    JsValue::Object(Rc::new(RefCell::new(value)))
}

#[derive(Default, Debug, Clone, PartialEq)]
struct InternalObject {
    pub properties: HashMap<String, JsValue>,
    pub prototype: Option<JsObjectRef>,
}

pub trait Object {
    fn set_prototype(&mut self, prototype: JsObjectRef);
    fn add_property(&mut self, key: &str, value: JsValue);
    fn get_property_value(&self, key: &str) -> JsValue;
}

impl Object for InternalObject {
    fn set_prototype(&mut self, prototype: JsObjectRef) {
        self.prototype = Some(prototype);
    }

    fn add_property(&mut self, key: &str, value: JsValue) {
        self.properties.insert(key.to_string(), value);
    }

    fn get_property_value(&self, key: &str) -> JsValue {
        if self.properties.contains_key(key) {
            return self.properties.get(key).map_or(JsValue::Undefined, |x| x.clone());
        }

        if self.prototype.is_some() {
            return self.prototype.as_ref().unwrap().borrow().get_property_value(key);
        }

        return JsValue::Undefined;
    }
}

impl InternalObject {
    pub fn new<T: Into<HashMap<String, JsValue>>>(value: T, prototype: Option<JsObjectRef>) -> Self {
        Self {
            properties: value.into(),
            prototype,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct JsObject {
    internal_object: RefCell<InternalObject>,
}

impl JsObject {
    pub fn new<T: Into<HashMap<String, JsValue>>>(value: T, prototype: Option<JsObjectRef>) -> Self {
        Self {
            internal_object: RefCell::new(InternalObject::new(value, prototype)),
        }
    }
}

trait HasInternalObject {
    fn get_internal_object(&self) -> &RefCell<InternalObject>;
}

impl<T: HasInternalObject> Object for T {
    fn set_prototype(&mut self, prototype: JsObjectRef) {
        self.get_internal_object().borrow_mut().set_prototype(prototype);
    }

    fn add_property(&mut self, key: &str, value: JsValue) {
        self.get_internal_object().borrow_mut().add_property(key, value);
    }

    fn get_property_value(&self, key: &str) -> JsValue {
        self.get_internal_object().borrow().get_property_value(key)
    }
}

impl HasInternalObject for JsObject {
    fn get_internal_object(&self) -> &RefCell<InternalObject> {
        &self.internal_object
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

#[derive(Debug, Clone, PartialEq)]
pub enum Func {
    Js(JsFunction),
    Native(NativeFunction),
}

pub trait Callable: Sized {
    fn call(&self, interpreter: &Interpreter, arguments: &Vec<JsValue>) -> Result<JsValue, String>;
}

#[derive(Clone)]
pub struct NativeFunction {
    internal_object: RefCell<InternalObject>,
    pub function: fn(&Interpreter, &Vec<JsValue>) -> Result<JsValue, String>,
}

pub fn create_native_function(function: fn(&Interpreter, &Vec<JsValue>) -> Result<JsValue, String>) -> JsValue {
    JsValue::Function(Func::Native(NativeFunction { function, internal_object: RefCell::new(InternalObject::default()), }))
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("native function")
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.function as usize == other.function as usize
    }
}

impl Callable for NativeFunction {
    fn call(&self, interpreter: &Interpreter, arguments: &Vec<JsValue>) -> Result<JsValue, String> {
        (self.function)(interpreter, arguments)
    }
}

impl Callable for Func {
    fn call(&self, interpreter: &Interpreter, arguments: &Vec<JsValue>) -> Result<JsValue, String> {
        match self {
            Func::Js(function) => function.call(interpreter, arguments),
            Func::Native(function) => function.call(interpreter, arguments)
        }
    }
}

impl Callable for JsFunction {
    fn call(&self, interpreter: &Interpreter, _: &Vec<JsValue>) -> Result<JsValue, String> {
        let result = interpreter.eval_node(self.body.as_ref());
        return result.map(|x| x.unwrap_or(JsValue::Undefined));
    }
}

impl HasInternalObject for JsFunction {
    fn get_internal_object(&self) -> &RefCell<InternalObject> {
        &self.internal_object
    }
}

impl HasInternalObject for NativeFunction {
    fn get_internal_object(&self) -> &RefCell<InternalObject> {
        &self.internal_object
    }
}

impl HasInternalObject for Func {
    fn get_internal_object(&self) -> &RefCell<InternalObject> {
        match self {
            Func::Js(function) => function.get_internal_object(),
            Func::Native(function) => function.get_internal_object(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsFunction {
    internal_object: RefCell<InternalObject>,
    pub arguments: Vec<JsFunctionArg>,
    pub body: Box<AstStatement>,
    pub environment: Box<Environment>,
}

impl JsFunction {
    pub fn new(arguments: Vec<JsFunctionArg>, body: Box<AstStatement>, environment: Box<Environment>) -> Self {
        Self {
            internal_object: RefCell::new(InternalObject::default()),
            arguments,
            body,
            environment,
        }
    }
}

impl Into<JsValue> for JsFunction {
    fn into(self) -> JsValue {
        JsValue::Function(Func::Js(self))
    }
}

pub fn create_empty_function_as_js_value() -> JsValue {
    create_empty_function().into()
}

pub fn create_empty_function() -> JsFunction {
    JsFunction {
        arguments: vec![],
        body: Box::new(AstStatement::BlockStatement(BlockStatementNode { statements: vec![] })),
        environment: Box::new(Environment::default()),
        internal_object: RefCell::new(InternalObject::default()),
    }
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
