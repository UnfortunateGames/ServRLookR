use crate::commands::types::*;

fn parse_2_u32(parameter: &str, full_command: &str) -> Result<u32, CommandError> {
    parameter
        .parse::<u32>()
        .map_err(
            |_| CommandError::InvalidParameter(format!(
                "Parameter {} in command {}, is invalid.",
                parameter, full_command
            ))
        )
}

fn parse_commands(commands: &str) -> Result<Vec<Command>, CommandError> {
    let mut command_list: Vec<Command> = Vec::new();
    let mut trimmed_command: String;
    let mut tokens: Vec<&str>;
    for command in commands.split(";") {
        trimmed_command = command
            .trim()
            .to_lowercase();
        if trimmed_command.is_empty() { continue; }
        tokens = trimmed_command
            .split_whitespace()
            .collect();
        match tokens.as_slice() {
            ["help"] => command_list.push(Command::Help),
            ["exit"] => command_list.push(Command::Exit),
            ["call", uid] => command_list.push(
                Command::Call(ID{
                    0: parse_2_u32(
                        uid,
                        &trimmed_command
                    )?
                })
            ),
            ["run", uid] => command_list.push(
                Command::Run(ID{
                    0: parse_2_u32(
                        uid,
                        &trimmed_command
                    )?
                })
            ),
            ["stop", uid] => command_list.push(
                Command::Stop(ID{
                    0: parse_2_u32(
                        uid,
                        &trimmed_command
                    )?
                })
            ),
            ["add"] => command_list.push(
                Command::Add
            ),
            ["remove", uid] => command_list.push(
                Command::Remove(ID{
                    0: parse_2_u32(
                        uid,
                        &trimmed_command
                    )?
                })
            ),
            ["edit", uid] => command_list.push(
                Command::Edit(ID{
                    0: parse_2_u32(
                        uid,
                        &trimmed_command
                    )?
                })
            ),
            ["read", uid] => command_list.push(
                Command::Edit(ID{
                    0: parse_2_u32(
                        uid,
                        &trimmed_command
                    )?
                })
            ),
            ["wait", seconds] => command_list.push(
                Command::Wait(
                    parse_2_u32(
                        seconds,
                        &trimmed_command
                    )? as u64
                )
            ),
            _ => return Err(
                CommandError::InvalidCommand(
                    trimmed_command
                )
            )
        }
    }

    Ok(command_list)
}
