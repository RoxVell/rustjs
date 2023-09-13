use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};
use crate::value::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    variables: HashMap<String, JsValue>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }
}

impl Environment {
    pub fn new(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            parent: Some(parent),
            ..Default::default()
        }
    }

    pub fn new_with_variables<T: Into<HashMap<String, JsValue>>>(variables: T) -> Self {
        Self {
            parent: None,
            variables: variables.into()
        }
    }

    pub fn print_variables(&self) {
        println!("{:?}", self.variables);
    }

    pub fn get_parent(&mut self) -> Option<Rc<RefCell<Environment>>> {
        std::mem::replace(&mut self.parent, None)
    }

    pub fn define_variable(&mut self, variable_name: String, value: JsValue) -> Result<(), String> {
        if self.variables.contains_key(&variable_name) {
            return Err(format!("Error with name {variable_name} already defined"));
        }

        self.variables.insert(variable_name.clone(), value.clone());

        // println!(
        //     "Defined new variable {} = {:#?} Variables: {:#?} Parent: {:#?}",
        //     variable_name, value, self.variables, self.parent
        // );

        return Ok(());
    }

    pub fn assign_variable(&mut self, variable_name: String, value: JsValue) -> Result<(), String> {
        if self.variables.contains_key(&variable_name) {
            self.variables.insert(variable_name.clone(), value);
            return Ok(());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow_mut().assign_variable(variable_name, value);
        }

        if !self.variables.contains_key(&variable_name) {
            return Err(format!("Variable \"{variable_name}\" is not defined"));
        }

        // TODO: throw an error while assigning value to constant
        self.variables.insert(variable_name.clone(), value);

        return Ok(());
    }

    pub fn get_variable_value(&self, variable_name: &str) -> JsValue {
//         println!("get_variable_value {} {:#?}", variable_name, self.variables);
        if self.variables.contains_key(variable_name) {
            return self.variables.get(variable_name).map_or(JsValue::Undefined, |x| x.clone());
        } else {
            return self
                .parent
                .as_ref()
                .map_or(JsValue::Undefined, |parent_env| parent_env.borrow().get_variable_value(variable_name));
        }
    }
}
