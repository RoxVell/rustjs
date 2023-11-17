use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use crate::bytecode::bytecode_compiler::CodeBlock;
use crate::bytecode::bytecode_interpreter::VM;
use crate::interpreter::environment::{Environment, EnvironmentRef};
use crate::nodes::{AstStatement, BlockStatementNode, Execute, Interpreter};
use crate::value::JsValue;
use crate::value::object::{JsObject, ObjectKind};

#[derive(Debug, Clone, PartialEq)]
pub enum JsFunction {
    Ordinary(OrdinaryFunction),
    Native(NativeFunction),
    Bytecode(CodeBlock),
}

impl JsFunction {
    pub fn native_function(function: fn(&[JsValue]) -> Result<JsValue, String>) -> Self {
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

impl AstCallable for OrdinaryFunction {
    fn call(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        self.body.as_ref().execute(interpreter)
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("native function")
    }
}

// impl AstCallable for NativeFunction {
//     fn call(&self, interpreter: &Interpreter, arguments: &[JsValue]) -> Result<JsValue, String> {
//         (self.function)(interpreter, arguments)
//     }
// }

// impl AstCallable for JsFunction {
//     fn call(&self, interpreter: &Interpreter, arguments: &[JsValue]) -> Result<JsValue, String> {
//         match self {
//             JsFunction::Ordinary(function) => function.call(interpreter, arguments),
//             JsFunction::Native(function) => function.call(interpreter, arguments),
//             _ => unreachable!(),
//         }
//     }
// }

pub trait AstCallable: Sized {
    fn call(&self, interpreter: &Interpreter) -> Result<JsValue, String>;
}

pub trait NativeCallable: Sized {
    fn call_fn(&self, arguments: &[JsValue]) -> Result<JsValue, String>;
}

impl NativeCallable for NativeFunction {
    fn call_fn(&self, arguments: &[JsValue]) -> Result<JsValue, String> {
        (self.function)(arguments)
    }
}

pub trait VmCallable: Sized {
    fn call(&self, vm: &mut VM, arguments: &[JsValue]);
}

#[derive(Clone, PartialEq)]
pub struct NativeFunction {
    pub function: fn(&[JsValue]) -> Result<JsValue, String>,
}

// impl Into<JsValue> for OrdinaryFunction {
//     fn into(self) -> JsValue {
//         JsValue::Object(JsObject::JsFunction::Ordinary(self))
//     }
// }
