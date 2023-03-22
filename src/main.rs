mod interpreter;
mod node;
mod parser;
mod scanner;
use interpreter::{Interpreter, JsValue};

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
    let code = "if (0) { print 1; print 2; } else { print \"0 is false\"; }";

    eval(code, true);
//      repl();
}

fn repl() {
    let mut parser = parser::Parser::default();
    let mut interpreter = Interpreter::default();

    let mut line = String::new();

    loop {
        print!("> ");
        let b1 = std::io::stdin().read_line(&mut line).unwrap();
        let ast = parser
            .parse(&line)
            .expect(format!("Error occured during parsing").as_str());
        line.clear();
        let result = interpreter
            .eval_node(&ast)
            .expect("Error during evaluating node");
        println!("Result: {}", result.unwrap_or(JsValue::Undefined));
    }

    //  repl();
}
