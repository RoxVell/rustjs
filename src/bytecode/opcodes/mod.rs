// mod push_literal;
// mod push_true;
// mod jump;
// mod jump_if_false;
// mod add;
// mod sub;
// mod div;
// mod mul;
// mod push_false;
// mod mul_mul;
// mod or;
// mod and;
// mod eq;
// mod neq;
// mod less;

// use crate::bytecode::bytecode_interpreter::VM;
// use crate::bytecode::bytecode_printer::BytecodePrinter;
// use crate::bytecode::opcodes::add::Add;
// use crate::bytecode::opcodes::and::And;
// use crate::bytecode::opcodes::div::Div;
// use crate::bytecode::opcodes::jump::Jump;
// use crate::bytecode::opcodes::jump_if_false::JumpIfFalse;
// use crate::bytecode::opcodes::mul::Mul;
// use crate::bytecode::opcodes::mul_mul::MulMul;
// use crate::bytecode::opcodes::neq::Neq;
// use crate::bytecode::opcodes::Opcode::{PushFalse, PushLiteral, PushTrue};
// use crate::bytecode::opcodes::or::Or;
// use crate::bytecode::opcodes::push_false::PushFalse;
// use crate::bytecode::opcodes::push_literal::PushLiteral;
// use crate::bytecode::opcodes::push_true::PushTrue;
// use crate::bytecode::opcodes::sub::Sub;
// use crate::bytecode::opcodes::eq::Eq;

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum Opcode {
    PushLiteral,
    PushTrue,
    PushFalse,
    Add,
    Sub,
    Mul,
    MulMul,
    Div,
    Eq,
    Neq,
    LessOrEqual,
    Less,
    MoreOrEqual,
    More,
    Or,
    And,
    Jump,
    JumpIfFalse,
    Pop,
    SetVar,
    GetVar,
    ExitScope,
    Call,
    GetGlobal,
    Return,
    SetProp,
    GetProp,
    LogicalNot,
    UnaryPlus,
    UnaryMinus,
}

// #[repr(u8)]
// #[derive(Debug)]
// pub(crate) enum Opcode {
//     PushLiteral(PushLiteral),
//     PushTrue(PushTrue),
//     PushFalse(PushFalse),
//     Add(Add),
//     Sub(Sub),
//     Mul(Mul),
//     MulMul(MulMul),
//     Div(Div),
//     Eq(Eq),
//     Neq(Neq),
//     LessOrEqual,
//     Less,
//     MoreOrEqual,
//     More,
//     Or(Or),
//     And(And),
//     Jump(Jump),
//     JumpIfFalse(JumpIfFalse),
// }
//
// trait Execute {
//     const NAME: &'static str;
//     fn execute(&self, vm: &mut VM);
// }
//
// trait Disassemble: Execute {
//     fn disassemble<'a>(&self, _: &mut BytecodePrinter<'a>) {
//         println!("{}", Self::NAME);
//     }
// }
//
// impl Execute for Opcode {
//     const NAME: &'static str = "";
//
//     fn execute(&self, vm: &mut VM) {
//         match self {
//             Opcode::PushLiteral(opcode) => opcode.execute(vm),
//             Opcode::PushTrue(opcode) => opcode.execute(vm),
//             Opcode::PushFalse(opcode) => opcode.execute(vm),
//             Opcode::Add(opcode) => opcode.execute(vm),
//             Opcode::Sub(opcode) => opcode.execute(vm),
//             Opcode::Mul(opcode) => opcode.execute(vm),
//             Opcode::MulMul(opcode) => opcode.execute(vm),
//             Opcode::Div(opcode) => opcode.execute(vm),
//             Opcode::Eq(opcode) => opcode.execute(vm),
//             Opcode::Neq(opcode) => opcode.execute(vm),
//             Opcode::LessOrEqual(opcode) => opcode.execute(vm),
//             Opcode::Less(opcode) => opcode.execute(vm),
//             Opcode::MoreOrEqual(opcode) => opcode.execute(vm),
//             Opcode::More(opcode) => opcode.execute(vm),
//             Opcode::Or(opcode) => opcode.execute(vm),
//             Opcode::And(opcode) => opcode.execute(vm),
//             Opcode::Jump(opcode) => opcode.execute(vm),
//             Opcode::JumpIfFalse(opcode) => opcode.execute(vm),
//         }
//     }
// }
//
// impl Disassemble for Opcode {
//     fn disassemble<'a>(&self, printer: &mut BytecodePrinter<'a>) {
//         match self {
//             Opcode::PushLiteral(opcode) => opcode.disassemble(printer),
//             Opcode::PushTrue(opcode) => opcode.disassemble(printer),
//             Opcode::PushFalse(opcode) => opcode.disassemble(printer),
//             Opcode::Add(opcode) => opcode.disassemble(printer),
//             Opcode::Sub(opcode) => opcode.disassemble(printer),
//             Opcode::Mul(opcode) => opcode.disassemble(printer),
//             Opcode::MulMul(opcode) => opcode.disassemble(printer),
//             Opcode::Div(opcode) => opcode.disassemble(printer),
//             Opcode::Eq(opcode) => opcode.disassemble(printer),
//             Opcode::Neq(opcode) => opcode.disassemble(printer),
//             Opcode::LessOrEqual(opcode) => opcode.disassemble(printer),
//             Opcode::Less(opcode) => opcode.disassemble(printer),
//             Opcode::MoreOrEqual(opcode) => opcode.disassemble(printer),
//             Opcode::More(opcode) => opcode.disassemble(printer),
//             Opcode::Or(opcode) => opcode.disassemble(printer),
//             Opcode::And(opcode) => opcode.disassemble(printer),
//             Opcode::Jump(opcode) => opcode.disassemble(printer),
//             Opcode::JumpIfFalse(opcode) => opcode.disassemble(printer),
//         }
//     }
// }
//
impl Opcode {
    pub fn is_binary(&self) -> bool {
        matches!(self, Opcode::Add
            | Opcode::Sub
            | Opcode::Mul
            | Opcode::Div
            | Opcode::Eq
            | Opcode::Neq
            | Opcode::And
            | Opcode::Or
            | Opcode::MulMul
            | Opcode::More
            | Opcode::Less
            | Opcode::LessOrEqual
            | Opcode::MoreOrEqual
        )
    }

    pub fn is_comparison(&self) -> bool {
        matches!(self, Opcode::More
            | Opcode::Less
            | Opcode::LessOrEqual
            | Opcode::MoreOrEqual
        )
    }
}

impl Into<Opcode> for u8 {
    fn into(self) -> Opcode {
        match self {
            0 => Opcode::PushLiteral,
            1 => Opcode::PushTrue,
            2 => Opcode::PushFalse,
            3 => Opcode::Add,
            4 => Opcode::Sub,
            5 => Opcode::Mul,
            6 => Opcode::MulMul,
            7 => Opcode::Div,
            8 => Opcode::Eq,
            9  => Opcode::Neq,
            10 => Opcode::LessOrEqual,
            11 => Opcode::Less,
            12 => Opcode::MoreOrEqual,
            13 => Opcode::More,
            14 => Opcode::Or,
            15 => Opcode::And,
            16 => Opcode::Jump,
            17 => Opcode::JumpIfFalse,
            18 => Opcode::Pop,
            19 => Opcode::SetVar,
            20 => Opcode::GetVar,
            21 => Opcode::ExitScope,
            22 => Opcode::Call,
            23 => Opcode::GetGlobal,
            24 => Opcode::Return,
            25 => Opcode::SetProp,
            26 => Opcode::GetProp,
            27 => Opcode::LogicalNot,
            28 => Opcode::UnaryPlus,
            29 => Opcode::UnaryMinus,
            _ => todo!()
        }
    }
}
