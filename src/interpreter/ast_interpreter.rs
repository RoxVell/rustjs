use std::cell::RefCell;
use std::rc::Rc;
use crate::interpreter::environment::{Environment, EnvironmentRef};
use crate::nodes::{AstExpression, AstStatement, Execute, FunctionArgument};
use crate::{Source};
use crate::value::function::{AstCallable, JsFunction, JsFunctionArg, NativeCallable};
use crate::value::JsValue;
use crate::value::object::{JsObject, ObjectKind};

pub struct Interpreter {
    pub environment: RefCell<EnvironmentRef>,
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = get_global_environment();
        Self {
            environment: RefCell::new(Rc::new(RefCell::new(environment))),
        }
    }

    pub fn with_environment(environment: Environment) -> Self {
        Self {
            environment: RefCell::new(Rc::new(RefCell::new(environment))),
        }
    }

    pub fn interpret(&self, statement: &AstStatement) -> Result<JsValue, String> {
        statement.execute(self)
    }

    pub fn set_environment(&self, environment: Environment) {
        self.environment.replace(Rc::new(RefCell::new(environment)));
    }

    pub(crate) fn create_new_environment(&self) -> Environment {
        return Environment::new(Rc::clone(&self.environment.borrow().clone()));
    }

    pub(crate) fn pop_environment(&self) {
        let parent_environment = self
            .environment
            .borrow()
            .borrow()
            .get_parent()
            .unwrap()
            .borrow()
            .to_owned();

        self.set_environment(parent_environment);
    }

    pub(crate) fn logical_or(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        if left.to_bool() {
            return Ok(left.clone());
        }
        return Ok(right.clone());
    }

    pub(crate) fn logical_and(&self, left: &JsValue, right: &JsValue) -> Result<JsValue, String> {
        if !left.to_bool() {
            return Ok(left.clone());
        }
        return Ok(right.clone());
    }

    pub(crate) fn call_function(&self, callee: &AstExpression, arguments: &Vec<AstExpression>, is_new: bool) -> Result<JsValue, String> {
        // println!("call_function {callee:?}");
        let calleer = callee.execute(self)?;

        // println!("calleer {calleer:?}");

        if let JsValue::Object(object) = &calleer {
            if let ObjectKind::Function(function) = &object.borrow().kind {
                let mut function_execution_environment = self.create_new_environment();

                // println!("expr {callee:?}");

                if let AstExpression::MemberExpression(expr) = &callee {
                    function_execution_environment.set_context(expr.object.execute(self)?);
                }

                // TODO: refactor, ugly as hell
                if is_new {
                    function_execution_environment.set_context(JsObject::empty().into());
                }

                let values: Vec<JsValue> = arguments
                    .iter()
                    .map(|param| param.execute(self).unwrap())
                    .collect();

                match function {
                    JsFunction::Ordinary(function) => {
                        function
                            .arguments
                            .iter()
                            .zip(arguments)
                            .for_each(|(arg, node)| {
                                let value = node.execute(self).unwrap();

                                function_execution_environment
                                    .define_variable(arg.name.clone(), value, false)
                                    .unwrap();
                            });
                        self.set_environment(function_execution_environment);
                        let result = function.call(self).unwrap();

                        if let JsValue::Object(result_object) = &result {
                            let proto = object.borrow().get_prototype();

                            if let JsValue::Object(object) = proto {
                                result_object.borrow_mut().set_proto(object);
                            }
                        }

                        // println!("{result:?}");
                        self.pop_environment();
                        return Ok(result);
                    }
                    JsFunction::Native(function) => {
                        self.set_environment(function_execution_environment);
                        let result = function.call_fn(&values);
                        self.pop_environment();
                        return result;
                    }
                    _ => unreachable!()
                }
            }
        }

        Err(format!("{} is not callable", calleer.get_type_as_str()))
    }

    pub(crate) fn create_js_function(
        &self,
        function_arguments: &Vec<FunctionArgument>,
        body: AstStatement,
    ) -> JsFunction {
        let mut arguments = Vec::with_capacity(function_arguments.len());

        for fn_arg_node in function_arguments {
            let default_value = fn_arg_node
                .default_value
                .as_ref()
                .map(|node| node.execute(self).unwrap())
                .unwrap_or(JsValue::Undefined);

            arguments.push(JsFunctionArg {
                name: fn_arg_node.name.id.clone(),
                default_value,
            });
        }

        JsFunction::ordinary_function(
            arguments,
            Box::new(body.clone()),
            self.environment.borrow().clone()
        )
    }

    pub(crate) fn eval_member_expression_key(
        &self,
        node: &AstExpression,
        computed: bool,
    ) -> Result<String, String> {
        if computed {
            let computed_key = node.execute(self)?;

            return match computed_key {
                JsValue::String(value) => Ok(value),
                JsValue::Number(value) => Ok(value.to_string()),
                _ => Err("".to_string()),
            };
        } else {
            return match node {
                AstExpression::StringLiteral(value) => Ok(value.value.clone()),
                AstExpression::NumberLiteral(node) => Ok(node.value.to_string()),
                AstExpression::Identifier(node) => Ok(node.id.clone()),
                _ => Err("Object key should be an identifier".to_string()),
            };
        }
    }
}

fn get_global_environment() -> Environment {
    fn console_log(arguments: &[JsValue]) -> Result<JsValue, String> {
        let result = arguments
            .iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<String>>()
            .join(" ");
        println!("{result}");
        return Ok(JsValue::Undefined);
    }

    fn set_prototype(arguments: &[JsValue]) -> Result<JsValue, String> {
        let target = arguments
            .get(0)
            .expect("Expected first argument to be a target");

        if let JsValue::Object(target_obj) = target {
            let prototype = arguments
                .get(1)
                .expect("Expected second argument to be a prototype object");

            if let JsValue::Object(prototype_obj) = prototype {
                target_obj
                    .borrow_mut()
                    .set_proto(prototype_obj.clone());
            } else {
                return Err(format!(
                    "Second arguments should be of type object, but got: {}",
                    target.get_type_as_str()
                ));
            }
        } else {
            return Err(format!(
                "First arguments should be of type object, but got: {}",
                target.get_type_as_str()
            ));
        }

        return Ok(JsValue::Undefined);
    }

    fn performance_now(_: &[JsValue]) -> Result<JsValue, String> {
        return Ok(JsValue::Number(
            std::time::SystemTime::now()
                .duration_since( std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as f64,
        ));
    }

    fn object_keys(args: &[JsValue]) -> Result<JsValue, String> {
        assert_eq!(args.len(), 1);

        if let JsValue::Object(object) = &args[0] {
            let keys: Vec<JsValue> = object.borrow().properties.keys().map(|x| JsValue::String(x.clone())).collect();
            return Ok(JsValue::Object(JsObject::array(keys).to_ref()));
        }

        return Err("First arguments should be an object".to_string());
    }

    fn object_values(args: &[JsValue]) -> Result<JsValue, String> {
        assert_eq!(args.len(), 1);

        if let JsValue::Object(object) = &args[0] {
            let values: Vec<JsValue> = object.borrow().properties.values().map(|x| x.clone()).collect();
            return Ok(JsValue::Object(JsObject::array(values).to_ref()));
        }

        return Err("First arguments should be an object".to_string());
    }

    fn object_entries(args: &[JsValue]) -> Result<JsValue, String> {
        assert_eq!(args.len(), 1);

        if let JsValue::Object(object) = &args[0] {
            let properties = &object.borrow().properties;
            let values: Vec<JsValue> = properties.keys()
                .zip(properties.values())
                .map(|(key, value)| {
                    JsObject::array(vec![JsValue::String(key.clone()), value.clone()]).to_js_value()
                })
                .collect();
            return Ok(JsValue::Object(JsObject::array(values).to_ref()));
        }

        return Err("First arguments should be an object".to_string());
    }

    Environment::with_variables([
        (
            "console".to_string(),
            (true, JsValue::object([
                ("log".to_string(), JsValue::native_function(console_log)),
            ])),
        ),
        ("print".to_string(), (true, JsValue::native_function(console_log))),
        (
            "setPrototypeOf".to_string(),
            (true, JsValue::native_function(set_prototype),)
        ),
        (
            "performance".to_string(),
            (true, JsValue::object([
                ("now".to_string(), JsValue::native_function(performance_now))
            ]),)
        ),
        (
            "Object".to_string(),
            (true, JsValue::object([
                ("keys".to_string(), JsValue::native_function(object_keys)),
                ("values".to_string(), JsValue::native_function(object_values)),
                ("entries".to_string(), JsValue::native_function(object_entries)),
            ])),
        )
    ])
}

fn interpret(interpreter: &mut Interpreter, code: &str) -> JsValue {
    let source = Source::inline_source(code.to_string());

    let ast = crate::parser::Parser::parse_code_to(source)
        .expect(format!("Error occurred during parsing").as_str());

    interpreter.interpret(&ast).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    pub fn eval_code(code: &str) -> JsValue {
        let source = Source::inline_source(code.to_string());
        let interpreter = Interpreter::new();

        let ast = crate::parser::Parser::parse_code_to(source)
            .expect(format!("Error occurred during parsing").as_str());

        interpreter.interpret(&ast).unwrap()
    }

    #[test]
    fn get_variable_value_from_parent_environment() {
        let variable_name = "abc";
        let variable_value = JsValue::Number(123.0);

        let mut parent_env = Environment::default();
        parent_env.define_variable(variable_name.to_string(), variable_value.clone(), false).unwrap();

        let child_env = Environment::new(Rc::new(RefCell::new(parent_env)));
        let value_from_parent_env = child_env.get_variable_value(variable_name);

        assert_eq!(value_from_parent_env, variable_value);
    }

    #[test]
    fn try_to_get_undefined_variable_from_environment() {
        let env = Environment::default();
        assert_eq!(env.get_variable_value("abc"), JsValue::Undefined);
    }

    #[test]
    fn add_operator_works() {
        let code = "2 + 2;";
        assert_eq!(eval_code(code), JsValue::Number(4.0));

        let code = "'Hello ' + 'world!';";
        assert_eq!(eval_code(code), JsValue::String("Hello world!".to_string()));
    }

    #[test]
    fn if_operator_works_then_branch() {
        let code = "let a; if (true) { a = 5; } else { a = 10; } a;";
        assert_eq!(eval_code(code), JsValue::Number(5.0));
    }

    #[test]
    fn if_operator_works_else_branch() {
        let code = "let a; if (false) { a = 5; } else { a = 10; } a;";
        assert_eq!(eval_code(code), JsValue::Number(10.0));
    }

    #[test]
    fn for_loop_works() {
        let code = "
            let a = 5;

            for (let i = 1; i < 11; i+=1) {
              a *= i;
            }

            a;
        ";

        assert_eq!(eval_code(code), JsValue::Number(18144000.0));
    }

    #[test]
    fn while_loop_works() {
        let code = "
            let a = 0;
            let i = 10;

            while (i > 0) {
                a += i;
                i -=1 ;
            }

            a;
        ";

        assert_eq!(eval_code(code), JsValue::Number(55.0));
    }

    #[test]
    fn equality_expression_equal_works() {
        let code = "5 == 5";
        assert_eq!(eval_code(code), JsValue::Boolean(true));
    }

    #[test]
    fn equality_expression_not_equal_works() {
        let code = "5 == 6";
        assert_eq!(eval_code(code), JsValue::Boolean(false));
    }

    #[test]
    fn inequality_expression_equal_works() {
        let code = "5 != 5";
        assert_eq!(eval_code(code), JsValue::Boolean(false));
    }

    #[test]
    fn inequality_expression_not_equal_works() {
        let code = "5 != 6";
        assert_eq!(eval_code(code), JsValue::Boolean(true));
    }

    #[test]
    fn conditional_expression_equal_works() {
        let code = "true ? 1 : 2;";
        assert_eq!(eval_code(code), JsValue::Number(1.0));
    }

    #[test]
    fn conditional_expression_not_equal_works() {
        let code = "false ? 1 : 2;";
        assert_eq!(eval_code(code), JsValue::Number(2.0));
    }

    #[test]
    fn object_expression_works() {
        let code = "
            let a = {
                5: 2 + 3,
                'qwe-123': 'string prop',
                abc: 'identifier prop',
                ['hello ' + 123]: 'hello 123',
            };

            a;
        ";

        let mut interpreter = Interpreter::new();

        let expected = JsValue::object([
            ("5".to_string(), JsValue::Number(5.0)),
            (
                "qwe-123".to_string(),
                JsValue::String("string prop".to_string()),
            ),
            (
                "abc".to_string(),
                JsValue::String("identifier prop".to_string()),
            ),
            (
                "hello 123".to_string(),
                JsValue::String("hello 123".to_string()),
            ),
        ]);

        assert_eq!(interpret(&mut interpreter, code), expected);
        assert_eq!(interpret(&mut interpreter, "a[5];"), JsValue::Number(5.0));
        assert_eq!(
            interpret(&mut interpreter, "a['qwe-123'];"),
            JsValue::String("string prop".to_string())
        );
        assert_eq!(
            interpret(&mut interpreter, "a['abc'];"),
            JsValue::String("identifier prop".to_string())
        );
        assert_eq!(
            interpret(&mut interpreter, "a.abc;"),
            JsValue::String("identifier prop".to_string())
        );
        assert_eq!(
            interpret(&mut interpreter, "a['hello ' + 123];"),
            JsValue::String("hello 123".to_string())
        );
    }

    #[test]
    fn object_function_property() {
        let code = "
            let a = {
                b: function(a,b) {
                    return a * 2 + b;
                }
            };

            a.b(3, 2);
        ";
        assert_eq!(eval_code(code), JsValue::Number(8.0));
    }

    #[test]
    fn nested_member_expression_works() {
        let code = "
            let a = {
                b: {
                    c: {
                        d: 'qwerty'
                    }
                }
            };
            a.b.c.d;
        ";
        assert_eq!(eval_code(code), JsValue::String("qwerty".to_string()));
    }

    #[test]
    fn assign_to_object_property_works() {
        let code = "
            let a = { b: 10 };
            a.b = 20;
            a.b;
        ";
        assert_eq!(eval_code(code), JsValue::Number(20.0));
    }

    #[test]
    fn mutate_object_as_reference_works() {
        let code = "
            let a = { b: 10 };
            let c = { d: a };
            a.b = 25;
            c.d.b;
        ";
        assert_eq!(eval_code(code), JsValue::Number(25.0));
    }

    #[test]
    fn object_method_this_expression() {
        let mut interpreter = Interpreter::new();

        let code = "
            let a = {
                abc: 10,
                getAbc: function(a, b) {
                    return this.abc;
                },
                setAbc: function(newValue) {
                    this.abc = newValue;
                }
            };

            a.getAbc();
        ";
        assert_eq!(interpret(&mut interpreter, code), JsValue::Number(10.0));
        assert_eq!(
            interpret(&mut interpreter, "a.setAbc(25); a.getAbc();"),
            JsValue::Number(25.0)
        );
    }

    #[test]
    fn comparison() {
        let mut interpreter = Interpreter::new();

        assert_eq!(interpret(&mut interpreter, "'abc' == 'abc'"), JsValue::Boolean(true));
        assert_eq!(interpret(&mut interpreter, "'abc' == 'qwe'"), JsValue::Boolean(false));
        assert_eq!(interpret(&mut interpreter, "123 == 123"), JsValue::Boolean(true));
        assert_eq!(interpret(&mut interpreter, "123 == 456"), JsValue::Boolean(false));
        assert_eq!(interpret(&mut interpreter, "true == true"), JsValue::Boolean(true));
        assert_eq!(interpret(&mut interpreter, "true == false"), JsValue::Boolean(false));
        assert_eq!(interpret(&mut interpreter, "false == false"), JsValue::Boolean(true));
        assert_eq!(eval_code("let a = {}; let b = {}; a == b;"), JsValue::Boolean(false));
        assert_eq!(eval_code("let a = {}; let b = a; a == b;"), JsValue::Boolean(true));
    }

    #[test]
    fn prototype_property_access() {
        let mut interpreter = Interpreter::new();

        let code = "
            let prototype = {
                a: 10
            };

            let target = { b: 30 };
            setPrototypeOf(target, prototype);
            target.a;
        ";
        assert_eq!(interpret(&mut interpreter, code), JsValue::Number(10.0));
    }

    #[test]
    fn prototype_mutable_property_access() {
        let mut interpreter = Interpreter::new();

        let code = "
            let prototype = {
                a: 10
            };

            let target = { b: 30 };
            setPrototypeOf(target, prototype);
            prototype.a = 50;
            target.a;
        ";
        assert_eq!(interpret(&mut interpreter, code), JsValue::Number(50.0));
    }

    #[test]
    fn two_objects_must_be_checked_for_equality_by_reference() {
        let code = "
            let a = { b: { c: 10 } };

            let d = {
                e: {
                    f: a
                }
            };

            d.e.f == a;
        ";
        assert_eq!(eval_code(code), JsValue::Boolean(true));

        let code = "
            let a = { b: { c: 10 } };

            let d = {
                e: {
                    f: {}
                }
            };

            d.e.f == a;
        ";
        assert_eq!(eval_code(code), JsValue::Boolean(false));
    }

    #[test]
    fn simple_class_usage() {
        let code = "
            class User {
                constructor(name, age) {
                    this.name = name;
                    this.age = age;
                }

                getUserInformation() {
                    return 'Name is ' + this.name + ', ' + this.age + ' years old';
                }
            }

            let user = new User('Anton', 26);
            user.getUserInformation();
        ";
        assert_eq!(eval_code(code), JsValue::String("Name is Anton, 26 years old".to_string()));
    }

    #[test]
    fn class_proto_of_instance_should_be_equal_to_class_prototype() {
        let mut interpreter = Interpreter::new();

        let code = "
            class User {
                constructor(name, age) {
                    this.name = name;
                    this.age = age;
                }

                getUserInformation() {
                    return 'Name is ' + this.name + ', ' + this.age + ' years old';
                }
            }

            let user = new User('Anton', 26);
            user.getUserInformation();
        ";
        interpret(&mut interpreter, code);
        let class = interpreter.environment.borrow().borrow().get_variable_value("User");
        let class_instance = interpreter.environment.borrow().borrow().get_variable_value("user");

        if let JsValue::Object(class_object) = &class {
            if let JsValue::Object(instance_object) = &class_instance {
                let class_prototype = class_object.borrow().get_prototype();
                let class_instance_proto = instance_object.borrow().get_proto().unwrap();

                if let JsValue::Object(class_prototype) = class_prototype {
                    assert!(Rc::ptr_eq(&class_prototype, &class_instance_proto));
                }
            }
        }
    }

    #[test]
    fn prototypes_of_instances_of_same_class_equals() {
        let mut interpreter = Interpreter::new();
        let code = "
            class A { constructor(a) { this.a = a; } }
            new A();
        ";
        let class_instance1 = interpret(&mut interpreter, code);
        let class_instance2 = interpret(&mut interpreter, "new A();");

        if let JsValue::Object(object1) = &class_instance1 {
            if let JsValue::Object(object2) = &class_instance2 {
                let prototype1 = object1.borrow().get_proto().unwrap();
                let prototype2 = object2.borrow().get_proto().unwrap();
                assert!(Rc::ptr_eq(&prototype1, &prototype2));
            }
        }
    }

    #[test]
    fn function_constructor_as_class() {
        let code = "
            function User(name, age) {
                this.name = name;
                this.age = age;
            }

            console.log(User.prototype);

            User.prototype.getUserInformation = function() {
                return 'Name is ' + this.name + ', ' + this.age + ' years old';
            }

            let user = new User('Anton', 26);
            user.getUserInformation();
        ";
        assert_eq!(eval_code(code), JsValue::String("Name is Anton, 26 years old".to_string()));
    }

    #[test]
    #[should_panic(expected = "Assignment to constant variable.")]
    fn attempt_to_reassign_constant_variable_should_error() {
        let code = "
          const a = 5;
          a = 10;
        ";
        eval_code(code);
    }

    #[test]
    fn template_strings_works() {
        let code = "
          const a = 5;
          const b = 'Hello';
          const c = 'World';
          `-- before ${a + 2} ${b} ${c} after--`
        ";
        assert_eq!(eval_code(code), JsValue::String("-- before 7 Hello World after--".to_string()));
    }

    #[test]
    fn negative_number_works() {
        assert_eq!(eval_code("-1"), JsValue::Number(-1.0));
        assert_eq!(eval_code("-(-1)"), JsValue::Number(1.0));
    }

    #[test]
    fn logical_not_unary_operator_works_with_boolean() {
        assert_eq!(eval_code("!true"), JsValue::Boolean(false));
        assert_eq!(eval_code("!false"), JsValue::Boolean(true));
        assert_eq!(eval_code("!!true"), JsValue::Boolean(true));
        assert_eq!(eval_code("!!false"), JsValue::Boolean(false));
    }
}
