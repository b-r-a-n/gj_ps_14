use super::*;
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

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), |mut commands: Commands| commands.add(SpawnEnergyUI::default()))
            .add_systems(OnEnter(GameState::Playing), |mut commands: Commands| commands.add(SpawnHandUI::default()))
            .add_systems(Update, (update_energy_ui, update_hand_ui, update_playable_indicator).run_if(in_state(GameState::Playing)))
        ;
    }
}

impl Plugin for MenuUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::MainMenu), |mut commands: Commands| commands.add(main_menu::SpawnMenuUI))
            .add_systems(OnExit(AppState::MainMenu), main_menu::despawn)
            .add_systems(Update, (main_menu::handle_interactions).run_if(in_state(AppState::MainMenu)))
        ;
    }
}

impl Plugin for LevelUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::LevelMenu), |mut commands: Commands| commands.add(level_menu::SpawnMenuUI))
            .add_systems(OnExit(AppState::LevelMenu), level_menu::despawn)
            .add_systems(Update, (level_menu::handle_interactions).run_if(in_state(AppState::LevelMenu)))
        ;
    }
}