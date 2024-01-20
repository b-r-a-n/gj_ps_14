use super::*;

pub use main_menu::*;
pub use level_menu::*;
pub use energy::*;
pub use hand::*;

pub mod energy;
pub mod hand;
pub mod main_menu;
pub mod level_menu;

pub struct GameUIPlugin;

pub struct MenuUIPlugin;

pub struct LevelUIPlugin;

pub fn despawn_game_ui(
    mut commands: Commands,
    game_ui: Query<Entity, Or<(With<HandUI>, With<EnergyUI>)>>,
) {
    for entity in game_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spawn_game_ui(
    mut commands: Commands,
    player_sprite_sheet: Res<PlayerSpriteSheet>,
    players: Query<Entity, With<Player>>,
) {
    commands.add(SpawnEnergyUI::default());
    commands.add(SpawnHandUI::default());
    for player_id in players.iter() {
        commands.entity(player_id)
            .insert(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: player_sprite_sheet.0.clone(),
                ..Default::default()
            });
    }
}

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (update_energy_ui, update_hand_ui, update_playable_indicator)
                .run_if(in_state(GameState::Playing)))
        ;
    }
}

impl Plugin for MenuUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MainMenuEvent>()
            .add_systems(Update, main_menu::handle_interactions)
        ;
    }
}

impl Plugin for LevelUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<LevelMenuEvent>()
            .add_systems(Update, level_menu::handle_interactions)
        ;
    }
}