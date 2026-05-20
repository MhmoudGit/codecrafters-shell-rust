#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    repl();
}

enum Command {
    Exit,
    Echo(String),
    Unknown,
}

fn commands_from_string(command: &str) -> Command {
    let args = command.split(" ").collect::<Vec<&str>>();
    match args[0] {
        "exit" => Command::Exit,
        "echo" => Command::Echo(args[1..].join(" ")),
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
                Command::Echo(text) => println!("{}", text),
                Command::Unknown => println!("{}: command not found", command),
            }
        }
    }
}
