pub mod echo;
pub mod help;
pub mod clear;
pub mod shutdown;

pub type CommandFn = fn(&str);

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub run: fn(&str) -> CommandEffect,
}

pub const COMMANDS: &[Command] = &[
    echo::COMMAND,
    help::COMMAND,
    clear::COMMAND,
    shutdown::COMMAND,
];

pub enum CommandEffect {
    None,
    Confirm(fn() -> !),
}

enum ShellState {
    Normal,
    Confirming(fn() -> !),
}

pub fn execute(line: &str) -> CommandEffect {
    let line = line.trim_start();
    if line.is_empty() {
        return CommandEffect::None;
    }

    let (user_cmd, args) = match line.find(' ') {
        Some(i) => (&line[..i], line[i + 1..].trim_start()),
        None    => (line, ""),
    };

    for entry in COMMANDS {
        if entry.name == user_cmd {
            return (entry.run)(args);
        }
    }

    crate::console::print("unknown command: ");
    crate::console::print(user_cmd);
    crate::console::print_char(b'\n');

    CommandEffect::None
}