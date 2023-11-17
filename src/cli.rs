use clap::Args;

/// Programming language interpreter
#[derive(clap::Parser)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Args)]
pub struct DefaultArgs {
    #[arg(long)]
    pub debug: bool,

    /// measure time
    #[arg(short, long)]
    pub time: bool,

    #[arg(short, long)]
    pub filename: String,

    /// ignore_warnings
    #[arg(long)]
    pub ignore_warnings: bool,

    /// ignore_errors
    #[arg(long)]
    pub ignore_errors: bool,
}

// #[derive(Clone)]
// enum InterpreterKind {
//     Ast,
//     VM,
// }

#[derive(clap::Subcommand)]
pub enum CliCommand {
    /// run code using virtual machine
    VM(DefaultArgs),
    /// run code using ast interpreter
    Ast(DefaultArgs),
    /// print bytecode
    PrintBytecode(DefaultArgs),
}
