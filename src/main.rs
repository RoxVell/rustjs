mod scanner;
mod node_type;
mod parser;
mod interpreter;
use interpreter::Interpreter;

fn eval(code: &str, is_print_ast: bool) {
  let mut parser = parser::Parser::default();
  let ast = parser.parse(code).expect(format!("Error occured during parsing").as_str());

  if is_print_ast {
    println!("{ast:#?}");
  }

  let mut interpreter = Interpreter::default();
  let result = interpreter.eval_node(&ast).expect("Error during evaluating node");

  match result {
    None => println!("No Value"),
    Some(value) => println!("> {}", value),
  }
}

fn main() {
  let code = "
  let a = 2 + 3 * 2 / (1 + 1);
  let b = a * 2;
  b;
  ";

  eval(code, true);
}

