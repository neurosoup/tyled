use bevy::prelude::*;
use tyled::{AppPlugin, launch_help};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(help) = launch_help(&args) {
        println!("{help}");
        return;
    }
    App::new().add_plugins(AppPlugin).run();
}
