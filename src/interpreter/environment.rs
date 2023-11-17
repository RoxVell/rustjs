use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};
use std::fmt::{Formatter};
use crate::bytecode::bytecode_compiler::GlobalVariable;
use crate::keywords::THIS_KEYWORD;
use crate::value::JsValue;

#[derive(Default, Clone, PartialEq)]
pub struct Environment {
    parent: Option<EnvironmentRef>,
    variables: HashMap<String, (bool, JsValue)>,
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Env").finish()
    }
}

pub type EnvironmentRef = Rc<RefCell<Environment>>;

impl Environment {
    pub fn new(parent: EnvironmentRef) -> Self {
        Self {
            parent: Some(parent),
            variables: HashMap::new(),
        }
    }

    pub fn with_globals(globals: Vec<GlobalVariable>) -> Self {
        let mut variables = HashMap::new();

        globals.into_iter().for_each(|x| {
            variables.insert(x.name, (true, x.value));
        });

        Self {
            parent: None,
            variables
        }
    }

    pub fn with_variables<T: Into<HashMap<String, (bool, JsValue)>>>(variables: T) -> Self {
        Self {
            parent: None,
            variables: variables.into(),
        }
    }

    pub fn print_variables(&self) {
        println!("{:?}", self.variables);
    }

    pub fn get_parent(&self) -> Option<EnvironmentRef> {
        self.parent.as_ref().map(|x| Rc::clone(x))
    }

    pub fn define_variable(&mut self, variable_name: String, value: JsValue, is_const: bool) -> Result<(), String> {
        if self.variables.contains_key(&variable_name) {
            return Err(format!("Variable with name '{variable_name}' already defined"));
        }

        self.variables.insert(variable_name.clone(), (is_const, value.clone()));

        // println!(
        //     "Defined new variable {} = {:#?} Variables: {:#?} Parent: {:#?}",
        //     variable_name, value, self.variables, self.parent
        // );

        return Ok(());
    }

    pub fn set_context(&mut self, value: JsValue) {
        self.define_variable(THIS_KEYWORD.to_string(), value, true).unwrap();
    }

    pub fn get_context(&self) -> JsValue {
        self.get_variable_value(THIS_KEYWORD)
    }

    pub fn assign_variable(&mut self, variable_name: String, value: JsValue) -> Result<(), String> {
        if self.variables.contains_key(&variable_name) {
            let (is_const, _) = self.variables.get(&variable_name).unwrap();

            if *is_const {
                return Err("Assignment to constant variable.".to_string());
            }

            self.variables.insert(variable_name.clone(), (*is_const, value));
            return Ok(());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign_variable(variable_name, value);
        }

        if !self.variables.contains_key(&variable_name) {
            return Err(format!("Variable '{variable_name}' is not defined"));
        }

        return Ok(());
    }

    pub fn get_variable_value(&self, variable_name: &str) -> JsValue {
        if self.variables.contains_key(variable_name) {
            self.variables.get(variable_name)
                .map_or(JsValue::Undefined, |(_, x)| x.clone())
        } else {
            self
                .parent
                .as_ref()
                .map_or(JsValue::Undefined, |parent_env| parent_env.borrow().get_variable_value(variable_name))
        }
    }
}
