use std::cell::Cell;
use crate::bytecode::bytecode_generator::CodeBlock;
use crate::bytecode::opcode::OpCode;

struct BytecodePrinter<'a> {
    pos: Cell<usize>,
    code_block: &'a CodeBlock,
}

impl<'a> BytecodePrinter<'a> {
    fn new(code_block: &'a CodeBlock) -> Self {
        Self {
            pos: Cell::new(0),
            code_block
        }
    }

    pub fn print(&self) {
        while self.pos.get() < self.code_block.bytecode.len() {
            print!("{}\t", self.pos.get());
            let opcode: OpCode = self.read_byte().into();

            match opcode {
                OpCode::PushLiteral => {
                    let index = self.read_byte();
                    println!("PushLiteral\tindex #{} ({})", index, self.code_block.constants[index as usize]);
                },
                OpCode::PushTrue => println!("PushTrue"),
                OpCode::PushFalse => println!("PushFalse"),
                OpCode::Pop => println!("Pop"),
                OpCode::Add => println!("Add"),
                OpCode::Sub => println!("Sub"),
                OpCode::Mul => println!("Mul"),
                OpCode::Div => println!("Div"),
                OpCode::Eq => println!("Eq"),
                OpCode::Neq => println!("Neq"),
                OpCode::MulMul => println!("MulMul"),
                OpCode::LessOrEqual => println!("LessOrEqual"),
                OpCode::Less => println!("Less"),
                OpCode::MoreOrEqual => println!("MoreOrEqual"),
                OpCode::More => println!("More"),
                OpCode::Or => println!("Or"),
                OpCode::And => println!("And"),
                OpCode::Jump => {
                    let index = self.read_byte();
                    println!("Jump\t\t{index}");
                },
                OpCode::JumpIfFalse => {
                    let index = self.read_byte();
                    println!("JumpIfFalse\t{index}");
                }
                OpCode::Halt => println!("Halt"),
            }
        }
    }

    fn read_byte(&self) -> u8 {
        let current_pos = self.pos.get();
        let byte = self.code_block.bytecode[current_pos];
        self.pos.set(current_pos + 1);
        byte
    }
}

pub fn print_code_block(code_block: &CodeBlock) {
    let bytecode_printer = BytecodePrinter::new(code_block);
    bytecode_printer.print();
}