use bevy::prelude::*;
use systems::*;

#[cfg(feature = "reload")]
hot_lib_reloader::define_lib_reloader! {
    unsafe SystemsReloader {
        lib_name: "systems",
        source_files: ["../systems/src/lib.rs"],
        generate_bevy_systems: true,
    }
}

#[cfg(feature = "reload")]
struct LibLoaderUpdateTimer(Timer);

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(player_movement_system)
        .add_system(bevy::window::close_on_esc);

    #[cfg(feature = "reload")]
    app.add_startup_system(setup_hot_reload)
        .add_system(update_lib);

    app.run();
}

#[cfg(feature = "reload")]
pub fn setup_hot_reload(mut commands: Commands) {
    let lib = SystemsReloader::new().expect("init lib loader");
    commands.insert_resource(lib);
    commands.insert_resource(LibLoaderUpdateTimer(Timer::from_seconds(1.0, true)));
}

#[cfg(feature = "reload")]
fn update_lib(
    time: Res<Time>,
    mut lib: ResMut<SystemsReloader>,
    mut timer: ResMut<LibLoaderUpdateTimer>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        lib.update().expect("update lib");
    }
}
