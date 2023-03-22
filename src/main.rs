mod interpreter;
mod node;
mod parser;
mod scanner;
use interpreter::{Interpreter, JsValue};
use std::env;

fn eval(code: &str, is_print_ast: bool) {
    let mut parser = parser::Parser::default();
    let ast = parser
        .parse(code)
        .expect(format!("Error occured during parsing").as_str());

    if is_print_ast {
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
    for argument in env::args() {
        println!("{argument}");
    }

    let code = "
    let a = 10;
    while (a) {
        a = a - 1;
        print a;
    }
    ";

//    eval(code, false);
    repl();


//    println!("\x1b[93mError\x1b[0m");
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
            Ok(result) =>  println!("{}", result.unwrap_or(JsValue::Undefined)),
            Err(_) => println!("Error during evaluating node"),
        }
    }
}
