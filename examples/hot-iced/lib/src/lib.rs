use iced::{widget::Text, Command, Element};
use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Tick(Instant),
}

pub struct State {
    pub time: Instant,
}

impl State {
    pub fn new() -> Self {
        Self {
            time: Instant::now(),
        }
    }
}

#[no_mangle]
pub fn update(state: &mut State, message: Message) -> Command<Message> {
    match message {
        Message::Tick(instant) => {
            state.time = instant;
        }
    }

    Command::none()
}

#[no_mangle]
pub fn view<'a>(state: &State) -> Element<'a, Message> {
    Text::new(format!("The time is {:?}!", state.time)).into()
}
