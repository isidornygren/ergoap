mod goal_provider;
mod picking;
mod score;

use bevy_app::{App, Update};

use crate::utility::picking::picking_system;
pub use goal_provider::{GoalProvider, GoalProviderBuilder, GoalProviderTrait};
pub use picking::Picker;
pub use score::Score;

pub trait Scorer {
    fn score(&self) -> Score;
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, picking_system);
}
