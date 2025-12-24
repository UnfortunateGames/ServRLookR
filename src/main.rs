mod utils;
mod commands;
use utils::*;
use commands::*;

fn main() {
    let mut server_list: ServerList = ServerList::new();
    let mut output_list: Vec<String> = Vec::new();
    let mut user_input: String;
    let mut commands: Vec<Command>;
    loop {
        println!("\n\n\n- ServR LookR -\n");
        server_list.print_servers();
        for call in server_list.call_running() { output_list.push(call) }
        for output in &output_list { println!("{output}") }
        output_list.clear();
        user_input = inputf("\n-> ");
        commands = match parse_commands(&user_input) {
            Ok(vector) => vector,
            Err(error) => {
                match error {
                    CommandError::InvalidCommand(
                        message
                    ) => output_list.push(message),
                    CommandError::InvalidParameter(
                        message
                    ) => output_list.push(message),
                }
                continue;
            }
        };
        for command in commands {
            match command.execute(&mut server_list) {
                Some(message) => output_list.push(message),
                None => continue
            }
        }
    }
}
