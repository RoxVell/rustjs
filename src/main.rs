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
mod cli;
mod source;

use nodes::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};
use ariadne::{Color, Fmt};
use clap::Parser;
use diagnostic::DiagnosticBag;
use crate::bytecode::bytecode_compiler::{BytecodeCompiler, CodeBlock, GlobalVariable};
use crate::bytecode::bytecode_interpreter::VM;
use crate::bytecode::bytecode_printer::BytecodePrinter;
use crate::cli::{Cli, CliCommand, DefaultArgs};
use crate::interpreter::environment::Environment;
use crate::source::{FileSource, InlineSource, Source};
use crate::symbol_checker::symbol_checker::SymbolChecker;

fn get_globals() -> Vec<GlobalVariable> {
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
        // GlobalVariable::new("print".to_string(), JsValue::native_bytecode_function(console_log)),
        GlobalVariable::new("print".to_string(), JsValue::native_function(console_log)),
        GlobalVariable::new("VERSION".to_string(), JsValue::Number(0.1)),
    ]
}

// fn eval_vm(code: &str, is_debug: bool) {
//     if is_debug {
//         println!("-----DEBUG (printing tokens)-----");
//         let mut scanner = scanner::Scanner::new(code.to_string());
//
//         while let Some(token) = scanner.next_token() {
//             println!("{:?}", token);
//         }
//     }
//
//     let parsing_time_start = Instant::now();
//     let ast = parser::Parser::parse_code_to_ast(code)
//         .expect(format!("Error occurred during parsing").as_str());
//     let parsing_time_end = parsing_time_start.elapsed();
//     println!("Parsing is done in {}ms", parsing_time_end.as_millis());
//
//     if is_debug {
//         println!("{:#?}", ast);
//     }
//
//     let diagnostic_bag_ref = Rc::new(RefCell::new(DiagnosticBag::new()));
//     let mut symbol_checker = SymbolChecker::new(code, Rc::clone(&diagnostic_bag_ref));
//     // symbol_checker.check_symbols(&ast);
//
//     for error in &diagnostic_bag_ref.borrow().warnings {
//         error.print_diagnostic();
//     }
//
//     for error in &diagnostic_bag_ref.borrow().errors {
//         error.print_diagnostic();
//     }
//
//     let compilation_errors_count = diagnostic_bag_ref.borrow().errors.len();
//     // let compilation_errors_count = 0;
//
//     if compilation_errors_count != 0 {
//         let compilation_error_message = format!("Compilation failed, found {compilation_errors_count} errors!");
//         println!("{}", compilation_error_message.fg(Color::Red));
//         return ();
//     }
//
//     if compilation_errors_count == 0 {
//         let globals = get_globals();
//
//         let compilation_time_start = Instant::now();
//         let mut bytecode_compiler = BytecodeCompiler::new(globals);
//         bytecode_compiler.compile(&ast);
//         let compilation_time_end = compilation_time_start.elapsed();
//         println!("Compilation is done in {}ms", compilation_time_end.as_millis());
//
//         let code_blocks = &bytecode_compiler.code_blocks;
//
//         let globals = bytecode_compiler.get_globals();
//
//         for code_block in code_blocks {
//             let bytecode_printer = BytecodePrinter::new(code_block, &globals);
//             bytecode_printer.print();
//         }
//
//         let mut interpreter = VM::new(globals);
//
//         println!("\n------- EVAL BEGIN --------");
//         let evaluation_time_start = Instant::now();
//         interpreter.eval(bytecode_compiler.code_blocks.last().unwrap().clone());
//         let evaluation_time_end = evaluation_time_start.elapsed();
//         println!("VM evaluation is done in {}ms", evaluation_time_end.as_millis());
//         println!("-------- EVAL END ---------\n");
//
//         println!("result stack:");
//         println!("{:?}", interpreter.dump_stack());
//
//         println!();
//
//         let result = interpreter.stack.last();
//
//         match result {
//             None => println!("> No Value"),
//             Some(value) => println!("> {}", value),
//         }
//
//         // let evaluation_time_start = Instant::now();
//         // let mut interpreter = Interpreter::default();
//         // let result = interpreter
//         //     .interpret(&ast)
//         //     .expect("Error during evaluating node");
//         // let evaluation_time_end = evaluation_time_start.elapsed();
//         // println!("Ast evaluation is done in {}ms", evaluation_time_end.as_millis());
//         // println!("> {}", result);
//
//     }
// }

// fn eval(code: &str, is_debug: bool) {
//     eval_vm(code, is_debug);
// }

fn main() {
    let cli = Cli::parse();
    let globals = get_globals();

    match &cli.command {
        Some(command) => match command {
            CliCommand::PrintBytecode(options) => {
                let ast = process_default_args(options, &globals);
                process_bytecode_compilation(&globals, &ast, options.time);
            }
            CliCommand::VM(options) => {
                process_bytecode_evaluation(options, &globals);
            }
            CliCommand::Ast(options) => {
                let ast = process_default_args(options, &globals);
                let environment = Environment::with_globals(globals);
                let ast_interpreter = Interpreter::with_environment(environment);
                let result = ast_interpreter.interpret(&ast).expect("Error during evaluating node");
                println!("> {}", result);
            }
        },
        None => {
            repl();
        }
    };
}

fn process_bytecode_evaluation(options: &DefaultArgs, globals: &[GlobalVariable]) {
    let ast = process_default_args(options, globals);
    let code_block = process_bytecode_compilation(&globals, &ast, options.time);
    let mut interpreter = VM::new(&globals);

    println!("------- EVAL BEGIN --------");

    let mut evaluation_time_start: Option<Instant> = None;

    if options.time {
        evaluation_time_start = Some(Instant::now());
    }
    interpreter.eval(code_block);
    if options.time {
        let evaluation_time_end = evaluation_time_start.unwrap().elapsed();
        println!("VM evaluation is done in {}ms", evaluation_time_end.as_millis());
    }
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

fn process_parsing<'a>(source: Source, measure_time: bool) -> (AstStatement, Option<Duration>) {
    let mut parsing_time_start: Option<Instant> = None;

    if measure_time {
        parsing_time_start = Some(Instant::now());
    }

    let ast = parser::Parser::parse_code_to(source)
        .expect(format!("Error occurred during parsing").as_str());

    if measure_time {
        (ast, Some(parsing_time_start.unwrap().elapsed()))
    } else {
        (ast, None)
    }
}

fn process_default_args(options: &DefaultArgs, globals: &[GlobalVariable]) -> AstStatement {
    let mut path = PathBuf::new();
    path.push(&options.filename);
    let source = Source::File(FileSource::new(path));

    if options.debug {
        println!("-----DEBUG (printing tokens)-----");
        let mut scanner = scanner::Scanner::new(source.code().to_string());

        while let Some(token) = scanner.next_token() {
            println!("{:?}", token);
        }
    }

    // TODO: avoid cloning
    let (ast, duration) = process_parsing(source.clone(), options.time);

    if options.time {
        println!("Parsing is done in {}ms", duration.unwrap().as_millis());
    }

    if options.debug {
        println!("{:#?}", ast);
    }

    if !options.ignore_errors || !options.ignore_warnings {
        let diagnostic_bag_ref = Rc::new(RefCell::new(DiagnosticBag::new()));
        let global_names = globals.iter().map(|x| x.name.clone()).collect();
        let mut symbol_checker = SymbolChecker::new(&source, Rc::clone(&diagnostic_bag_ref), global_names);
        symbol_checker.check_symbols(&ast);

        if !options.ignore_warnings {
            for error in &diagnostic_bag_ref.borrow().warnings {
                error.print_diagnostic();
            }
        }

        if !options.ignore_errors {
            for error in &diagnostic_bag_ref.borrow().errors {
                error.print_diagnostic();
            }

            let compilation_errors_count = diagnostic_bag_ref.borrow().errors.len();

            if compilation_errors_count != 0 {
                let compilation_error_message = format!("Compilation failed, found {compilation_errors_count} errors!");
                panic!("{}", compilation_error_message.fg(Color::Red));
            }
        }
    }

    ast
}

fn process_bytecode_compilation(globals: &[GlobalVariable], ast: &AstStatement, measure_time: bool) -> CodeBlock {
    let mut compilation_time_start: Option<Instant> = None;

    if measure_time {
        compilation_time_start = Some(Instant::now());
    }
    let mut bytecode_compiler = BytecodeCompiler::new(globals);
    bytecode_compiler.compile(&ast);
    if measure_time {
        let compilation_time_end = compilation_time_start.unwrap().elapsed();
        println!("Compilation is done in {}ms", compilation_time_end.as_millis());
    }

    let code_blocks = &bytecode_compiler.code_blocks;

    let globals = bytecode_compiler.get_globals();

    for code_block in code_blocks {
        let bytecode_printer = BytecodePrinter::new(code_block, &globals);
        bytecode_printer.print();
    }

    bytecode_compiler.code_blocks.last().unwrap().clone()
}

// fn format_file(file_path: &str) {
//     let source_code = fs::read_to_string(file_path).expect("Should have been able to read the file");
//     let mut parser = Parser::default();
//     let ast = parser.parse(source_code.as_str()).unwrap();
//     println!("{:#?}", ast);
//     let formatted_source = format_ast(&ast, 2);
//     fs::write(file_path, formatted_source).unwrap();
// }

// fn eval_file(file_path: &str) {
//     let source_code = fs::read_to_string(file_path)
//         .expect("Should have been able to read the file");
//     eval(source_code.as_str(), true);
// }

fn repl() {
    let mut parser = parser::Parser::default();
    let interpreter = Interpreter::new();

    let mut line = String::new();

    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout()).expect("flush failed!");

        let mut inline_source = InlineSource::default();
        std::io::stdin().read_line(&mut inline_source.code).unwrap();
        let ast = parser
            .parse(Source::Inline(inline_source))
            .expect(format!("Error occured during parsing").as_str());
        line.clear();

        match interpreter.interpret(&ast) {
            Ok(result) => println!("{}", result),
            Err(e) => println!("\x1b[31mError during evaluating node: {e}\x1b[0m"),
        }

    }
}
