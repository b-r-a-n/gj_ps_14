use super::*;

pub use actions::*;
use bevy::utils::HashMap;
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

pub fn spawn_level() {
    // Load in the card info (maybe this should happen earlier, e.g. the app could invoke it)
    // Spawn the player, cards, and tiles based on the level info
    // Add the cards to the player's deck
    // Add the properties to to the tile grid
}

pub fn despawn_level() {}

pub fn shuffle_deck(
    mut deck: Query<&mut Deck, With<Player>>,
) {
    deck.get_single_mut().expect("Should have 1 deck").shuffle();
}

#[derive(Resource)]
pub struct DeckList(pub Vec<ContentID>);

impl Default for DeckList {

    fn default() -> Self {
        Self(vec![
            1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4]
            .iter()
            .map(|id| ContentID(*id))
            .collect())
    }
}

pub fn spawn_cards(
    mut commands: Commands,
    mut deck: Query<&mut Deck, With<Player>>,
    deck_list: Res<DeckList>,
    card_sprites: Res<CardSpriteSheet>,
) {
    let mut deck = deck.get_single_mut().expect("Should be exactly 1 deck");
    for content_id in deck_list.0.iter() {
        let card_instance_id = commands.spawn((
            content_id.clone(),
            card_sprites.0.clone(),
            InDeck,
        )).id();
        deck.add(card_instance_id);
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
    card_instances: Query<Entity, With<ContentID>>,
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
            .init_resource::<CardInfoMap>()
            .init_resource::<DeckList>()

            .insert_resource(MapParameters {
                columns: 3,
                rows: 1,
                flame_spawner: FlameSpawner::Static(vec![(3, 1)]),
            })

            .add_plugins(ui::GameUIPlugin)
            .add_state::<GameState>()
            .add_state::<TurnState>()

            .add_systems(OnTransition { from: GameState::None, to: GameState::Loading }, (
                load_card_infos,
                spawn_player, 
                // spawn_object_infos,
                schedule_transition::<NextGameState>
            ))
            .add_systems(OnEnter(GameState::None), (
                despawn_card_infos,
                despawn_player,
                despawn_tiles,
                // despawn_object_infos,
            ))
            .add_systems(OnTransition { from: GameState::Loading, to: GameState::Loaded }, (
                spawn_cards,
                spawn_tiles,
                schedule_transition::<NextGameState>
            ))
            .add_systems(OnTransition { from: GameState::Loaded, to: GameState::Playing }, (
                shuffle_deck, 
                spawn_fires,
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
