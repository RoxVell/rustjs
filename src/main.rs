// mod interpreter;
mod node;
mod parser;
mod scanner;
mod value;
mod keywords;
mod visitor;
mod symbol_checker;
mod diagnostic;
mod nodes;
mod bytecode;
mod interpreter;

use nodes::*;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;
use crate::parser::Parser;
use diagnostic::DiagnosticBag;
use crate::bytecode::bytecode_compiler::{BytecodeCompiler, GlobalVariable};
use crate::bytecode::bytecode_interpreter::VM;
use crate::bytecode::bytecode_printer::{BytecodePrinter};
use crate::symbol_checker::symbol_checker::SymbolChecker;

fn get_globals() -> Vec<GlobalVariable> {
    fn console_log(_: &VM, arguments: &[JsValue]) -> Result<JsValue, String> {
        let result = arguments
            .iter()
            .map(|arg| format!("{}", arg))
            .collect::<Vec<String>>()
            .join(" ");
        println!("{result}");
        return Ok(JsValue::Undefined);
    }

    vec![
        GlobalVariable::new("print".to_string(), JsValue::native_bytecode_function(console_log)),
        GlobalVariable::new("VERSION".to_string(), JsValue::Number(0.1)),
    ]
}

fn eval(code: &str, is_debug: bool) {
    if is_debug {
        println!("-----DEBUG (printing tokens)-----");
        let mut scanner = scanner::Scanner::new(code.to_string());

        while let Some(token) = scanner.next_token() {
            println!("{:?}", token);
        }
    }

    let ast = Parser::parse_code_to_ast(code)
        .expect(format!("Error occurred during parsing").as_str());

    if is_debug {
        println!("{:#?}", ast);
    }

    let diagnostic_bag_ref = Rc::new(RefCell::new(DiagnosticBag::new()));
    let mut symbol_checker = SymbolChecker::new(code, Rc::clone(&diagnostic_bag_ref));
    symbol_checker.check_symbols(&ast);

    for error in &diagnostic_bag_ref.borrow().warnings {
        error.print_diagnostic();
    }

    for error in &diagnostic_bag_ref.borrow().errors {
        error.print_diagnostic();
    }

    println!("{}", diagnostic_bag_ref.borrow().errors.len());

    if diagnostic_bag_ref.borrow().errors.len() == 0 {
        let globals = get_globals();

        let mut bytecode_compiler = BytecodeCompiler::new(globals);
        bytecode_compiler.compile(&ast);

        let code_blocks = &bytecode_compiler.code_blocks;

        let globals = bytecode_compiler.get_globals();

        for code_block in code_blocks {
            let bytecode_printer = BytecodePrinter::new(code_block, &globals);
            bytecode_printer.print();
        }

        let mut interpreter = VM::new(globals);

        println!("\n------- EVAL BEGIN --------");
        interpreter.eval(bytecode_compiler.code_blocks.last().unwrap().clone());
        println!("-------- EVAL END ---------\n");

        println!("result stack:");
        println!("{:?}", interpreter.dump_stack());

        println!();

        let result = interpreter.stack.last();

        match result {
            None => println!("> No Value"),
            Some(value) => println!("> {}", value),
        }
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

// fn format_file(file_path: &str) {
//     let source_code = fs::read_to_string(file_path).expect("Should have been able to read the file");
//     let mut parser = Parser::default();
//     let ast = parser.parse(source_code.as_str()).unwrap();
//     println!("{:#?}", ast);
//     let formatted_source = format_ast(&ast, 2);
//     fs::write(file_path, formatted_source).unwrap();
// }

fn eval_file(file_path: &str) {
    let source_code = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");
    eval(source_code.as_str(), false);
}

fn repl() {
    let mut parser = Parser::default();
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

        match interpreter.interpret(&ast) {
            Ok(result) => println!("{}", result),
            Err(e) => println!("\x1b[31mError during evaluating node: {e}\x1b[0m"),
        }
    }
}
