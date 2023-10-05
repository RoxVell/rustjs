use std::cmp::Ordering;
use crate::bytecode::bytecode_generator::CodeBlock;
use crate::bytecode::opcode::OpCode;
use crate::nodes::JsValue;

pub struct VM {
    ip: usize,
    sp: usize,
    stack: Vec<JsValue>,
    code_block: CodeBlock
}

impl VM {
    pub fn new() -> Self {
        Self {
            ip: 0,
            sp: 0,
            stack: vec![],
            code_block: CodeBlock::new(),
        }
    }

    pub fn eval(&mut self, code_block: CodeBlock) {
        // self.ip = 0;
        self.ip = 0;
        self.code_block = code_block;

        while self.ip < self.code_block.bytecode.len() {
            let opcode: OpCode = self.read_byte().into();
            // println!("{opcode:?}");
            match opcode {
                OpCode::PushLiteral => {
                    let constant_idx = self.read_byte();
                    let value = self.code_block.constants[constant_idx as usize].clone();
                    self.push(value);
                },
                OpCode::PushTrue => {
                    self.push(JsValue::Boolean(true));
                }
                OpCode::PushFalse => self.push(JsValue::Boolean(false)),
                OpCode::Jump => {
                    let jump_address = self.read_byte();
                    self.ip = jump_address as usize;
                },
                OpCode::JumpIfFalse => {
                    let value = self.pop();
                    let jump_address = self.read_byte();

                    if !value.to_bool() {
                        self.ip = jump_address as usize;
                    }
                },
                OpCode::Pop => {},
                OpCode::Halt => {},
                opcode if opcode.is_binary() => {
                    let left_value = self.pop();
                    let right_value = self.pop();

                    let result = match opcode {
                        OpCode::Or => JsValue::Boolean(left_value.to_bool() || right_value.to_bool()),
                        OpCode::And => JsValue::Boolean(left_value.to_bool() && right_value.to_bool()),
                        OpCode::Add => (&left_value + &right_value).expect("Error during adding two values"),
                        OpCode::Sub => (&left_value - &right_value).expect("Error during subbing two values"),
                        OpCode::Mul => (&left_value * &right_value).expect("Error during multiplication two values"),
                        OpCode::Div => (&left_value / &right_value).expect("Error during division two values"),
                        OpCode::Eq => JsValue::Boolean(left_value.eq(&right_value)),
                        OpCode::Neq => JsValue::Boolean(left_value.ne(&right_value)),
                        OpCode::MulMul => left_value.exponentiation(&right_value).expect("Error during exponentiation two values"),
                        opcode if opcode.is_comparison() => {
                            let cmp_result = left_value.partial_cmp(&right_value);

                            if cmp_result.is_none() {
                                panic!(
                                    "Cannot compare value with type \"{}\" and \"{}\"",
                                    left_value.get_type_as_str(),
                                    right_value.get_type_as_str()
                                );
                            }

                            let cmp_result = cmp_result.unwrap();

                            let result = match opcode {
                                OpCode::More => matches!(cmp_result, Ordering::Greater),
                                OpCode::Less => matches!(cmp_result, Ordering::Less),
                                OpCode::LessOrEqual => matches!(cmp_result, Ordering::Less | Ordering::Equal),
                                OpCode::MoreOrEqual => matches!(cmp_result, Ordering::Greater | Ordering::Equal),
                                _ => unreachable!()
                            };

                            JsValue::Boolean(result)
                        },
                        _ => unreachable!()
                    };
                    self.push(result);
                },
                _ => todo!(),
            }
        }
    }

    fn push(&mut self, value: JsValue) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> JsValue {
        self.stack.pop().expect("There is nothing to pop")
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.code_block.bytecode[self.ip];
        self.ip += 1;
        byte
    }
}

pub fn eval_code_block(code_block: CodeBlock) -> Option<JsValue> {
    let mut interpreter = VM::new();
    interpreter.eval(code_block);
    interpreter.stack.last().cloned()
}