use iced::{Command, Element, widget::Text};
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

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[unsafe(no_mangle)]
pub fn update(state: &mut State, message: Message) -> Command<Message> {
    match message {
        Message::Tick(instant) => {
            state.time = instant;
        }
    }

    Command::none()
}

#[unsafe(no_mangle)]
pub fn view<'a>(state: &State) -> Element<'a, Message> {
    Text::new(format!("The time is {:?}!", state.time)).into()
}
