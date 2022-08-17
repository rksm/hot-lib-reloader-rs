use hot_lib::*;

#[hot_lib_reloader::hot_module(dylib = "lib", file_watch_debounce = 500)]
mod hot_lib {
    pub use lib::*;
    pub use nannou::prelude::*;

    hot_functions_from_file!("lib/src/lib.rs");

    #[lib_version]
    pub fn version() -> usize {}
}

fn model(app: &nannou::App) -> Model {
    Model::new(app.new_window().view(view).event(event).build().unwrap())
}

pub fn update(app: &App, model: &mut Model, update: Update) {
    model.version = hot_lib::version();
    hot_lib::update(app, model, update)
}

fn main() {
    nannou::app(model).update(update).run();
}
