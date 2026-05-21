#[allow(unused_imports)]
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command as og_cmd, Stdio};

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
        let parsed = parse_redirect(parse_args(command));
        let args = parsed.args;

        if args.is_empty() {
            return true;
        }

        let command_name = args[0].clone();
        let command_args = args[1..].to_vec();

        let cmd = Command::from_string(&command_name);
        match cmd {
            Command::Exit => return false,
            Command::Echo => Command::echo_cmd(command_args, parsed.stdout, parsed.stderr),
            Command::Type => Command::type_cmd(command_args),
            Command::PWD => Command::pwd_cmd(parsed.stdout, parsed.stderr),
            Command::CD => Command::cd_cmd(command_args),
            Command::Unknown => {
                Command::external_command(command_name, command_args, parsed.stdout, parsed.stderr)
            }
        }

        true
    }

    fn echo_cmd(
        text: Vec<String>,
        redirect: Option<(String, bool)>,
        err_redirect: Option<(String, bool)>,
    ) {
        let output = format!("{}\n", text.join(" "));

        if let Some((file, append)) = err_redirect {
            touch_redirect(&file, append);
        }

        if let Some((file, append)) = redirect {
            write_to_file(&file, &output, append);
        } else {
            print!("{output}");
        }
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

    fn pwd_cmd(redirect: Option<(String, bool)>, err_redirect: Option<(String, bool)>) {
        let output = format!("{}\n", env::current_dir().unwrap().display());

        if let Some((file, append)) = err_redirect {
            touch_redirect(&file, append);
        }

        if let Some((file, append)) = redirect {
            write_to_file(&file, &output, append);
        } else {
            print!("{output}");
        }
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

    fn external_command(
        name: String,
        args: Vec<String>,
        redirect: Option<(String, bool)>,
        err_redirect: Option<(String, bool)>,
    ) {
        match find_in_path(&name) {
            Some(_path) => {
                let mut cmd = og_cmd::new(&name);
                cmd.args(&args);

                if let Some((file, append)) = redirect {
                    match OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(append)
                        .truncate(!append)
                        .open(file)
                    {
                        Ok(file) => {
                            cmd.stdout(Stdio::from(file));
                        }
                        Err(e) => {
                            println!("redirection error: {e}");
                            return;
                        }
                    }
                }

                if let Some((file, append)) = err_redirect {
                    match OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(append)
                        .truncate(!append)
                        .open(file)
                    {
                        Ok(file) => {
                            cmd.stderr(Stdio::from(file));
                        }
                        Err(e) => {
                            println!("redirection error: {e}");
                            return;
                        }
                    }
                }

                if let Err(e) = cmd.status() {
                    println!("{e}");
                }
            }
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseState {
    Normal,
    SingleQuote,
    DoubleQuote,
    Escape(Box<ParseState>),
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
                '\\' => state = ParseState::Escape(Box::new(ParseState::Normal)),

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
                '\\' => state = ParseState::Escape(Box::new(ParseState::DoubleQuote)),
                _ => current.push(c),
            },

            ParseState::Escape(prev) => {
                current.push(c);
                state = *prev;
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

#[derive(Debug)]
struct Redirect {
    args: Vec<String>,
    stdout: Option<(String, bool)>, // file, append
    stderr: Option<(String, bool)>, // file, append
}

fn parse_redirect(args: Vec<String>) -> Redirect {
    let mut clean_args = Vec::new();
    let mut stdout = None;
    let mut stderr = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            ">" | "1>" => {
                if i + 1 < args.len() {
                    stdout = Some((args[i + 1].clone(), false));
                    i += 2;
                } else {
                    println!("syntax error: expected file after >");
                    break;
                }
            }
            ">>" | "1>>" => {
                if i + 1 < args.len() {
                    stdout = Some((args[i + 1].clone(), true));
                    i += 2;
                } else {
                    println!("syntax error: expected file after >>");
                    break;
                }
            }
            "2>" => {
                if i + 1 < args.len() {
                    stderr = Some((args[i + 1].clone(), false));
                    i += 2;
                } else {
                    println!("syntax error: expected file after 2>");
                    break;
                }
            }
            "2>>" => {
                if i + 1 < args.len() {
                    stderr = Some((args[i + 1].clone(), true));
                    i += 2;
                } else {
                    println!("syntax error: expected file after 2>>");
                    break;
                }
            }
            _ => {
                clean_args.push(args[i].clone());
                i += 1;
            }
        }
    }

    Redirect {
        args: clean_args,
        stdout,
        stderr,
    }
}

fn write_to_file(path: &str, content: &str, append: bool) {
    let result = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(path);

    match result {
        Ok(mut file) => {
            if let Err(e) = file.write_all(content.as_bytes()) {
                println!("redirection error: {e}");
            }
        }
        Err(e) => println!("redirection error: {e}"),
    }
}

fn touch_redirect(path: &str, append: bool) {
    let result = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(path);

    if let Err(e) = result {
        println!("redirection error: {e}");
    }
}
