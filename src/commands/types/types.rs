use std::fmt::{self, Display};

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

pub struct ServerList(pub Vec<Server>);

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
    pub fn find_server(&mut self, uid: &ID) -> Option<&mut Server> {
        self.0
            .iter_mut()
            .find(
                |server| server.unique_id.0 == uid.0
            )
    }
    pub fn print_servers(&self) {
        for server in self.0.iter() {
            println!("{server}");
        }
    }
}
