#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    repl();
}

enum Command {
    Exit,
    Echo,
    Type,
    Unknown,
}

fn commands_from_string(command: &str) -> Command {
    match command {
        "exit" => Command::Exit,
        "echo" => Command::Echo,
        "type" => Command::Type,
        _ => Command::Unknown,
    }
}

fn echo_command(text: Vec<&str>) {
    println!("{}", text.join(" "));
}

fn type_command(command: Vec<&str>) {
    let cmd_name = command[0];
    let cmd = commands_from_string(&cmd_name);
    if let Command::Unknown = cmd {
        println!("{}: command not found", cmd_name)
    } else {
        println!("{}: is a shell builtin", cmd_name)
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
            let args = command.split(" ").collect::<Vec<&str>>();
            let command_name = args[0];
            let command_args = args[1..].to_vec();

            let cmd = commands_from_string(command_name);
            match cmd {
                Command::Exit => break,
                Command::Echo => echo_command(command_args),
                Command::Type => type_command(command_args),
                Command::Unknown => println!("{}: command not found", command),
            }
        }
    }
}
