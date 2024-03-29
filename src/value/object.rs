use std::cell::{RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use crate::value::function::{JsFunction};
use crate::value::JsValue;

const PROTOTYPE_PROPERTY: &'static str = "prototype";

#[derive(Debug, Clone, PartialEq)]
pub struct JsObject {
    pub kind: ObjectKind,
    pub properties: HashMap<String, JsValue>,
    __proto__: Option<JsObjectRef>,
}

pub type JsObjectRef = Rc<RefCell<JsObject>>;

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectKind {
    Ordinary,
    Function(JsFunction),
    Array,
}

impl JsObject {
    pub fn new<T: Into<HashMap<String, JsValue>>>(kind: ObjectKind, properties: T) -> Self {
        Self {
            kind,
            properties: properties.into(),
            __proto__: None,
        }
    }

    pub fn to_ref(self) -> JsObjectRef {
        Rc::new(RefCell::new(self))
    }

    /// Creates an empty object with no properties & no prototype
    pub fn empty() -> Self {
        Self::new(ObjectKind::Ordinary, [])
    }

    pub fn empty_ref() -> JsObjectRef {
        Self::new(ObjectKind::Ordinary, []).to_ref()
    }

    pub fn array(properties: Vec<JsValue>) -> Self {
        let properties_with_keys: HashMap<String, JsValue> = properties
            .into_iter()
            .enumerate()
            .map(|(i, x)| (i.to_string(), x)).collect();
        Self::new(ObjectKind::Array, properties_with_keys)
    }

    pub fn set_proto(&mut self, prototype: JsObjectRef) {
        self.__proto__ = Some(prototype);
    }

    pub fn get_proto(&self) -> Option<JsObjectRef> {
        self.__proto__.clone()
    }

    pub fn set_prototype(&mut self, prototype: JsObjectRef) {
        self.add_property(PROTOTYPE_PROPERTY, JsValue::Object(prototype))
    }

    pub fn get_prototype(&self) -> JsValue {
        self.get_property_value(PROTOTYPE_PROPERTY)
    }

    pub fn add_property(&mut self, key: &str, value: JsValue) {
        self.properties.insert(key.to_string(), value);
    }

    pub fn get_property_value(&self, key: &str) -> JsValue {
        if self.properties.contains_key(key) {
            return self.properties.get(key).map_or(JsValue::Undefined, |x| x.clone());
        }

        if self.__proto__.is_some() {
            return self.__proto__.as_ref().unwrap().borrow().get_property_value(key);
        }

        return JsValue::Undefined;
    }

    pub fn is_function(&self) -> bool {
        matches!(self.kind, ObjectKind::Function(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self.kind, ObjectKind::Ordinary)
    }

    pub fn to_js_value(self) -> JsValue {
        JsValue::Object(Rc::new(RefCell::new(self)))
    }
}

impl Into<JsValue> for JsObject {
    fn into(self) -> JsValue {
        self.to_js_value()
    }
}
