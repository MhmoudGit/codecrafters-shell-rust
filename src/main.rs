#[allow(unused_imports)]
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command as og_cmd;

fn main() {
    repl();
}

fn repl() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let command = command.trim();

        if command != "" {
            if !Command::execute(command) {
                break;
            }
        }
    }
}

enum Command {
    Exit,
    Echo,
    Type,
    PWD,
    CD,
    Unknown,
}

impl Command {
    fn from_string(command: &str) -> Command {
        match command {
            "exit" => Command::Exit,
            "echo" => Command::Echo,
            "type" => Command::Type,
            "pwd" => Command::PWD,
            "cd" => Command::CD,
            _ => Command::Unknown,
        }
    }

    fn execute(command: &str) -> bool {
        let args = parse_args(command);

        if args.is_empty() {
            return true;
        }

        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();

        let cmd = Command::from_string(&command_name);
        match cmd {
            Command::Exit => return false,
            Command::Echo => Command::echo_cmd(command_args),
            Command::Type => Command::type_cmd(command_args),
            Command::PWD => Command::pwd_cmd(),
            Command::CD => Command::cd_cmd(command_args),
            Command::Unknown => Command::external_command(command_name, command_args),
        }

        true
    }

    fn echo_cmd(text: Vec<String>) {
        println!("{}", text.join(" "));
    }

    fn type_cmd(command: Vec<String>) {
        for cmd_name in command {
            let cmd = Command::from_string(&cmd_name);

            if !matches!(cmd, Command::Unknown) {
                println!("{} is a shell builtin", cmd_name);
                continue;
            }

            match find_in_path(&cmd_name) {
                Some(path) => println!("{} is {}", cmd_name, path.display()),
                None => println!("{}: not found", cmd_name),
            }
        }
    }

    fn pwd_cmd() {
        println!("{}", env::current_dir().unwrap().display());
    }

    fn cd_cmd(path: Vec<String>) {
        if path.len() == 0 {
            println!("cd: missing argument");
            return;
        }

        let home = env::var("HOME").unwrap();
        let target_dir = if path[0] == "~" { &home } else { &path[0] };

        match env::set_current_dir(target_dir) {
            Ok(_) => {}
            Err(_) => println!("cd: {target_dir}: No such file or directory"),
        }
    }

    fn external_command(name: String, args: Vec<String>) {
        match find_in_path(&name) {
            Some(_path) => match og_cmd::new(&name).args(&args).status() {
                Ok(_) => {}
                Err(e) => println!("{e}"),
            },
            None => println!("{}: command not found", name),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseState {
    Normal,
    SingleQuote,
    DoubleQuote,
}

fn parse_args(command: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut state = ParseState::Normal;

    for c in command.chars() {
        match state {
            ParseState::Normal => match c {
                '\'' => state = ParseState::SingleQuote,
                '"' => state = ParseState::DoubleQuote,

                ' ' => {
                    if !current.is_empty() {
                        args.push(current);
                        current = String::new();
                    }
                }

                _ => current.push(c),
            },

            ParseState::SingleQuote => match c {
                '\'' => state = ParseState::Normal,
                _ => current.push(c),
            },

            ParseState::DoubleQuote => match c {
                '"' => state = ParseState::Normal,
                _ => current.push(c),
            },
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}
