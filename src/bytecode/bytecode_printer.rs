use std::cell::Cell;
use crate::bytecode::bytecode_compiler::{CodeBlock, GlobalVariable};
use crate::bytecode::opcodes::Opcode;

pub struct BytecodePrinter<'a> {
    pos: Cell<usize>,
    pub code_block: &'a CodeBlock,
    pub globals: &'a [GlobalVariable],
}

impl<'a> BytecodePrinter<'a> {
    pub fn new(code_block: &'a CodeBlock, globals: &'a [GlobalVariable]) -> Self {
        Self {
            pos: Cell::new(0),
            code_block,
            globals
        }
    }

    pub fn print(&self) {
        println!("------- BEGIN {:?}-------", self.code_block.name);

        while self.pos.get() < self.code_block.bytecode.len() {
            print!("{:#04X}\t", self.pos.get());
            let opcode: Opcode = self.read_byte().into();

            match opcode {
                Opcode::PushLiteral => {
                    let index = self.read_byte();
                    println!("PushLiteral\tindex #{} ({})", index, self.code_block.constants[index as usize]);
                },
                Opcode::PushTrue => println!("PushTrue"),
                Opcode::PushFalse => println!("PushFalse"),
                Opcode::Add => println!("Add"),
                Opcode::Sub => println!("Sub"),
                Opcode::Mul => println!("Mul"),
                Opcode::Div => println!("Div"),
                Opcode::Eq => println!("Eq"),
                Opcode::Neq => println!("Neq"),
                Opcode::MulMul => println!("MulMul"),
                Opcode::LessOrEqual => println!("LessOrEqual"),
                Opcode::Return => println!("Return"),
                Opcode::SetProp => println!("SetProp"),
                Opcode::GetProp => println!("GetProp"),
                Opcode::Less => println!("Less"),
                Opcode::MoreOrEqual => println!("MoreOrEqual"),
                Opcode::More => println!("More"),
                Opcode::Or => println!("Or"),
                Opcode::And => println!("And"),
                Opcode::Pop => println!("Pop"),
                Opcode::ExitScope => {
                    let n_pop = self.read_byte();
                    println!("ExitScope\t{n_pop}");
                },
                Opcode::Jump => {
                    let index = self.read_byte();
                    println!("Jump\t\t{:#04X}", index);
                },
                Opcode::JumpIfFalse => {
                    let index = self.read_byte();
                    println!("JumpIfFalse\t{index:#04X}");
                }
                Opcode::SetVar => {
                    let index = self.read_byte();
                    println!("SetVar\t\t{index} ({})", self.code_block.locals[index as usize].name);
                },
                Opcode::GetVar => {
                    let index = self.read_byte();
                    println!("GetVar\t\t{index} ({})", self.code_block.locals[index as usize].name);
                },
                Opcode::GetGlobal => {
                    let index = self.read_byte();
                    println!("GetGlobal\t{index} ({})", self.globals[index as usize].name);
                },
                Opcode::Call => {
                    let params_count = self.read_byte();
                    println!("Call\t\t{params_count}");
                }
            }
        }
        println!("-------- END {:?}--------\n", self.code_block.name);
    }

    pub(crate) fn read_byte(&self) -> u8 {
        let current_pos = self.pos.get();
        let byte = self.code_block.bytecode[current_pos];
        self.pos.set(current_pos + 1);
        byte
    }
}
