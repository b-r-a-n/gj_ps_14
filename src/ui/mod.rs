use super::*;

pub use hand::*;
pub use level_menu::*;
pub use main_menu::*;
pub use resource::*;
pub use tooltip::*;

pub mod hand;
pub mod level_menu;
pub mod main_menu;
pub mod resource;
pub mod tooltip;

pub struct GameUIPlugin;

pub struct TooltipPlugin;

pub struct MenuUIPlugin;

pub struct LevelUIPlugin;

pub fn despawn_game_ui(
    mut commands: Commands,
    game_ui: Query<Entity, Or<(With<HandUI>, With<ResourceUI>)>>,
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
    commands.add(SpawnHandUI::default());
    for player_id in players.iter() {
        commands.entity(player_id).insert(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: player_sprite_sheet.0.clone(),
            ..Default::default()
        });
    }
}

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CardClicked>()
            .add_event::<EndTurnClicked>()
            .add_systems(
                Update,
                (
                    update_energy_ui,
                    update_water_ui,
                    update_deck_ui,
                    update_recycled_ui,
                    update_discarded_ui,
                    update_hand_images,
                    update_hand_title_texts,
                    update_hand_energy_texts,
                    update_hand_water_texts,
                    update_interactions,
                    update_playable_indicator,
                    handle_click.run_if(in_state(TurnState::WaitingForInput)),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_tooltip_container,))
            .add_systems(
                Update,
                (
                    add_tooltips_to_cards,
                    update_hovered,
                    trigger_tooltip,
                    handle_hover_removed,
                ),
            );
    }
}

impl Plugin for MenuUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MainMenuEvent>()
            .add_systems(Update, main_menu::handle_interactions);
    }
}

impl Plugin for LevelUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelMenuEvent>()
            .add_systems(Update, level_menu::handle_interactions);
    }
}
