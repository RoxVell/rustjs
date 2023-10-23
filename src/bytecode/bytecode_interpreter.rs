use std::cmp::Ordering;
use crate::bytecode::bytecode_compiler::{CodeBlock, GlobalVariable};
use crate::bytecode::opcodes::Opcode;
use crate::nodes::JsValue;
use crate::value::function::{JsFunction, VmCallable};
use crate::value::object::{ObjectKind};

#[derive(Debug)]
pub struct CallFrame {
    pub ip: usize,
    pub bp: usize,
    pub code_block: CodeBlock,
}

impl CallFrame {
    pub fn new(bp: usize, code_block: CodeBlock) -> Self {
        Self {
            ip: 0,
            bp,
            code_block,
        }
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.ip = ip;
    }

    pub fn set_bp(&mut self, bp: usize) {
        self.bp = bp;
    }
}

pub struct VM<'a> {
    pub stack: Vec<JsValue>,
    pub call_stack: Vec<CallFrame>,
    pub globals: &'a [GlobalVariable],
}

impl<'a> VM<'a> {
    pub fn new(globals: &'a [GlobalVariable]) -> Self {
        Self {
            stack: vec![],
            call_stack: vec![],
            globals,
        }
    }

    pub fn eval<'b>(&mut self, code_block: CodeBlock) {
        self.call_stack.push(CallFrame::new(0, code_block));
        self.run();
    }

    fn frame(&self) -> &CallFrame {
        self.call_stack.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.call_stack.last_mut().unwrap()
    }

    fn ip(&self) -> usize {
        self.frame().ip
    }

    fn bp(&self) -> usize {
        self.frame().bp
    }

    fn run(&mut self) {
        if self.call_stack.is_empty() {
            return;
        }

        while self.ip() < self.frame().code_block.bytecode.len() {
            let opcode: Opcode = self.read_byte().into();
            // println!("{opcode:?}");
            match opcode {
                Opcode::Call => {
                    let params_count = self.read_byte();
                    let value = self.peek(params_count as usize);

                    if let JsValue::Object(object) = &value {
                        if let ObjectKind::Function(function) = &object.borrow().kind {
                            match function {
                                JsFunction::NativeBytecode(function) => {
                                    let mut params: Vec<JsValue> = Vec::with_capacity(params_count as usize);
                                    for _ in 0..params_count {
                                        params.insert(0, self.pop());
                                    }
                                    function.call(self, &params);
                                },
                                JsFunction::Bytecode(function) => {
                                    let new_call_frame = CallFrame::new(self.stack.len() - 1 - params_count as usize, function.clone());
                                    self.call_stack.push(new_call_frame);
                                    return self.run();
                                },
                                JsFunction::Ordinary(_) | JsFunction::Native(_) => unreachable!(),
                            }
                        }
                    }
                },
                Opcode::Return => {
                    let return_value = self.pop();
                    self.stack.truncate(self.bp());
                    self.call_stack.pop();
                    self.push(return_value);
                },
                Opcode::GetProp => {
                    let key = self.pop();
                    let object = self.pop();
                    let value = object.as_object().borrow()
                        .get_property_value(key.as_string());
                    self.push(value);
                },
                Opcode::SetProp => {
                    let value = self.pop();
                    let key = self.pop();
                    let object = self.peek_ref(0);

                    match &key {
                        JsValue::String(key) => {
                            object.as_object().borrow_mut().add_property(key, value);
                        },
                        JsValue::Number(key) => {
                            object.as_object().borrow_mut().add_property(
                                key.to_string().as_str(),
                                value
                            );
                        },
                        JsValue::Undefined | JsValue::Null | JsValue::Boolean(_) | JsValue::Object(_) => panic!("{} type cannot be used as a key for an object", value.get_type_as_str()),
                    };
                },
                Opcode::GetVar => {
                    let idx = self.read_byte();
                    self.push(self.stack[self.bp() + idx as usize].clone());
                },
                Opcode::GetGlobal => {
                    let idx = self.read_byte();
                    let value = self.globals[idx as usize].value.clone();
                    self.push(value);
                },
                Opcode::SetVar => {
                    let idx = self.read_byte();
                    let bp = self.bp();
                    self.stack[bp + idx as usize] = self.peek(0);
                },
                Opcode::PushLiteral => {
                    let constant_idx = self.read_byte();
                    let value = self.frame().code_block.constants[constant_idx as usize].clone();
                    self.push(value);
                },
                Opcode::PushTrue => {
                    self.push(JsValue::Boolean(true));
                }
                Opcode::PushFalse => self.push(JsValue::Boolean(false)),
                Opcode::Pop => { self.pop(); },
                Opcode::ExitScope => {
                    let n_pop = self.read_byte();
                    for _ in 0..n_pop {
                        // TODO: mb try to just decrease ip by n_pop?
                        self.pop();
                    }
                    self.call_stack.pop();
                    return self.run();
                },
                Opcode::Jump => {
                    let jump_address = self.read_byte();
                    self.frame_mut().set_ip(jump_address as usize);
                },
                Opcode::JumpIfFalse => {
                    let value = self.pop();
                    let jump_address = self.read_byte();

                    if !value.to_bool() {
                        self.frame_mut().set_ip(jump_address as usize);
                    }
                },
                opcode if opcode.is_binary() => {
                    let left_value = self.pop();
                    let right_value = self.pop();

                    let result = match opcode {
                        Opcode::Or => JsValue::Boolean(left_value.to_bool() || right_value.to_bool()),
                        Opcode::And => JsValue::Boolean(left_value.to_bool() && right_value.to_bool()),
                        Opcode::Add => (&left_value + &right_value).expect("Error during adding two values"),
                        Opcode::Sub => (&left_value - &right_value).expect("Error during subbing two values"),
                        Opcode::Mul => (&left_value * &right_value).expect("Error during multiplication two values"),
                        Opcode::Div => (&left_value / &right_value).expect("Error during division two values"),
                        Opcode::Eq => JsValue::Boolean(left_value.eq(&right_value)),
                        Opcode::Neq => JsValue::Boolean(left_value.ne(&right_value)),
                        Opcode::MulMul => left_value.exponentiation(&right_value).expect("Error during exponentiation two values"),
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
                                Opcode::More => matches!(cmp_result, Ordering::Greater),
                                Opcode::Less => matches!(cmp_result, Ordering::Less),
                                Opcode::LessOrEqual => matches!(cmp_result, Ordering::Less | Ordering::Equal),
                                Opcode::MoreOrEqual => matches!(cmp_result, Ordering::Greater | Ordering::Equal),
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

    pub fn dump_stack(&self) {
        println!("┌{:─^21}┐", " STACK ");

        for (i, value) in self.stack.iter().enumerate().rev() {
            print!("|{: ^21}|", format!("{}", value.display_with_no_colors()));
            if !self.call_stack.is_empty() && self.bp() == i {
                print!(" <-- bp");
            }
            println!();
        }

        if self.stack.len() == 0 {
            println!("|{: ^21}|", "(empty)");
        }

        println!("└{:─^21}┘", "");
    }

    pub fn push(&mut self, value: JsValue) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> JsValue {
        self.stack.pop().expect("There is nothing to pop")
    }

    pub fn peek(&self, n: usize) -> JsValue {
        self.stack.get(self.stack.len() - 1 - n)
            .expect(format!("Cannot peek stack by {n}").as_str()).clone()
    }

    pub fn peek_ref(&'a self, n: usize) -> &'a JsValue {
        self.stack.get(self.stack.len() - 1 - n).expect(format!("Cannot peek stack by {n}").as_str())
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.frame().code_block.bytecode[self.ip()];
        let ip = self.ip();
        self.frame_mut().set_ip(ip + 1);
        byte
    }
}
