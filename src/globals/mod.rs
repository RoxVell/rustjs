use crate::bytecode::bytecode_compiler::GlobalVariable;
use crate::globals::math::math_object;
use crate::globals::object::object_object;
use crate::value::JsValue;

mod math;
mod object;

pub fn get_globals() -> Vec<GlobalVariable> {
    fn console_log(arguments: &[JsValue]) -> Result<JsValue, String> {
        let result = arguments
            .iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<String>>()
            .join(" ");
        println!("{result}");
        return Ok(JsValue::Undefined);
    }

    vec![
        GlobalVariable::new("print", JsValue::native_function(console_log)),
        GlobalVariable::new("Math", math_object()),
        GlobalVariable::new("Object", object_object()),
    ]
}
