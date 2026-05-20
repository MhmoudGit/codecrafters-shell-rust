#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    repl();
}

enum Command {
    Exit,
    Unknown,
}

fn commands_from_string(command: &str) -> Command {
    match command {
        "exit" => Command::Exit,
        _ => Command::Unknown,
    }
}

fn repl() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        if command != "" {
            let cmd = commands_from_string(command);
            match cmd {
                Command::Exit => break,
                Command::Unknown => println!("{}: command not found", command),
            }
        }
    }
}
