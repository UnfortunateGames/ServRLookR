mod internals;

// use std::process::exit;
use color_eyre::eyre::Result;
use internals::looker;

#[tokio::main]
async fn main() -> Result<(), ()> {
    if color_eyre::install().is_err() {
		return Err(());
    }
    return match looker::LookerState::new().run().await {
    	Ok(_) => Ok(()),
    	Err(_) => {
            println!("Failed to enter raw mode in Crossterm.");
            Err(())
        },
    }
}
