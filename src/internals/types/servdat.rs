use crate::internals::{
    LookerState,
    types::AppEvent,
};
use std::{
    io::{self, BufRead},
    fs::File,
    fmt::{self, Display},
    path::Path,
    thread::sleep,
    time::Duration
};
use tokio::sync::mpsc::UnboundedReceiver;
use ratatui::{
    backend::CrosstermBackend,
    Terminal
};

#[derive(PartialEq, Eq, Clone)]
pub struct ID(pub u32);

#[derive(Clone)]
pub enum ServerStatus {
    Up,
    Running,
    Down
}

#[derive(Clone)]
pub enum Token {
    Identifier(String),
    Expression(i32),
    SemiColon,
    AndAnd,
    OrOr,
}

pub enum LexError {
    MismatchIdentifier,
    MismatchExpression,
    MismatchOperator,
}

pub enum ParseError {
    ExpectedXGotY(Token, Token),
    InvalidCommandX(String),
}

#[derive(Clone, PartialEq, Eq)]
pub enum Command {
    Help, Version, Exit,
    Echo, Clear,
    Call, Shutdown, Activate,
    Run, Stop,
    Add, Remove, Edit, Read,
    Wait,
}

#[derive(Clone)]
pub enum Expression {
    Name(String),
    Number(i32),
}

#[derive(Clone)]
pub enum Operator {
    AndAnd, OrOr,
    SemiColon,
}

pub struct Statement {
    pub cmd: Option<Command>,
    pub param: Option<Expression>,
    pub op: Option<Operator>,
}

#[derive(Clone)]
pub struct Server {
    pub name: String,
    pub status: ServerStatus,
    pub message: String,
    pub err_message: String,
    pub unique_id: ID
}

#[derive(Clone)]
pub struct ServerList(Vec<Server>);

enum ExtrResult {
    InvalExpr,
    ServNotFound,
}

pub fn parse_2_u32(parameter: &str) -> Option<u32> {
    return match parameter.parse::<u32>() {
        Ok(uid) => Some(uid),
        Err(_) => None,
    }
}

pub fn parse_2_i32(parameter: &str) -> Option<i32> {
    return match parameter.parse::<i32>() {
        Ok(uid) => Some(uid),
        Err(_) => None,
    }
}

fn extr_uid_n_find_serv(ls: &mut ServerList, param: Option<Expression>) -> Result<&mut Server, ExtrResult> {
    let Some(Expression::Number(uid)) = param else { return Err(ExtrResult::InvalExpr); };
    let Some(serv) = ls.find_server(&ID(uid as u32)) else { return Err(ExtrResult::ServNotFound); };

    Ok(serv)
}

fn report_extr_uid(extr_res: ExtrResult, ls: &mut LookerState) {
	match extr_res {
        ExtrResult::InvalExpr => ls.out_print(Some("ERROR"), "Invalid Parameter."),
        ExtrResult::ServNotFound => ls.out_print(Some("ERROR"), "Could not find server."),
    };
}

async fn get_server_input(
    ls: &mut LookerState,
    recv: &mut UnboundedReceiver<AppEvent>,
    term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Option<[String; 5]> {
    let mut inputs: [String; 5] = [
        "Enter the server's name.".to_string(),
        "Enter the server's status.".to_string(),
        "Enter the server message.".to_string(),
        "Enter the error message.".to_string(),
        "Enter the unique ID.".to_string(),
    ];
    for i in inputs.iter_mut() {
        ls.out_print(Some("PROMPT"), i);
        let input = ls.get_user_input(recv, term).await?;
        *i = input.clone();
    }

    Some(inputs)
}

fn parse_input_2_serv(ls: &mut LookerState, inputs: [String; 5]) -> Option<Server> {
    Some(Server{
        name: inputs[0].to_string(),
        status: match inputs[1].to_lowercase().as_str() {
            "up" => ServerStatus::Up,
            "down" => ServerStatus::Down,
            "running" => ServerStatus::Running,
            _ => {
                ls.out_print(Some("ERROR"), "Invalid Server Status given.");
                return None;
            },
        },
        message: inputs[2].to_string(),
        err_message: inputs[3].to_string(),
        unique_id: match parse_2_u32(&inputs[4]) {
            Some(id) => ID(id),
            None => {
                ls.out_print(Some("ERROR"), "Could not parse unique ID to ID, maybe it is not a number?");
                return None;
            },
        },
    })
}

impl Command {
    pub async fn execute(
        self,
		param: Option<Expression>,
        ls: &mut LookerState,
        recv: &mut UnboundedReceiver<AppEvent>,
        term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> bool {
        match self {
            Command::Help => {
                if param.is_some() {
                    ls.out_print(Some("ERROR"), "You have given parameters to a parameter-less command.");
                    return false;
                }
                let help_message: &str = r#"
 - ServRLookR -
 A mock server caller TUI.
 
 - Available Commands -
 	HELP            => Print this help message.
 	VERSION         => Print the current version.
 	EXIT            => Exit the program, as Ctrl+C doesn't work anymore.
 	CALL 	 [UID]  => Calls the server, it may return a message, or an error message.
 	SHUTDOWN [UID]  => Shuts down a running or up server.
 	ACTIVATE [UID]  => Activates a downed server.
 	RUN      [UID]  => Runs a server, that gets called every tick.
 	STOP     [UID]  => Stops a running server, but it doesn't shut it down.
 	ADD      [UID]  => Prompts you to make the server components.
 	REMOVE   [UID]  => Removes a server, forever.
 	EDIT     [UID]  => Edit a preexisting server.
 	READ     [Path] => Reads a file from [Path], it must be this structure:
 		[NAME]   	    => The name of said server.
 		[STATUS] 	    => Must be; UP, DOWN, RUNNING. Case insensitive.
 		[MESSAGE]	    => The message when it's called when UP or RUNNING.
 		[ERROR MESSAGE] => The message displayed when down, and called.
 		[UNIQUE ID]     => The unique ID of the server
 	WAIT [SECONDS]  => Waits for SECONDS seconds.
"#;
                ls.out_print(None, help_message);
            },

            Command::Version => {
                if param.is_some() {
                    ls.out_print(Some("ERROR"), "You've given parameters to a parameter-less command.");
                    return false;
                }
				ls.out_print(Some("CONSOLE"), "ServRLookR_v0.2.0")
            },

            Command::Exit => { /* This is up to the runner. */ },

            Command::Echo => {
                if let Some(expr) = param {
                    match expr {
                        Expression::Name(s) => ls.out_print(Some("ECHO"), &s),
                        Expression::Number(n) => ls.out_print(Some("ECHO"), format!("{n}").as_str()),
                    }
                }
            },

            Command::Clear => {
                if param.is_some() {
                    ls.out_print(Some("ERROR"), "You've given parameters to a parameter-les command.");
                    return false;
                }
				ls.output.clear();                
            }

            Command::Call => match extr_uid_n_find_serv(&mut ls.list, param) {
                Ok(serv) => {
                    let (s_name, s_msg) = {(serv.name.clone(), serv.call_message().to_string())};
                    ls.out_print(Some(&s_name), &s_msg);
                },
                Err(err) => {
                    report_extr_uid(err, ls);
                    return false;
                },
            },

            Command::Shutdown => match extr_uid_n_find_serv(&mut ls.list, param) {
               Ok(serv) => {
                   if let ServerStatus::Down = serv.status {
                       ls.out_print(Some("CONSOLE"), "The server is already down.");
                       return false;
                   }
                   serv.status = ServerStatus::Down;
               },
                Err(err) => {
                    report_extr_uid(err, ls);
                    return false;
                },
            },

            Command::Activate => match extr_uid_n_find_serv(&mut ls.list, param) {
                Ok(serv) => {
                    if let ServerStatus::Down = serv.status {
                        serv.status = ServerStatus::Up;
                        return true;
                    }
                    ls.out_print(Some("CONSOLE"), "The server is already up.");
                    return false;
                },
                Err(err) => {
                    report_extr_uid(err, ls);
                    return false;
                },
            },

            Command::Run => match extr_uid_n_find_serv(&mut ls.list, param) {
                Ok(serv) => {
                    match serv.status {
                	    ServerStatus::Down => ls.out_print(Some("CONSOLE"), "Cannot run, server is down."),
                	    ServerStatus::Running => ls.out_print(Some("CONSOLE"), "Server is already running."),
                	    ServerStatus::Up => {
                            serv.status = ServerStatus::Running;
							return true;
						},
                	}
                    return false;
                },
                Err(err) => {
                    report_extr_uid(err, ls);
                    return false;
                },
            },

            Command::Stop => match extr_uid_n_find_serv(&mut ls.list, param) {
                Ok(serv) => {
                	match serv.status {
                	    ServerStatus::Down => ls.out_print(Some("CONSOLE"), "Server is down."),
                	    ServerStatus::Up => ls.out_print(Some("CONSOLE"), "Server is not running."),
                	    ServerStatus::Running => {
                            serv.status = ServerStatus::Up;
                            return true;
                        },
                	}
                    return false;
                },
                Err(err) => {
                    report_extr_uid(err, ls);
                    return false;
                },
            },

            Command::Add => {
                let Some(inputs) = get_server_input(ls, recv, term).await else {
                    ls.out_print(Some("ERROR"), "Error when handling event signals.");
                    return false;
                };
                let Some(serv) = parse_input_2_serv(ls, inputs) else { return false; };
                ls.list.add_server(serv);
            }

            Command::Remove => {
                let Some(Expression::Number(uid)) = param else {
                    ls.out_print(Some("ERROR"), "Invalid parameter, expected unique ID.");
                    return false;
                };
                let past_len = ls.list.0.len();
                ls.list.0.retain(|s| s.unique_id.0 != uid as u32);
                if ls.list.0.len() == past_len {
                    ls.out_print(Some("WARNING"), format!("No server removed with ID {uid}").as_str());
                    return false;
                }
            }

            Command::Edit => {
                let Some(Expression::Number(uid)) = param else {
                    ls.out_print(Some("ERROR"), "Invalid parameter, expected unique ID.");
                    return false;
                };
                let Some(inputs) = get_server_input(ls, recv, term).await else {
                    ls.out_print(Some("ERROR"), "Error when handling event signals.");
                    return false;
                };
                let Some(serv) = ls.list.find_server(&ID(uid as u32)) else {
                    ls.out_print(Some("ERROR"), format!("Server with ID {uid} is not in the server list.").as_str());
                    return false;
                };

                if !inputs[0].is_empty() { serv.name = inputs[0].clone(); }
                if !inputs[1].is_empty() { serv.status = match inputs[1].to_lowercase().as_str() {
                    "up" => ServerStatus::Up,
                    "down" =>  ServerStatus::Down,
                    "running" => ServerStatus::Running,
                    _ => {
                        ls.out_print(Some("ERROR"), "Invalid server status.");
                        return false;
                    },
                }}
                if !inputs[2].is_empty() { serv.message = inputs[2].clone(); }
                if !inputs[3].is_empty() { serv.err_message = inputs[3].clone(); }
                if !inputs[4].is_empty() { serv.unique_id = match parse_2_u32(&inputs[4]) {
                    Some(uid) => ID(uid),
                    None => { return false; },
                }}
            }

            Command::Read => {
                let Some(Expression::Name(path_str)) = param else {
                    ls.out_print(Some("ERROR"), "Invalid parameter, expected Path.");
                    return false;
                };
                let path: &Path = Path::new(&path_str);
                let file: File = match File::open(path) {
                    Ok(f) => f,
                    Err(_) => {
                        ls.out_print(Some("ERROR"), format!("Path {path_str} is not a file.").as_str());
                        return false;
                    },
                };
                let reader: io::BufReader<File> = io::BufReader::new(file);
                /* I'm sorry! Can future me fix this for me? */
                let mut read_data: [String; 5] = [String::new(), String::new(), String::new(), String::new(), String::new()];
                let mut idx: usize = 0;
                for line in reader.lines() {
                    if idx > 6 {
                        ls.out_print(Some("ERROR"), "File had too much data.");
                        return false;
                    } if idx == 5 { continue; }
                    let Ok(rv) = line else {
                        ls.out_print(Some("ERROR"), "Buffer reader had an error, please restart the program");
                        return false;
                    };
                    read_data[idx] = rv;
                    idx += 1;
                } if idx < 5 {
                    ls.out_print(Some("ERROR"), "File had less data than expected.");
                    return false;
                }
                let Some(serv) = parse_input_2_serv(ls, read_data) else { return false; };
                ls.list.add_server(serv);
            },

            Command::Wait => {
                let Some(Expression::Number(sec)) = param else {
                    ls.out_print(Some("ERROR"), "Invalid parameter, expected a number.");
                    return false;
                };
                sleep(Duration::from_secs(sec as u64));
            },
        }

        true
    }
}

impl Statement {
    pub fn new() -> Statement {
        Statement{
            cmd: None,
            param: None,
            op: None
        }
    }

    pub async fn synced_exec(
        &self,
        ls: &mut LookerState,
        recv: &mut UnboundedReceiver<AppEvent>,
        term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> bool {
        let Some(cmd) = self.cmd.clone() else { return false; };
        let _param = self.param.clone();
        return cmd.execute(_param, ls, recv, term).await;
    }
}

impl Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, " - {} -\n -> Status ->> {}\n -> Unique ID ->> {}",
            self.name,
            match self.status {
                ServerStatus::Down => "DOWN",
                ServerStatus::Running => "RUNNING",
                ServerStatus::Up => "UP"
            },
            self.unique_id.0
        )
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _self = self.clone();
        write!(
            f, "{}",
            match _self {
                Token::Identifier(s) => s,
                Token::Expression(s) => s.to_string(),
                Token::SemiColon => ";".to_string(),
                Token::AndAnd => "&&".to_string(),
                Token::OrOr => "||".to_string(),
            }
        )
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "{}",
            match self {
                Command::Help => "[HELP]",
                Command::Version => "[VERSION]",
                Command::Exit => "[EXIT]",
                Command::Echo => "[ECHO]",
                Command::Clear => "[CLEAR]",
                Command::Call => "[CALL]",
                Command::Shutdown => "[SHUTDOWN]",
                Command::Activate => "[ACTIVATE]",
                Command::Run => "[RUN]",
                Command::Stop => "[STOP]",
                Command::Add => "[ADD]",
                Command::Remove => "[REMOVE]",
                Command::Edit => "[EDIT]",
                Command::Read => "[READ]",
                Command::Wait => "[WAIT]",
            }
        )
    }
}

impl Server {
    pub fn call_message(&self) -> &str {
		if let ServerStatus::Down = self.status {
            return self.err_message.as_str();
        }
        self.message.as_str()
    }
}

impl ServerList {
    pub fn new() -> ServerList {
        ServerList(Vec::new())
    }

    pub fn find_server(&mut self, uid: &ID) -> Option<&mut Server> {
        self.0
            .iter_mut()
            .find(
                |server| server.unique_id.0 == uid.0
            )
    }
    pub fn add_server(&mut self, server: Server) { self.0.push(server) }
    pub fn return_serverlist(&self) -> String {
		let mut rval: String = String::new();
        for server in &self.0 {
            rval.push_str(&format!("{server}\n"));
        }
        rval += "\n";

        rval
    }
    /* -- UNUSED --
    pub fn print_servers(&self) {
        for server in self.0.iter() {
            println!("{server}");
        }
        println!();
    }
    */
    pub fn call_running(&self) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        for server in &self.0 {
            if let ServerStatus::Running = server.status {
                output.push(format!(
                    "[{}] {}", server.name, server.message.to_owned()
                ))
            }
        }

        output
    }
}
