use hot_lib::*;
use iced::{executor, Application, Command, Element, Settings, Subscription, Theme};

#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    use iced::{Command, Element};
    pub use lib::*;
    hot_functions_from_file!("./lib/src/lib.rs");
}

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

pub struct App {
    state: State,
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (App, Command<Self::Message>) {
        (
            App {
                state: State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("A hot application")
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(1000)).map(Message::Tick)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        update(&mut self.state, message)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        view(&self.state)
    }
}
