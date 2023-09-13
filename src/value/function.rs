use std::cell::RefCell;
use std::rc::Rc;
use crate::interpreter::{Environment, Interpreter};
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

    pub fn ordinary_function(arguments: Vec<JsFunctionArg>, body: Box<AstStatement>, environment: Box<Environment>) -> Self {
        OrdinaryFunction::new(arguments, body, environment).into()
    }

    pub fn empty() -> Self {
        OrdinaryFunction::empty_function().into()
    }
}

impl Into<JsValue> for JsFunction {
    fn into(self) -> JsValue {
        JsValue::Object(Rc::new(RefCell::new(
            JsObject { kind: ObjectKind::Function(self), properties: Default::default(), prototype: None }
        )))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrdinaryFunction {
    pub arguments: Vec<JsFunctionArg>,
    pub body: Box<AstStatement>,
    pub environment: Box<Environment>,
}

impl OrdinaryFunction {
    pub fn new(arguments: Vec<JsFunctionArg>, body: Box<AstStatement>, environment: Box<Environment>) -> Self {
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
            environment: Box::new(Environment::default()),
        }
    }
}

impl Into<JsFunction> for OrdinaryFunction {
    fn into(self) -> JsFunction {
        JsFunction::Ordinary(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsFunctionArg {
    pub name: String,
    pub default_value: JsValue,
}

impl Callable for OrdinaryFunction {
    fn call(&self, interpreter: &Interpreter, _: &Vec<JsValue>) -> Result<JsValue, String> {
        let result = interpreter.eval_node(self.body.as_ref());
        return result.map(|x| x.unwrap_or(JsValue::Undefined));
    }
}

impl std::fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
