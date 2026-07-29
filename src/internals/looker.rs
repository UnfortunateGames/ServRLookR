use crate::internals::{
    types::*,
    tokenizer::*,
    runner::*,
};
use crossterm::event::KeyCode;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Position, Direction},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, List, ListItem, Paragraph},
    Frame,
    Terminal,
};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Clone)]
pub struct LookerState {
	pub list: ServerList,
    pub output:   Vec<String>,
    input:    String,
    i_idx:    usize,
}

/*
fn display_tokens(ls: &mut LookerState, tokens: Vec<Token>) {
    for token in tokens {
        ls.out_print(Some("DEBUG"), format!("[{token}]").as_str());
    }
}
*/

impl LookerState {
	pub fn new() -> LookerState {
        LookerState{
			list:    ServerList::new(),
            output:  Vec::new(),
            input:   String::new(),
            i_idx:   0,
        }
    }

	fn clamp_cursor(&self, nw_cursor_pos: usize) -> usize {
		nw_cursor_pos.clamp(0, self.input.chars().count())
    }

	fn mv_cursor_l(&mut self) {
        self.i_idx = self.i_idx.saturating_sub(1);
        let nw_cursor_pos: usize = self.i_idx;

        self.clamp_cursor(nw_cursor_pos);
    }

	fn mv_cursor_r(&mut self) {
        self.i_idx = self.i_idx.saturating_add(1);
        let nw_cursor_pos: usize = self.i_idx;

        self.clamp_cursor(nw_cursor_pos);
    }

    fn byte_idx(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
			.nth(self.i_idx)
			.unwrap_or(self.input.len())
    }

    fn ins_chr(&mut self, nw_chr: char) {
        let i = self.byte_idx();
        self.input.insert(i, nw_chr);
        self.mv_cursor_r();
    }

    fn del_chr(&mut self) {
        let is_start_of_str: bool = self.i_idx == 0;
        if is_start_of_str { return; }
        let cur_idx = self.i_idx;
        let left_side =
        	self.input.chars()
                .take(cur_idx - 1);
        let right_side =
        	self.input.chars()
                .skip(cur_idx);
        self.input = left_side.chain(right_side).collect();
        self.mv_cursor_l();
    }

    pub fn out_print(&mut self, label: Option<&str>, msg: &str) {
        match label {
            Some(l_label) => self.output.push(format!(" [ {l_label} ] {msg}\n")),
            None => self.output.push(format!(" {msg}\n")),
        }
    }

	fn render(&mut self, frame: &mut Frame) {
		let main_lo = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(frame.area());
        let out_lo = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .split(main_lo[0]);
        let [slist_area, output_area, input_area, tip_area] = [
            out_lo[0], out_lo[1], main_lo[1], main_lo[2],
        ];

		let msg = vec![
                "Start ".into(),
                "typing ".bold(),
                "to enter commands. ".into(),
                "For more details enter ".into(),
                "'help'.".italic()
            ];
		let style = Style::default();
        let text = Text::from(Line::from(msg)).patch_style(style);
        let tip_msg = Paragraph::new(text);
        frame.render_widget(tip_msg, tip_area);

        let input_msg: String = format!(" -> {}", self.input);
        let input = Paragraph::new(input_msg.as_str())
            .style(Style::default().fg(Color::Blue))
            .block(Block::bordered().title(" Enter a command: "));
        frame.render_widget(input, input_area);
        frame.set_cursor_position(Position::new(
            input_area.x + self.i_idx as u16 + 5,
            input_area.y + 1
        ));

		while self.output.len() > (output_area.height - 2) as usize {
            self.output.remove(0);
        }
        let out: Vec<ListItem> = self.output
            .iter()
            .map(|line| ListItem::new(line.to_owned()))
            .collect();
        let out_msg = List::new(out).block(Block::bordered().title("Output"));
        frame.render_widget(out_msg, output_area);

        let sl: Vec<ListItem> = self.list.return_serverlist()
            .lines()
            .map(|line| ListItem::new(line.to_owned()))
            .collect();
        let sl_msg = List::new(sl).block(Block::bordered().title("ServerList"));
        frame.render_widget(sl_msg, slist_area);
    }

    pub async fn get_user_input(
        &mut self,
        receiver: &mut UnboundedReceiver<AppEvent>,
        term: &mut Terminal<CrosstermBackend<std::io::Stdout>>
    ) -> Option<String> {
        let user_input: String;
        loop {
			let received: AppEvent = match receiver.recv().await {
				Some(evt) => evt,
	            None => { continue; }
	        };
	    	match received {
	    	    AppEvent::Error => return None,
	    	    AppEvent::Tick => for call in self.list.call_running() {
                    self.out_print(None, &call);
	    	    },
	    	    AppEvent::Render => if term.draw(|frame| self.render(frame)).is_err() {
                    return None;
        	    },
	    	    AppEvent::Key(key) => match key.code {
        	        KeyCode::Backspace => self.del_chr(),
        	        KeyCode::Enter => {
						self.i_idx = 0;
        	            // if self.input.to_lowercase() == "exit" { std::process::exit(0); }
                        user_input = self.input.clone();
        	            self.input.clear();
                        return Some(user_input);
        	        },
        	        KeyCode::Char(chr) => self.ins_chr(chr),
        	        _ => {}
	    	    },
        	}
        }
    }

	pub async fn run(&mut self) -> Result<(), ()> {
		let mut be: Backend = Backend::new(30.0, 1.0);
        let mut term;
        let be_term = ratatui::backend::CrosstermBackend::new(std::io::stdout());
        match ratatui::Terminal::new(be_term) {
            Ok(out) => term = out,
            Err(_) => return Err(())
        };
	    if crossterm::terminal::enable_raw_mode().is_err() { return Err(()); }
        if term.clear().is_err() { return Err(()); }
		
	    be.start();
	
		loop {
			let Some(_input) = self.get_user_input(&mut be.receiver, &mut term).await else { break; };
            let tokens: Vec<Token>;
            match tokenize_cmd(&_input) {
                Ok(rv) => tokens = rv,
                Err(err) => {
                    self.out_print(Some("DEVERROR"),
                	    match err {
                	    	LexError::MismatchIdentifier => "Identifier Mismatch.",
                	    	LexError::MismatchExpression => "Expression Mismatch.",
                	    	LexError::MismatchOperator   => "Operator Mismatch.",
                		}
                	);
                    continue;
                },
            }
            let instr_list: Vec<Statement>;
            match parse_tokens(tokens) {
                Ok(rv) => instr_list = rv,
                Err(err) => {
                    self.out_print(Some("ERROR"),
                        match err {
                            ParseError::ExpectedXGotY(x, y) =>
								format!("Expected {x}, got {y}."),
                            ParseError::InvalidCommandX(x) =>
                            	format!("Invalid command {x}.")
                        }.as_str()
                    );
                    continue;
                }
            }
            if run_instr_list(instr_list, self, &mut be.receiver, &mut term).await { break; }
        }
	
	    match crossterm::terminal::disable_raw_mode() {
			Ok(_) => Ok(()),
	        Err(_) => Err(())
	    }
	}
}
