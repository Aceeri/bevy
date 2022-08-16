//! Shows how to return to the calling function after a windowed Bevy app has exited.

use bevy::{prelude::*, window::WindowPlugin, winit::WinitSettings};

fn main() {
    println!("Running Bevy App");
    App::new()
        .insert_resource(WinitSettings {
            return_from_run: true,
            ..default()
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Close the window to return to the main function".into(),
                ..default()
            }),
            ..default()
        }))
        .add_system(system)
        .run();
    println!("Bevy App has exited. We are back in our main function.");
}

fn system() {
    info!("Logging from Bevy App");
}
