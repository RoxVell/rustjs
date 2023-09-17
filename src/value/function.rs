use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::interpreter::{Environment, EnvironmentRef, Interpreter};
use crate::node::{AstStatement, BlockStatementNode};
use crate::value::JsValue;
use crate::value::object::{JsObject, ObjectKind};

#[derive(Debug, Clone, PartialEq)]
pub enum JsFunction {
    Ordinary(OrdinaryFunction),
    Native(NativeFunction),
}

impl JsFunction {
    pub fn native_function(function: fn(&Interpreter, &Vec<JsValue>) -> Result<JsValue, String>) -> Self {
        Self::Native(NativeFunction { function })
    }

    pub fn ordinary_function(arguments: Vec<JsFunctionArg>, body: Box<AstStatement>, environment: EnvironmentRef) -> Self {
        OrdinaryFunction::new(arguments, body, environment).into()
    }

    pub fn to_object(self) -> JsObject {
        JsObject::new(ObjectKind::Function(self), [])
    }

    pub fn empty() -> Self {
        OrdinaryFunction::empty_function().into()
    }
}

impl Into<JsValue> for JsFunction {
    fn into(self) -> JsValue {
        JsValue::Object(JsObject::new(ObjectKind::Function(self), []).to_ref())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrdinaryFunction {
    pub arguments: Vec<JsFunctionArg>,
    pub body: Box<AstStatement>,
    pub environment: EnvironmentRef,
}

impl OrdinaryFunction {
    pub fn new(arguments: Vec<JsFunctionArg>, body: Box<AstStatement>, environment: EnvironmentRef) -> Self {
        Self {
            arguments,
            body,
            environment,
        }
    }

    pub fn empty_function() -> Self {
        Self {
            arguments: vec![],
            body: Box::new(AstStatement::BlockStatement(BlockStatementNode { statements: vec![] })),
            environment: Rc::new(RefCell::new(Environment::default())),
        }
    }
}

impl Into<JsFunction> for OrdinaryFunction {
    fn into(self) -> JsFunction {
        JsFunction::Ordinary(self)
    }
}

#[derive(Clone, PartialEq)]
pub struct JsFunctionArg {
    pub name: String,
    pub default_value: JsValue,
}

impl Debug for JsFunctionArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Argument '{}' (default = {}))", self.name, self.default_value)
    }
}

impl Callable for OrdinaryFunction {
    fn call(&self, interpreter: &Interpreter, _: &Vec<JsValue>) -> Result<JsValue, String> {
        let result = interpreter.eval_node(self.body.as_ref());
        return result.map(|x| x.unwrap_or(JsValue::Undefined));
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("native function")
    }
}

impl Callable for NativeFunction {
    fn call(&self, interpreter: &Interpreter, arguments: &Vec<JsValue>) -> Result<JsValue, String> {
        (self.function)(interpreter, arguments)
    }
}

impl Callable for JsFunction {
    fn call(&self, interpreter: &Interpreter, arguments: &Vec<JsValue>) -> Result<JsValue, String> {
        match self {
            JsFunction::Ordinary(function) => function.call(interpreter, arguments),
            JsFunction::Native(function) => function.call(interpreter, arguments)
        }
    }
}

pub trait Callable: Sized {
    fn call(&self, interpreter: &Interpreter, arguments: &Vec<JsValue>) -> Result<JsValue, String>;
}

#[derive(Clone, PartialEq)]
pub struct NativeFunction {
    pub function: fn(&Interpreter, &Vec<JsValue>) -> Result<JsValue, String>,
}

// impl Into<JsValue> for OrdinaryFunction {
//     fn into(self) -> JsValue {
//         JsValue::Object(JsObject::JsFunction::Ordinary(self))
//     }
// }
