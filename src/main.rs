mod interpreter;
mod node;
mod parser;
mod scanner;
use interpreter::{create_js_object, Environment, Interpreter, JsObject, JsValue};
use std::env;
use std::fs;

fn eval(code: &str, is_debug: bool) {
    println!("-----DEBUG (printing tokens)-----");

    if is_debug {
        let mut scanner = scanner::Scanner::new(code.to_string());

        while let Some(token) = scanner.next_token() {
            println!("{:?}", token);
        }
    }

    println!("---------");

    let mut parser = parser::Parser::default();
    let ast = parser
        .parse(code)
        .expect(format!("Error occured during parsing").as_str());

    if is_debug {
        println!("{ast:#?}");
    }

    let mut interpreter = Interpreter::default();
    let result = interpreter
        .eval_node(&ast)
        .expect("Error during evaluating node");

    match result {
        None => println!("No Value"),
        Some(value) => println!("> {}", value),
    }
}

fn main() {
    let path = std::env::args().nth(1);

    if path.is_some() {
        eval_file(&path.unwrap());
    } else {
        repl();
    }
}

fn eval_file(file_path: &str) {
    let source_code =
        fs::read_to_string(file_path).expect("Should have been able to read the file");
    eval(source_code.as_str(), true);
}

fn repl() {
    let mut parser = parser::Parser::default();
    let mut interpreter = Interpreter::default();

    let mut line = String::new();

    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout()).expect("flush failed!");
        std::io::stdin().read_line(&mut line).unwrap();
        let ast = parser
            .parse(&line)
            .expect(format!("Error occured during parsing").as_str());
        line.clear();

        match interpreter.eval_node(&ast) {
            Ok(result) => println!("{}", result.unwrap_or(JsValue::Undefined)),
            Err(e) => println!("\x1b[31mError during evaluating node: {e}\x1b[0m"),
        }
    }
}

fn eval_code(code: &str) -> JsValue {
    let mut interpreter = Interpreter::default();

    let ast = parser::Parser::parse_code_to_ast(code)
        .expect(format!("Error occured during parsing").as_str());

    interpreter.eval_node(&ast).unwrap().unwrap()
}

use std::rc::Rc;

#[test]
fn get_variable_value_from_parent_environment() {
    let variable_name = "abc";
    let variable_value = JsValue::Number(123.0);

    let mut parent_env = Environment::default();
    parent_env.define_variable(variable_name.to_string(), variable_value.clone());

    let child_env = Environment::new(Box::new(parent_env));
    let value_from_parent_env = child_env.get_variable_value(variable_name).unwrap();

    assert_eq!(value_from_parent_env, variable_value);
}

#[test]
fn try_to_get_undefined_variable_from_environment() {
    let env = Environment::default();
    assert_eq!(env.get_variable_value("abc"), None);
}

#[test]
fn add_operator_works() {
    let code = "2 + 2;";
    assert_eq!(eval_code(code), JsValue::Number(4.0));
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

    a;";

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

    a;";

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
    let code = "let a = { a: 10 }; a;";
    let expected = create_js_object(JsObject::new_with_properties([(
        "a".to_string(),
        JsValue::Number(10.0),
    )]));
    assert_eq!(eval_code(code), expected);
}

#[test]
fn member_expression_works() {
    let code = "let a = { b: 10 }; a.b;";
    assert_eq!(eval_code(code), JsValue::Number(10.0));
}

#[test]
fn nested_member_expression_works() {
    let code = "let a = {
        b: {
            c: {
                d: \"qwerty\"
            }
        }
    };
    a.b.c.d;";
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
