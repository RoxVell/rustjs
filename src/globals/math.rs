use crate::value::JsValue;

pub fn math_object() -> JsValue {
    macro_rules! math_func {
        ($name:ident) => {
            fn $name(args: &[JsValue]) -> Result<JsValue, String> {
                assert_eq!(args.len(), 1);

                if let JsValue::Number(number) = &args[0] {
                    return Ok(JsValue::Number(number.$name()));
                }

                return Err("First arguments should be a number".to_string());
            }
        }
    }

    math_func!(cos);
    math_func!(sin);
    math_func!(round);
    math_func!(abs);
    math_func!(ceil);
    math_func!(floor);

    JsValue::object([
        ("round".to_string(), JsValue::native_function(round)),
        ("abs".to_string(), JsValue::native_function(abs)),
        ("ceil".to_string(), JsValue::native_function(ceil)),
        ("floor".to_string(), JsValue::native_function(floor)),
        ("sin".to_string(), JsValue::native_function(sin)),
        ("cos".to_string(), JsValue::native_function(cos)),
        ("PI".to_string(), JsValue::Number(std::f64::consts::PI)),
    ])
}
