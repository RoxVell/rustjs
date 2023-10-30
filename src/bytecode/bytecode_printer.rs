use std::cell::Cell;
use crate::bytecode::bytecode_compiler::{CodeBlock, GlobalVariable};
use crate::bytecode::opcodes::Opcode;

pub struct BytecodePrinter<'a> {
    pos: Cell<usize>,
    pub code_block: &'a CodeBlock,
    pub globals: &'a [GlobalVariable],
}

impl<'a> BytecodePrinter<'a> {
    const ADDRESS_COLUMN_WIDTH: usize = 10;
    const OPCODE_NAME_COLUMN_WIDTH: usize = 15;

    pub fn new(code_block: &'a CodeBlock, globals: &'a [GlobalVariable]) -> Self {
        Self {
            pos: Cell::new(0),
            code_block,
            globals
        }
    }

    pub fn print(&self) {
        println!("--------------- BEGIN {:?}---------------", self.code_block.name);

        while self.pos.get() < self.code_block.bytecode.len() {
            let address = format!("{:#06X}", self.pos.get());
            // print!("{address:<WIDTH$}", WIDTH=Self::ADDRESS_COLUMN_WIDTH);
            let opcode: Opcode = self.read_byte().into();

            let (opcode, operands) = match opcode {
                Opcode::PushLiteral => {
                    let index = self.read_byte();
                    ("PushLiteral".to_string(), format!("index #{} ({})", index, self.code_block.constants[index as usize]))
                    // println!("PushLiteral\tindex #{} ({})", index, self.code_block.constants[index as usize]);
                },
                Opcode::PushTrue => ("PushTrue".to_string(), "".to_string()),
                Opcode::PushFalse => ("PushFalse".to_string(), "".to_string()),
                Opcode::UnaryMinus => ("UnaryMinus".to_string(), "".to_string()),
                Opcode::UnaryPlus => ("UnaryPlus".to_string(), "".to_string()),
                Opcode::LogicalNot => ("LogicalNot".to_string(), "".to_string()),
                Opcode::Add => ("Add".to_string(), "".to_string()),
                Opcode::Sub => ("Sub".to_string(), "".to_string()),
                Opcode::Mul => ("Mul".to_string(), "".to_string()),
                Opcode::Div => ("Div".to_string(), "".to_string()),
                Opcode::Eq => ("Eq".to_string(), "".to_string()),
                Opcode::Neq => ("Neq".to_string(), "".to_string()),
                Opcode::MulMul => ("MulMul".to_string(), "".to_string()),
                Opcode::LessOrEqual => ("LessOrEqual".to_string(), "".to_string()),
                Opcode::Return => ("Return".to_string(), "".to_string()),
                Opcode::SetProp => ("SetProp".to_string(), "".to_string()),
                Opcode::GetProp => ("GetProp".to_string(), "".to_string()),
                Opcode::Less => ("Less".to_string(), "".to_string()),
                Opcode::MoreOrEqual => ("MoreOrEqual".to_string(), "".to_string()),
                Opcode::More => ("More".to_string(), "".to_string()),
                Opcode::Or => ("Or".to_string(), "".to_string()),
                Opcode::And => ("And".to_string(), "".to_string()),
                Opcode::Pop => ("Pop".to_string(), "".to_string()),
                Opcode::ExitScope => {
                    let n_pop = self.read_byte();
                    ("ExitScope".to_string(), n_pop.to_string())
                    // println!("ExitScope\t{n_pop}");
                },
                Opcode::Jump => {
                    let index = self.read_byte();
                    ("Jump".to_string(), format!("{:#04X}", index))
                    // println!("Jump\t\t{:#04X}", index);
                },
                Opcode::JumpIfFalse => {
                    let index = self.read_byte();
                    ("JumpIfFalse".to_string(), format!("{index:#04X}"))
                    // println!("JumpIfFalse\t{index:#04X}");
                }
                Opcode::SetVar => {
                    let index = self.read_byte();
                    ("SetVar".to_string(), format!("{index} ({})", self.code_block.locals[index as usize].name))
                    // println!("SetVar\t\t{index} ({})", self.code_block.locals[index as usize].name);
                },
                Opcode::GetVar => {
                    let index = self.read_byte();
                    ("GetVar".to_string(), format!("{index} ({})", self.code_block.locals[index as usize].name))
                    // println!("GetVar\t\t{index} ({})", self.code_block.locals[index as usize].name);
                },
                Opcode::GetGlobal => {
                    let index = self.read_byte();
                    ("GetGlobal".to_string(), format!("{index} ({})", self.globals[index as usize].name))
                    // println!("GetGlobal\t{index} ({})", self.globals[index as usize].name);
                },
                Opcode::Call => {
                    let params_count = self.read_byte();
                    ("Call".to_string(), params_count.to_string())
                    // println!("Call\t\t{params_count}");
                }
            };
            println!("{address:<ADDRESS_WIDTH$}{opcode:<OPCODE_WIDTH$}{operands:<}",
                ADDRESS_WIDTH=Self::ADDRESS_COLUMN_WIDTH,
                OPCODE_WIDTH=Self::OPCODE_NAME_COLUMN_WIDTH
            );
            // print!("{opcode}")
        }
        println!("---------------- END {:?}----------------\n", self.code_block.name);
    }

    pub(crate) fn read_byte(&self) -> u8 {
        let current_pos = self.pos.get();
        let byte = self.code_block.bytecode[current_pos];
        self.pos.set(current_pos + 1);
        byte
    }
}
