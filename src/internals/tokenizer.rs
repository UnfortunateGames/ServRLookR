use crate::internals::types::*;

#[derive(PartialEq, Eq, Clone, Copy)]
enum LexState {
    Start,      /* Starting.   				  */
    Identifier, /* For command *identifiers*. */
    Expression, /* For parameters.     		  */
    Operator,   /* For operators.      		  */
}

fn finalize_token(token: &str) -> Token {
	let rv: Token = match token {
        ";"  => Token::SemiColon,
        "&&" => Token::AndAnd,
        "||" => Token::OrOr,
        t => match parse_2_i32(t) {
            Some(expr) => Token::Expression(expr),
            None => Token::Identifier(t.to_lowercase()),
        }
    };

    rv
}

fn report_ftoken(s: &LexState, t: &Token) -> Option<LexError> {
    return match (s, t) {
        (LexState::Identifier, Token::Identifier(_)) => None,
        (LexState::Identifier, _) => Some(LexError::MismatchIdentifier),
        (LexState::Expression, Token::Expression(_)) => None,
        (LexState::Expression, _) => Some(LexError::MismatchExpression),
        (LexState::Operator, Token::Identifier(_)) |
		(LexState::Operator, Token::Expression(_)) => Some(LexError::MismatchOperator),
        _ => None,
    }
}

fn wrap_ftoken(tokens: &mut Vec<Token>, token: &mut String, chr: char, state: LexState) -> Option<LexError> {
    let t: Token = finalize_token(&token);
    tokens.push(t.clone());
    token.clear();
    token.push(chr);
    return report_ftoken(&state, &t);
}

pub fn tokenize_cmd(commands: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut token: String = String::new();
    let mut state: LexState = LexState::Start;
    let mut has_split_string: bool = false;
    for chr in commands.chars() {
        match (state, chr) {
            (LexState::Start, c) => {
                token.push(c);
                if c.is_numeric() {
                    state = LexState::Expression;
                } else if c == ';' || c == '&' || c == '|' {
                    state = LexState::Operator;
                } else {
                    state = LexState::Identifier;
                }
            },
            (LexState::Identifier, c) => {
                if c.is_alphabetic() && !has_split_string {
                    token.push(c);
                    continue;
                } else if c.is_whitespace() {
                    has_split_string = true;
                    continue;
                }
                has_split_string = false;

				wrap_ftoken(&mut tokens, &mut token, c, state);

				if c.is_numeric() {
                    state = LexState::Expression;
                } else if c == ';' || c == '&' || c == '|' {
                    state = LexState::Operator;
                }
            },
            (LexState::Expression, c) => {
                if c.is_numeric() {
                    token.push(c);
                    continue;
                } else if c.is_whitespace() { continue; }

                wrap_ftoken(&mut tokens, &mut token, c, state);

				if c.is_alphabetic() {
                    state = LexState::Identifier;
                } else if c == ';' || c == '&' || c == '|' {
                    state = LexState::Operator;
                }
            },
            (LexState::Operator, c) => {
                if !token.is_empty() {
                    /* This will always work. */
                    let Some(last_c) = token.chars().nth(0) else { return Err(LexError::MismatchOperator); };
                    match (last_c, c) {
                        ('&', '&') | ('|', '|') => {
                            token.push(c);
                            continue;
                        },
                        ('&', _) => {},
                        ('|', _) => {},
                        _ => {},
                    }
                } else if c == ';' || c == '&' || c == '|' {
                    token.push(c);
                    continue;
                } if c.is_whitespace() { continue; }

				wrap_ftoken(&mut tokens, &mut token, c, state);

				if c.is_alphabetic() {
                    state = LexState::Identifier;
                } else if c.is_numeric() {
                    state = LexState::Expression;
                }
            }
        }
    }
    if !token.is_empty() { wrap_ftoken(&mut tokens, &mut token, 'X', state); }

    Ok(tokens)
}
