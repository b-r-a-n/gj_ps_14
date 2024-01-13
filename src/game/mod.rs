use super::*;
pub use cards::*;
pub use actions::*;
pub use player::*;
pub use stats::*;

mod actions;
mod cards;
mod player;
mod stats;

// TODO: This sync methods don't work for removed cards

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerSpriteSheet>()
            .add_event::<CardEvent>()
            .add_systems(Startup, |mut commands: Commands| commands.add(SpawnPlayer { max_energy: 100, ..default() }))
            .add_systems(Update, (apply_change::<GamePosition>, apply_change::<Energy>))
            .add_systems(Update, (handle_card_events, apply_card_actions, announce_card_actions))
            .add_systems(Update, (sync_deck, sync_hand));
    }
}
