use bevy::prelude::*;

#[cfg(not(feature = "reload"))]
use systems::*;
#[cfg(feature = "reload")]
use systems_hot::*;

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "systems")]
mod systems_hot {
    use bevy::prelude::*;
    pub use components::*;
    hot_functions_from_file!("systems/src/lib.rs");
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_startup_system(systems::setup)
        .add_system_set(
            SystemSet::new()
                .with_system(player_movement_system)
                .with_system(player_shooting_system)
                .with_system(bullet_movement_system)
                .with_system(bullet_hit_system)
                .with_system(spawn_other_ships)
                .with_system(move_other_ships),
        )
        .add_system(bevy::window::close_on_esc);

    app.run();
}
