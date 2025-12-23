use std::io::{self, Write};

pub fn inputf(message: &str) -> String {
    let mut output: String = String::new();
    print!("{message}");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut output).unwrap();
    
    output
}
