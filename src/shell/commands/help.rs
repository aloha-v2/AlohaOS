use super::{Command, COMMANDS, CommandEffect};
use crate::console;

pub const COMMAND: Command = Command {
    name: "help",
    description: "show available commands or help for a specific command",
    usage: "help [command]",
    run,
};

pub fn run(args: &str) -> CommandEffect {
    if args.is_empty() {
        print_all();
    } else {
        print_one(args);
    }
    CommandEffect::None
}

fn print_all() {
    console::print("Available commands:\n");
    for cmd in COMMANDS {
        console::print("  ");
        console::print(cmd.name);
        console::print(" - ");
        console::print(cmd.description);    
        console::print_char(b'\n');
    }
}

fn print_one(name: &str) {
    for cmd in COMMANDS {
        if cmd.name == name {
            console::print("Usage: ");
            console::print(cmd.usage);
            console::print_char(b'\n');
            console::print("       ");
            console::print(cmd.description);
            console::print_char(b'\n');
            return;
        }
    }
    console::print("help: unknown command: ");
    console::print(name);
    console::print_char(b'\n');
}