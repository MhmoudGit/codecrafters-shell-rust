#[allow(unused_imports)]
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command as og_cmd;

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
    for cmd_name in command {
        let cmd = commands_from_string(cmd_name);

        if !matches!(cmd, Command::Unknown) {
            println!("{} is a shell builtin", cmd_name);
            continue;
        }

        match find_in_path(cmd_name) {
            Some(path) => println!("{} is {}", cmd_name, path.display()),
            None => println!("{}: not found", cmd_name),
        }
    }
}

fn find_in_path(cmd_name: &str) -> Option<std::path::PathBuf> {
    let path_var = env::var_os("PATH")?;

    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(cmd_name);

        if is_executable(&candidate) {
            return Some(candidate);
        }

        #[cfg(windows)]
        {
            for ext in ["exe", "cmd", "bat", "com"] {
                let candidate = dir.join(format!("{cmd_name}.{ext}"));

                if is_executable(&candidate) {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    if let Ok(meta) = fs::metadata(path) {
        meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
    } else {
        false
    }
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => matches!(
            ext.to_ascii_lowercase().as_str(),
            "exe" | "cmd" | "bat" | "com"
        ),
        None => false,
    }
}

fn run_command(name: &str, args: Vec<&str>) {
    match find_in_path(name) {
        Some(_path) => match og_cmd::new(name).args(args).status() {
            Ok(_) => {}
            Err(e) => println!("{e}"),
        },
        None => println!("{}: command not found", name),
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
                Command::Unknown => run_command(command_name, command_args),
            }
        }
    }
}
