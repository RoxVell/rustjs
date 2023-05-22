use super::js_value::JsValue;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    parent: Option<Box<Environment>>,
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
    pub fn new(parent: Box<Environment>) -> Self {
        Self {
            parent: Some(parent),
            ..Default::default()
        }
    }

    pub fn get_parent(&mut self) -> Option<Environment> {
        std::mem::replace(&mut self.parent, None).map(|x| *x)
//        self.parent.map(|x| *x)
    }

    pub fn define_variable(&mut self, variable_name: String, value: JsValue) -> Result<(), String> {
        if self.variables.contains_key(&variable_name) {
            return Err(format!("Error with name {variable_name} already defined"));
        }

        self.variables.insert(variable_name.clone(), value.clone());

        //        println!(
        //            "Defined new variable {} = {:?} Variables: {:?}",
        //            variable_name, value, self.variables
        //        );

        return Ok(());
    }

    pub fn assign_variable(&mut self, variable_name: String, value: JsValue) -> Result<(), String> {
        if !self.variables.contains_key(&variable_name) {
            return Err(format!("Error with name {variable_name} is not defined"));
        }

        // TODO: throw an error while assigning value to constant
        self.variables.insert(variable_name.clone(), value);

        return Ok(());
    }

    pub fn get_variable_value(&self, variable_name: &str) -> Option<JsValue> {
//        println!("{:#?}", self.variables);
        if self.variables.contains_key(variable_name) {
            return self.variables.get(variable_name).map(|x| x.clone());
        } else {
            return self.parent.as_ref().map(|parent_env| parent_env.get_variable_value(variable_name))?;
        }

    }
}
