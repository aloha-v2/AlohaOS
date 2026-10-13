use super::{Command, CommandEffect};
use crate::console;
use crate::arch::x86_64::power;

pub const COMMAND: Command = Command {
    name: "shutdown",
    description: "power off the machine",
    usage: "shutdown",
    run,
};

pub fn run(_args: &str) -> CommandEffect {
    console::print("Are you sure you want to shutdown? (yes/no): ");
    CommandEffect::Confirm(power::shutdown)
}