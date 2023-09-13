use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use crate::value::function::{JsFunction};
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct JsObject {
    pub kind: ObjectKind,
    pub properties: HashMap<String, JsValue>,
    pub prototype: Option<JsObjectRef>,
}

pub type JsObjectRef = Rc<RefCell<JsObject>>;

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectKind {
    Ordinary,
    Function(JsFunction),
}

impl JsObject {
    pub fn new<T: Into<HashMap<String, JsValue>>>(properties: T, prototype: Option<JsObjectRef>) -> Self {
        Self {
            kind: ObjectKind::Ordinary,
            properties: properties.into(),
            prototype,
        }
    }

    /// Creates an empty object with no properties & no prototype
    pub fn empty() -> Self {
        Self::new([], None)
    }

    pub fn set_prototype(&mut self, prototype: JsObjectRef) {
        self.prototype = Some(prototype);
    }

    pub fn add_property(&mut self, key: &str, value: JsValue) {
        self.properties.insert(key.to_string(), value);
    }

    pub fn get_property_value(&self, key: &str) -> JsValue {
        if self.properties.contains_key(key) {
            return self.properties.get(key).map_or(JsValue::Undefined, |x| x.clone());
        }

        if self.prototype.is_some() {
            return self.prototype.as_ref().unwrap().borrow().get_property_value(key);
        }

        return JsValue::Undefined;
    }

    pub fn is_function(&self) -> bool {
        matches!(self.kind, ObjectKind::Function(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self.kind, ObjectKind::Ordinary)
    }
}

impl Into<JsValue> for JsObject {
    fn into(self) -> JsValue {
        JsValue::Object(Rc::new(RefCell::new(self)))
    }
}
