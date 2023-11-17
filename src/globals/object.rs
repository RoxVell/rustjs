use crate::nodes::JsValue;
use crate::value::object::JsObject;

pub fn object_object() -> JsValue {
    fn object_keys(args: &[JsValue]) -> Result<JsValue, String> {
        assert_eq!(args.len(), 1);

        if let JsValue::Object(object) = &args[0] {
            let keys: Vec<JsValue> = object
                .borrow()
                .properties
                .keys()
                .map(|x| JsValue::String(x.clone()))
                .collect();
            return Ok(JsValue::Object(JsObject::array(keys).to_ref()));
        }

        return Err("First arguments should be an object".to_string());
    }

    fn object_values(args: &[JsValue]) -> Result<JsValue, String> {
        assert_eq!(args.len(), 1);

        if let JsValue::Object(object) = &args[0] {
            let values: Vec<JsValue> = object
                .borrow()
                .properties
                .values()
                .map(|x| x.clone())
                .collect();
            return Ok(JsValue::Object(JsObject::array(values).to_ref()));
        }

        return Err("First arguments should be an object".to_string());
    }

    fn object_entries(args: &[JsValue]) -> Result<JsValue, String> {
        assert_eq!(args.len(), 1);

        if let JsValue::Object(object) = &args[0] {
            let properties = &object.borrow().properties;
            let values: Vec<JsValue> = properties
                .keys()
                .zip(properties.values())
                .map(|(key, value)| {
                    JsObject::array(vec![JsValue::String(key.clone()), value.clone()]).to_js_value()
                })
                .collect();
            return Ok(JsValue::Object(JsObject::array(values).to_ref()));
        }

        return Err("First arguments should be an object".to_string());
    }

    JsValue::object([
        ("keys".to_string(), JsValue::native_function(object_keys)),
        ("values".to_string(), JsValue::native_function(object_values)),
        ("entries".to_string(), JsValue::native_function(object_entries)),
    ])
}