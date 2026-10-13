use crate::console;
use super::{Command, CommandEffect};

pub const COMMAND: Command = Command {
    name: "clear",
    description: "clear the screen",
    usage: "clear",
    run,
};


pub fn run(_args: &str) -> CommandEffect {
    console::clear_screen();
    CommandEffect::None
}