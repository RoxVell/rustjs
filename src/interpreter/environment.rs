use super::js_value::JsValue;
use std::collections::HashMap;

pub(super) struct Environment<'a> {
    parent: Option<&'a Environment<'a>>,
    variables: HashMap<String, JsValue>,
}

impl Default for Environment<'static> {
    fn default() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }
}

impl<'a> Environment<'a> {
    pub fn new(parent: Option<&'a Environment<'a>>) -> Self {
        Self {
            parent,
            ..Default::default()
        }
    }

    pub fn define_variable(&mut self, variable_name: String, value: JsValue) -> Result<(), String> {
        if self.variables.contains_key(&variable_name) {
            return Err(format!("Error with name {variable_name} already defined"));
        }

        self.variables.insert(variable_name.clone(), value.clone());

        println!(
            "Defined new variable {} = {:?} Variables: {:?}",
            variable_name, value, self.variables
        );

        return Ok(());
    }

    pub fn get_variable_value(&self, variable_name: String) -> Option<JsValue> {
        println!("get_variable_value: {}", variable_name);
        return self.variables.get(&variable_name).map(|x| x.clone());
    }
}
