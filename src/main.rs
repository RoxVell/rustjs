mod interpreter;
mod node;
mod parser;
mod scanner;
use crate::node::{format_ast};
use interpreter::*;
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
        println!("{:#?}", ast);
    }

    let interpreter = Interpreter::default();
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
        // format_file(&path.unwrap());
    } else {
        repl();
    }
}

fn format_file(file_path: &str) {
    let source_code =
        fs::read_to_string(file_path).expect("Should have been able to read the file");
    let mut parser = parser::Parser::default();
    let ast = parser.parse(source_code.as_str()).unwrap();
    println!("{:#?}", ast);
    let formatted_source = format_ast(&ast, 2);
    fs::write(file_path, formatted_source).unwrap();
}

fn eval_file(file_path: &str) {
    let source_code = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
    eval(source_code.as_str(), false);
}

fn repl() {
    let mut parser = parser::Parser::default();
    let interpreter = Interpreter::default();

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

// fn eval_code(code: &str) -> JsValue {
//     let interpreter = Interpreter::default();
//
//     let ast = parser::Parser::parse_code_to_ast(code)
//         .expect(format!("Error occured during parsing").as_str());
//
//     interpreter.eval_node(&ast).unwrap().unwrap()
// }

// fn interpret(interpreter: &mut Interpreter, code: &str) -> JsValue {
//     let ast = parser::Parser::parse_code_to_ast(code)
//         .expect(format!("Error occured during parsing").as_str());
//
//     interpreter.eval_node(&ast).unwrap().unwrap()
// }
