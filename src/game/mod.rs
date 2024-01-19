use super::*;

pub use actions::*;
use bevy::transform::commands;
pub use cards::*;
pub use game::*;
pub use player::*;
use rand::Rng;
pub use stats::*;
pub use card::*;
pub use tiles::*;


mod actions;
mod cards;
mod card;
mod game;
mod player;
mod stats;
mod tiles;

pub fn add_cards_to_deck(
    mut deck: Query<&mut Deck, With<Player>>,
    card_instances: Query<Entity, With<BaseCardInfo>>,
) {
    let mut deck = deck.get_single_mut().expect("Should be exactly 1 deck");
    for card_instance_id in card_instances.iter() {
        info!("Adding card {:?} to deck", card_instance_id);
        deck.add(card_instance_id);
    }
    deck.shuffle();
}

pub fn spawn_cards(
    mut commands: Commands,
    card_infos: Query<Entity, With<CardInfo>>,
) {
    for card_info_id in card_infos.iter() {
        (0..10).for_each(|_| {
            commands.add(SpawnCard {
                base_card_info: card_info_id,
            });
        });
    }
}

pub fn check_for_turn_ready(
    pending_actions: Query<(Entity, &CardActionType)>,
    mut pending_action_count: Local<usize>,
    mut next_turn_state: ResMut<NextState<TurnState>>,
) {
    if pending_actions.iter().len() != *pending_action_count {
        *pending_action_count = pending_actions.iter().len();
    }
    if pending_actions.iter().len() == 0 {
        next_turn_state.set(TurnState::Started);
    }
}

fn cleanup_temporary_state(
    mut commands: Commands,
    card_instances: Query<Entity, With<BaseCardInfo>>,
) {
    for card_instance_id in card_instances.iter() {
        commands.entity(card_instance_id)
            .remove::<Playable>()
            .remove::<NeedsEnergy>()
            .remove::<NeedsMoveable>()
            .remove::<NeedsWater>()
            .remove::<WasPlayed>();
    }
}

fn restore_energy(
    mut energy: Query<&mut Energy, With<Player>>,
) {
    let mut energy = energy.get_single_mut().expect("Should be exactly 1 energy");
    energy.current = energy.maxium/2;
}

fn grow_flames(
    mut tiles: Query<&mut Tile>,
) {
    for mut tile in tiles.iter_mut() {
        match *tile {
            Tile::Fire(Intensity::Low) => {
                *tile = Tile::Fire(Intensity::Medium);
            },
            Tile::Fire(Intensity::Medium) => {
                *tile = Tile::Fire(Intensity::High);
            },
            _ => {}
        }
    }
}

fn propagate_flames(
    mut commands: Commands,
    tiles: Query<(Entity, &Tile)>,
    positions: Query<&GamePosition>,
    grid: Query<&Grid>,
) {
    let grid = grid.get_single().expect("Failed to get grid");
    for (tile_id, tile) in tiles.iter() {
        if let Tile::Fire(Intensity::High) = tile {
            let position = positions.get(tile_id)
                .expect("Failed to get position");
            for neighbor in grid.neighbors(position) {
                if let Ok((neighbor_id, neighbor_tile)) = tiles.get(neighbor) {
                    if let Tile::Empty = *neighbor_tile {
                        commands.entity(neighbor_id)
                            .insert(Tile::Fire(Intensity::Low));
                    }
                }
            }
        }
    }
}

fn check_for_level_end(
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_turn_state: ResMut<NextState<TurnState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    tiles: Query<&Tile>
) {
    let mut fire_count = 0;
    let mut empty_count = 0;
    for tile in tiles.iter() {
        match tile {
            Tile::Fire(_) => fire_count += 1,
            Tile::Empty => empty_count += 1,
            _ => {}
        }
    }
    if empty_count == 0 {
        info!("Level ended | Failure");
        next_app_state.set(AppState::MainMenu);
        next_game_state.set(GameState::None);
        next_turn_state.set(TurnState::None);
    } else if fire_count == 0 {
        info!("Level ended | Success");
        next_app_state.set(AppState::LevelMenu);
        next_game_state.set(GameState::Loaded);
        next_turn_state.set(TurnState::None);
    }
}


pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerSpriteSheet>()
            .init_resource::<CardSpriteSheet>()
            .init_resource::<TileSpriteSheet>()
            .add_plugins(ui::GameUIPlugin)
            .add_state::<GameState>()
            .add_state::<TurnState>()
            .add_systems(OnTransition { from: AppState::MainMenu, to: AppState::LevelMenu }, (
                |mut game_state: ResMut<NextState<GameState>>| game_state.set(GameState::Loading),
            ))
            .add_systems(OnExit(AppState::Game), (
                ui::despawn_game_ui,
            ))
            .add_systems(OnTransition { from: GameState::None, to: GameState::Loading }, (
                load_card_infos, 
                schedule_transition::<NextGameState>
            ))
            .add_systems(OnTransition { from: AppState::LevelMenu, to: AppState::Game }, (
                spawn_player, 
                spawn_cards,
                spawn_tiles,
                schedule_transition::<NextGameState>
            ).run_if(in_state(GameState::Loaded)))
            .add_systems(OnTransition { from: GameState::Loaded, to: GameState::Playing }, (
                add_cards_to_deck, 
                add_random_fire_tiles,
                schedule_transition::<NextTurnState>
            ))
            .add_systems(OnEnter(TurnState::Starting), (
                fill_hand_with_cards, 
                )
                .chain()
                .run_if(in_state(GameState::Playing)
            ))
            .add_systems(Update, (
                check_for_turn_ready,
                )
                .run_if(in_state(GameState::Playing))
                .run_if(in_state(TurnState::Starting)))
            .add_systems(OnEnter(TurnState::Started), (
                restore_energy,
                schedule_transition::<NextTurnState>
                ).run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(TurnState::Animating), (
                animate_cards,
                cleanup_temporary_state,
            ))
            .add_systems(Update, (
                check_for_turn_over
                )
                .run_if(in_state(TurnState::WaitingForInput))
                .run_if(in_state(GameState::Playing))
            )
            .add_systems(Update, (
                transition::<GameState, NextGameState>,
                transition::<TurnState, NextTurnState>
            ))
            .add_systems(OnEnter(TurnState::Ended), (
                propagate_flames,
                grow_flames,
                check_for_level_end,
                schedule_transition::<NextTurnState>
            ).chain())
            .add_systems(Update, (
                apply_change::<GamePosition>, 
                apply_change::<Energy>, 
                apply_change::<Water>, 
                apply_card,
                apply_card_actions, 
                sync_deck, 
                sync_hand, 
                update_playable, 
                handle_card_added_to_hand,
                handle_resource_change,
                handle_position_change,
                log_playability_changes,
                update_tiles,
                ).run_if(in_state(GameState::Playing)))
            ;
    }
}
