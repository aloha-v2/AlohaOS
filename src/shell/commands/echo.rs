use crate::console;
use super::{Command, CommandEffect};

pub const COMMAND: Command = Command {
    name: "echo",
    description: "print the given text",
    usage: "echo <text>",
    run,
};

pub fn run(args: &str) -> CommandEffect {
    console::print(args);
    console::print_char(b'\n');
    CommandEffect::None
}