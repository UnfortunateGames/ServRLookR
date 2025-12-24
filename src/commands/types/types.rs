use std::{
    io::{self, BufRead},
    fs::File,
    fmt::{self, Display},
    process::exit,
    path::Path,
    thread::sleep,
    time::Duration
};
use crate::utils::inputf;

#[derive(PartialEq, Eq)]
pub struct ID(pub u32);

pub enum ServerStatus {
    UP,
    RUNNING,
    DOWN
}

pub enum Command {
    Help,
    Exit,
    Call(ID),
    Run(ID),
    Stop(ID),
    Shutdown(ID),
    Activate(ID),
    Add,
    Remove(ID),
    Edit(ID),
    Read(String),
    Wait(u64)
}

pub enum CommandError {
    InvalidCommand(String),
    InvalidParameter(String)
}

pub struct Server {
    pub name: String,
    pub status: ServerStatus,
    pub message: String,
    pub err_message: String,
    pub unique_id: ID
}

pub struct ServerList(Vec<Server>);

fn cmd_edit(server_list: &mut ServerList, uid: &ID) -> Option<String> {
    println!("\nEnter nothing to retain it's value");
    let target: &mut Server = match server_list.find_server(uid) {
        Some(server) => server,
        None => return Some(format!(
            "[ERROR] Unique ID: {} was not found in list.",
            uid.0
        ))
    };
    let new_items: Vec<String> = vec![
        inputf("Name of server -> "),
        inputf("Message output -> "),
        inputf("Error message -> "),
        inputf("Unique ID -> ")
    ];
    if !new_items[0].is_empty() {
        target.name = new_items[0].to_owned();
    } if !new_items[1].is_empty() {
        target.message = new_items[1].to_owned();
    } if !new_items[2].is_empty() {
        target.err_message = new_items[2].to_owned();
    } if !new_items[3].is_empty() {
        target.unique_id = ID{
            0: match new_items[3].parse::<u32>() {
                Ok(value) => value,
                Err(_) => return Some(format!(
                    "[ERROR] Unique ID: {} is not a valid ID.",
                    new_items[3]
                ))
            }
        };
    }
    
    None
}

fn cmd_read(server_list: &mut ServerList, path_string: &str) -> Option<String> {
    let path: &Path = Path::new(path_string);
    let file: File = match File::open(path) {
        Ok(value) => value,
        Err(_) => return Some(format!(
            "[ERROR] {} is not a valid path.",
            path_string
        ))
    };
    let reader: io::BufReader<File> = io::BufReader::new(file);
    let mut read_data: Vec<String> = Vec::new();
    for line in reader.lines() {
        if read_data.len() > 6 { return Some("File has too many lines.".to_string()) }
        match line {
            Ok(value) => read_data.push(value),
            Err(_) => return Some("[ERROR] Could not read file.".to_string())
        }
    }
    if read_data.len() < 5 {
        return Some("[ERROR] Missing data in file.".to_string())
    }
    let get_item =
        |i: usize| read_data[i]
            .trim()
            .to_string();
    server_list.add_server(
        Server{
            name: get_item(0),
            status: match get_item(1)
                .to_lowercase()
                .as_str() {
                "up" => ServerStatus::UP,
                "running" => ServerStatus::RUNNING,
                "down" => ServerStatus::DOWN,
                _ => return Some("[ERROR] Invalid status!".to_string())
            },
            message: get_item(2),
            err_message: get_item(3),
            unique_id: match get_item(4).parse::<u32>() {
                Ok(value) => ID{0: value},
                Err(_) => return Some("[ERROR] Invalid Unique ID!".to_string())
            }
        }
    );

    None
}

impl Command {
    pub fn execute(&self, server_list: &mut ServerList) -> Option<String> {
        match self {
            Command::Help => {},
            Command::Exit => exit(0),
            Command::Call(uid) => {
                match server_list.find_server(uid) {
                    Some(server) => {
                        if let ServerStatus::DOWN = server.status {
                            return Some(format!(
                                "[{}] {}", server.name, server.err_message.to_owned()
                            ))
                        }
                        return Some(format!(
                            "[{}] {}", server.name, server.message.to_owned()
                        ))
                    },
                    None => return Some(format!(
                        "[ERROR] Unique ID: {} was not found in list.",
                        uid.0
                    ))
                }
            },
            Command::Run(uid) => {
                match server_list.find_server(uid) {
                    Some(server) => {
                        match server.status {
                            ServerStatus::DOWN => return Some(format!(
                                "[CONSOLE] {} is currently down.", server.name
                            )),
                            ServerStatus::RUNNING => return Some(format!(
                                "[CONSOLE] {} is already running.", server.name
                            )),
                            ServerStatus::UP => server.status = ServerStatus::RUNNING
                        }
                    },
                    None => return Some(format!(
                        "[ERROR] Unique ID: {} was not found in list.",
                        uid.0
                    ))
                }
            },
            Command::Stop(uid) => {
                match server_list.find_server(uid) {
                    Some(server) => {
                        match server.status {
                            ServerStatus::DOWN => return Some(format!(
                                "[CONSOLE] {} is currently down.", server.name
                            )),
                            ServerStatus::UP => return Some(format!(
                                "[CONSOLE] {} is currently not running.", server.name
                            )),
                            ServerStatus::RUNNING => server.status = ServerStatus::UP
                        }
                    },
                    None => return Some(format!(
                        "[ERROR] Unique ID: {} was not found in list.",
                        uid.0
                    ))
                }
            },
            Command::Shutdown(uid) => {
                match server_list.find_server(uid) {
                    Some(server) => {
                        if let ServerStatus::DOWN = server.status {
                            return Some(format!(
                                "[CONSOLE] {} is already down.", server.name
                            ))
                        }
                        server.status = ServerStatus::DOWN;
                    },
                    None => return Some(format!(
                        "[ERROR] Unique ID: {} was not found in list.",
                        uid.0
                    ))
                }
            },
            Command::Activate(uid) => {
                match server_list.find_server(uid) {
                    Some(server) => {
                        match server.status {
                            ServerStatus::DOWN => server.status = ServerStatus::UP,
                            ServerStatus::RUNNING => return Some(format!(
                                "[CONSOLE] {} is currently running.", server.name
                            )),
                            ServerStatus::UP => return Some(format!(
                                "[CONSOLE] {} is already up.", server.name
                            ))
                        }
                    },
                    None => return Some(format!(
                        "[ERROR] Unique ID: {} was not found in list.",
                        uid.0
                    ))
                }
            },
            Command::Add => {
                let server: Server = Server{
                    name: inputf("\nEnter it's name -> "),
                    status: ServerStatus::UP,
                    message: inputf("Enter it's message -> "),
                    err_message: inputf("Enter it's down message -> "),
                    unique_id: match inputf("What will it's ID be? -> ").parse::<u32>() {
                        Ok(value) => ID{0: value},
                        Err(_) => return Some("Invalid Unique ID!".to_string())
                    }
                };
                server_list.0.push(server);
            },
            Command::Remove(uid) => {
                let past_len: usize = server_list.0.len();
                server_list.0.retain(
                    |server| server.unique_id != *uid
                );
                if server_list.0.len() == past_len {
                    return Some(format!(
                        "Server List had no ID with {}",
                        uid.0
                    ))
                }
            },
            Command::Edit(uid) => return cmd_edit(server_list, uid),
            Command::Read(path) => return cmd_read(server_list, path),
            Command::Wait(seconds) => sleep(
                Duration::from_secs(*seconds)
            )
        }
        None
    }
}

impl Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "- {} -\nStatus -> {}\nUnique ID -> {}",
            self.name,
            match self.status {
                ServerStatus::DOWN => "DOWN",
                ServerStatus::RUNNING => "RUNNING",
                ServerStatus::UP => "UP"
            },
            self.unique_id.0
        )
    }
}

impl ServerList {
    pub fn new() -> ServerList {
        ServerList{0: Vec::new()}
    }
    pub fn find_server(&mut self, uid: &ID) -> Option<&mut Server> {
        self.0
            .iter_mut()
            .find(
                |server| server.unique_id.0 == uid.0
            )
    }
    pub fn add_server(&mut self, server: Server) { self.0.push(server) }
    pub fn print_servers(&self) {
        for server in self.0.iter() {
            println!("{server}");
        }
        println!();
    }
    pub fn call_running(&self) -> Vec<String> {
        let mut output: Vec<String> = Vec::new();
        for server in &self.0 {
            if let ServerStatus::RUNNING = server.status {
                output.push(format!(
                    "[{}] {}", server.name, server.message.to_owned()
                ))
            }
        }

        output
    }
}
