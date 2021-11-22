use regex;

#[derive(Debug)]
pub enum Command {
    Text(String),
    RegisterUsername(String),
    Connect(String),
    Disconnect,
    Quit,
}

impl Command {
    pub fn parse(cmd: String) -> Command {
        let register_username_re = regex::Regex::new(r"^/register (.*)$").unwrap();
        let quit_re = regex::Regex::new(r"^/quit$").unwrap();
        let connect_re = regex::Regex::new(r"^/connect (.*):(.*)$").unwrap();
        let disconnect_re = regex::Regex::new(r"^/disconnect$").unwrap();

        if register_username_re.is_match(&cmd) {
            Command::RegisterUsername(String::from(&cmd[10..]))
        } else if quit_re.is_match(&cmd) {
            Command::Quit
        } else if connect_re.is_match(&cmd) {
            Command::Connect(String::from(&cmd[9..]))
        } else if disconnect_re.is_match(&cmd) {
            Command::Disconnect
        } else {
            Command::Text(cmd)
        }
    }

    pub fn get_from_stdin() -> Command {
        let mut input = String::new();
        let stdin = std::io::stdin();

        stdin.read_line(&mut input).unwrap();

        input = input.trim().to_string();

        Command::parse(input)
    }
}
