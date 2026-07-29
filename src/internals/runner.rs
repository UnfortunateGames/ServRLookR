use ratatui::{
    Terminal,
    backend::CrosstermBackend,
};
use tokio::sync::mpsc::UnboundedReceiver;
use crate::internals::types::*;
use crate::internals::LookerState;

enum ParserState {
    Command,
    Parameter,
    Operator,
}

pub fn parse_tokens(tokens: Vec<Token>) -> Result<Vec<Statement>, ParseError> {
	let mut il: Vec<Statement> = vec![Statement::new()];
    let mut il_idx: usize = 0;
    let mut ps: ParserState = ParserState::Command;
	for token in tokens {
        match ps {
            ParserState::Command => {
                let Token::Identifier(s) = token else {
                    return Err(ParseError::ExpectedXGotY(
                        Token::Identifier(String::new()),
                        token
                    ));
                };
                il[il_idx].cmd = Some(match s.as_str() {
                    "help"     => Command::Help,
                    "version"  => Command::Version,
                    "exit"     => Command::Exit,
                    "echo"     => Command::Echo,
                    "clear"    => Command::Clear,
                    "call"     => Command::Call,
                    "shutdown" => Command::Shutdown,
                    "activate" => Command::Activate,
                    "run"      => Command::Run,
                    "stop"     => Command::Stop,
                    "add"      => Command::Add,
                    "remove"   => Command::Remove,
                    "edit"     => Command::Edit,
                    "read"     => Command::Read,
                    "wait"     => Command::Wait,
                    _ => return Err(ParseError::InvalidCommandX(s)),
                });
                ps = ParserState::Parameter;
                continue;
            },
            ParserState::Parameter => {
                ps = ParserState::Operator;
                il[il_idx].param = match token {
                    Token::Identifier(s) => Some(Expression::Name(s)),
                    Token::Expression(n) => Some(Expression::Number(n)),
                    t => {
                        il[il_idx].op = Some(match t {
                            Token::SemiColon => Operator::SemiColon,
                            Token::AndAnd    => Operator::AndAnd,
                            Token::OrOr      => Operator::OrOr,
                            _  				 => {
                                println!("If you're seeing this, the parser messed up.");
                                Operator::SemiColon
                            },
                        });
						ps = ParserState::Command;
                        il.push(Statement::new());
            			il_idx += 1;

                        None
                    },
                };
            },
            ParserState::Operator => {
                il[il_idx].op = Some(match token {
                    Token::SemiColon => Operator::SemiColon,
                    Token::AndAnd    => Operator::AndAnd,
                    Token::OrOr      => Operator::OrOr,
                    t => return Err(ParseError::ExpectedXGotY(Token::SemiColon, t)),
                });
				ps = ParserState::Command;
                il.push(Statement::new());
            	il_idx += 1;
            },
        }
    }

    Ok(il)
}

/* Returns a bool if the program should exit or not. */
pub async fn run_instr_list(
    instr_list: Vec<Statement>,
    ls: &mut LookerState,
    recv: &mut UnboundedReceiver<AppEvent>,
    term: &mut Terminal<CrosstermBackend<std::io::Stdout>>
) -> bool {
    let mut cmd_rval: bool = true;
    let mut end: bool = false;
    let mut exit_rval: bool = false;
    for stmnt in instr_list {
        if end {
            /*
             * Also not possible conceptually.
             * Probably remove this? When sure, remove this.
             *
             * TODO(V3): REMOVE THESE CASES WHERE NORMALLY IMPOSSIBLE.
             */
            ls.out_print(Some("ERROR"), "Command list had no operator, yet continues.");
            break;
        }
        if stmnt.cmd == Some(Command::Exit) {
            exit_rval = true;
        }
        match stmnt.op.clone() {
            Some(op) => match op {
                Operator::AndAnd => {if cmd_rval {
                    cmd_rval = stmnt.synced_exec(ls, recv, term).await;
                }},
                Operator::OrOr => {
                    let rval = stmnt.synced_exec(ls, recv, term).await;
                    if !cmd_rval { cmd_rval = rval; }
                },
                Operator::SemiColon => {
                    cmd_rval = stmnt.synced_exec(ls, recv, term).await;
                },
            },
            None => {
                stmnt.synced_exec(ls, recv, term).await;
                end = true;
            },
        }
    }

    exit_rval
}
