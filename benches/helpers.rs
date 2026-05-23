use bevy::prelude::*;
use ergoap::prelude::*;

pub fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(ErgoapPlugin);
    app
}
