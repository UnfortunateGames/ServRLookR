use crate::commands::types::*;
use crate::utils::inputf;
use std::{
    process::exit,
    thread::sleep,
    time::Duration
};

fn run_commands(commands: &Vec<Command>, server_list: &mut ServerList) -> Vec<String> {
    let mut output_list: Vec<String> = Vec::new();
    for command in commands {
        match command {
            Command::Help => {},
            Command::Exit => exit(0),
            Command::Call(uid) => {
                match server_list.find_server(uid) {
                    Some(server) => output_list.push(
                        server.message.to_owned()
                    ),
                    None => output_list.push(format!(
                        "Unique ID: {} was not found in list.", uid.0
                    ))
                }
            },
            Command::Run(uid) => {
                match server_list.find_server(uid) {
                    Some(server) => server.status = ServerStatus::RUNNING,
                    None => output_list.push(format!(
                        "Unique ID: {} was not found in list.", uid.0
                    ))
                }
            },
            Command::Stop(uid) => {
                match server_list.find_server(uid) {
                    Some(server) => server.status = ServerStatus::UP,
                    None => output_list.push(format!(
                        "Unique ID: {} was not found in list.", uid.0
                    ))
                }
            },
            Command::Shutdown(uid) => {
                match server_list.find_server(uid) {
                    Some(server) => server.status = ServerStatus::DOWN,
                    None => output_list.push(format!(
                        "Unique ID: {} was not found in list.", uid.0
                    ))
                }
            }
            Command::Add => {
                let server: Server = Server{
                    name: inputf("Enter it's name -> "),
                    status: ServerStatus::UP,
                    message: inputf("Enter it's message -> "),
                    err_message: inputf("Enter it's down message -> "),
                    unique_id: match inputf("What will it's ID be? -> ").parse::<u32>() {
                        Ok(value) => ID{0: value},
                        Err(_) => {
                            output_list.push("Invalid Unique ID!".to_string());
                            continue;
                        }
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
                    output_list.push(format!(
                        "Server List had no ID with {}", uid.0
                    ))
                }
            },
            Command::Edit(uid) => {

            },
            Command::Read(path) => {

            },
            Command::Wait(seconds) => sleep(
                Duration::from_secs(*seconds)
            )
        }
    }

    output_list
}