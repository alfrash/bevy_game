// Bring in the core Bevy prelude, which includes all the most common types and traits you'll need.
use bevy::prelude::*;

// Import our custom PlayerPlugin from the player module.
use crate::player::PlayerPlugin;

// Declare the `player` module. This tells Rust to look for a `player.rs` file or a `player/mod.rs` file.
mod player;

fn main() {
    // `App::new()` creates a new Bevy application. Everything in Bevy revolves around the App.
    App::new()
        // `insert_resource` adds a global resource to the game. Here, we set the background clear color to white.
        .insert_resource(ClearColor(Color::WHITE))
        // `add_plugins` adds a group of plugins. `DefaultPlugins` includes essential engine features 
        // like window creation, input handling, rendering, and asset loading.
        .add_plugins(
            DefaultPlugins.set(AssetPlugin {
                // Here we configure the AssetPlugin to look for assets in the "src/assets" folder 
                // instead of the default "assets" folder.
                file_path: "src/assets".into(),
                ..default()
            }),
        )
        // `add_systems` registers a system to run. `Startup` means it runs exactly once when the game starts.
        .add_systems(Startup, setup_camera)
        // Add our custom PlayerPlugin, which contains all the logic and systems related to the player.
        .add_plugins(PlayerPlugin) // Update this line
        // Finally, run the application loop!
        .run();
}

// Systems are just regular Rust functions that take special arguments provided by Bevy (like Commands, Res, Query).
// `mut commands: Commands` allows us to spawn or despawn entities in the world.
fn setup_camera(mut commands: Commands){
    // Spawn a basic 2D orthographic camera so we can see stuff on the screen.
    // Without a camera, the window would just show the ClearColor.
    commands.spawn(Camera2d);
}
