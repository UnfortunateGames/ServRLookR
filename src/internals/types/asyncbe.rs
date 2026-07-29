use futures::{StreamExt, FutureExt};
use crossterm::event::{KeyEvent, EventStream, KeyEventKind, Event};
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use std::time::Duration;

#[derive(Clone)]
pub enum AppEvent {
	Error,
    Tick,
    Render,
    Key(KeyEvent),
}

pub struct Backend {
    pub receiver:   UnboundedReceiver<AppEvent>,
    pub sender: 	UnboundedSender<AppEvent>,
    pub frame_rate: f64,
    pub tick_rate:  f64
}

impl Backend {
    pub fn new(fr: f64, tr: f64) -> Backend {
        let (tx, rx) = unbounded_channel::<AppEvent>();

        Backend{
            receiver:   rx,
            sender:     tx,
            frame_rate: fr,
            tick_rate:  tr
        }
    }

    pub fn start(&mut self) {
		let _sender = self.sender.clone();
		let tick_delay = Duration::from_secs_f64(1.0 / self.tick_rate);
        let render_delay = Duration::from_secs_f64(1.0 / self.frame_rate);

        tokio::spawn(async move{
            let mut reader = EventStream::new();
            let mut tick_interval =
            	tokio::time::interval(tick_delay); 
			let mut render_interval =
            	tokio::time::interval(render_delay);

            loop {
                let tick_delay = tick_interval.tick();
                let render_delay = render_interval.tick();
            	let cterm_evt = reader.next().fuse();

                tokio::select! {
					maybe_event = cterm_evt => {
						match maybe_event {
							Some(Ok(Event::Key(key))) => {
                                if key.kind == KeyEventKind::Press {
                                    _sender.send(AppEvent::Key(key)).unwrap();
                                }
                            },
                            Some(Ok(_)) => {},
                            Some(Err(_)) => {
								_sender.send(AppEvent::Error).unwrap();
                            },
                            None => {}
                        }
                    },
                    _ = tick_delay => {
                        _sender.send(AppEvent::Tick).unwrap();
                    }
                    _ = render_delay => {
						_sender.send(AppEvent::Render).unwrap();
                    }
                }
            }
        });
    }	
}
