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
            CardStatus::Unknown,
        )).id();
        deck.add(card_instance_id);
    }
}

pub fn despawn_cards(
    mut commands: Commands,
    cards: Query<Entity, With<ContentID>>,
) {
    for card_id in cards.iter() {
        commands.entity(card_id).despawn_recursive();
    }
}

const SPAWN_POINTS : [[Option<(i32, i32)>; 5]; 5] = [
    [Some((3, 1)), None, None, None, None],
    [Some((2, 2)), None, None, None, None],
    [Some((3, 1)), Some((3, 2)), None, None, None],
    [Some((2, 2)), Some((3, 2)), None, None, None],
    [Some((2, 2)), Some((3, 2)), Some((3, 1)), None, None],
];
const MAP_SIZES : [(i32, i32); 5] = [
    (3, 1),
    (3, 3),
    (4, 4),
    (5, 5),
    (6, 6),
];
pub fn prepare_for_new_level(
    mut map: ResMut<MapParameters>,
    mut deck: Query<&mut Deck, With<Player>>,
    mut hand: Query<&mut Hand, With<Player>>,
    mut position: Query<&mut GamePosition, With<Player>>,
    mut level_index: Local<usize>,
) {
    // Update the MapParameters resource
    *map = MapParameters {
        columns: MAP_SIZES[*level_index].0,
        rows: MAP_SIZES[*level_index].1,
        flame_spawner: FlameSpawner::Static(SPAWN_POINTS[*level_index].iter().cloned().flatten().collect()),
    };
    *level_index += 1;
    *level_index %= SPAWN_POINTS.len();

    // Reset the transitory player state
    let mut position = position.get_single_mut().expect("Should be exactly 1 player");
    position.x = 1;
    position.y = 1;
    position.d = GameDirection::Up;

    // Reset the transitory card state
    deck.get_single_mut().expect("Should be exactly 1 deck").reset();
    hand.get_single_mut().expect("Should be exactly 1 deck").reset();
}

pub fn start_turn(
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
    tiles: Query<&Tile>,
    changes: Query<Entity, Changed<Tile>>,
) {
    if changes.is_empty() {
        return;
    }
    let mut fire_count = 0;
    let mut empty_count = 0;
    for tile in tiles.iter() {
        match tile {
            Tile::Fire(_) => fire_count += 1,
            Tile::Empty => empty_count += 1,
            _ => {}
        }
    }
    info!("Fire count: {} | Empty count: {}", fire_count, empty_count);
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

fn put_flames_out(
    mut tiles: Query<&mut Tile>,
    pos: Query<&GamePosition, (With<Player>, Changed<GamePosition>)>,
    grid: Query<&Grid>,
) {
    if pos.is_empty() {
        return;
    }
    let pos = pos.get_single().expect("Should only be one player position");
    let grid = grid.get_single().expect("Failed to get grid");
    let tile_id = grid.get(pos);
    let mut tile = tiles.get_mut(tile_id).expect("Failed to get tile");
    *tile = Tile::Empty;
}


#[derive(Component)]
pub enum CardStatus {
    Playable,
    Unplayable,
    Unknown,
}

impl CardStatus {
    pub fn is_playable(&self) -> bool {
        match self {
            CardStatus::Playable => true,
            _ => false,
        }
    }
}

fn update_playability(
    player_info: Query<(&GamePosition, &Energy, &Water, &Hand), With<Player>>,
    mut card_instances: Query<(&ContentID, &mut CardStatus)>,
    card_info: Res<CardInfoMap>,
    tile_grid: Query<&Grid>,
    blocked_tiles: Query<&BlockedTile>,

) {
    let (position, energy, water, hand) = player_info.get_single()
        .expect("Should be exactly 1 player");
    let tile_grid = tile_grid.get_single().expect("Failed to get tile grid");
    for card_instance_id in hand.0.iter().flatten() {
        let (content_id, mut status) = card_instances.get_mut(*card_instance_id)
            .expect("Failed to get card instance");
        let card_info = card_info.0.get(&*content_id)
            .expect("Failed to get card info");
        if card_info.resource_cost.energy > energy.current || card_info.resource_cost.water > water.current {
            *status = CardStatus::Unplayable;
            continue;
        }
        match &card_info.position_change {
            MovementInfo { position: TileTarget::Offset(dist), rotation: rot} => {
                let target_pos = position.rotated(rot).offset(*dist);
                let tile_id = tile_grid.get(&target_pos);
                if blocked_tiles.get(tile_id).is_ok() {
                    *status = CardStatus::Unplayable;
                    continue;
                } 
            }
            _ => {}
        }
        *status = CardStatus::Playable;
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
            .init_resource::<MapParameters>()

            .add_plugins(ui::GameUIPlugin)
            .add_state::<GameState>()
            .add_state::<TurnState>()

            .add_systems(OnTransition { from: GameState::None, to: GameState::Loading }, (
                load_card_infos,
                spawn_player, 
                schedule_transition::<NextGameState>
            ))
            .add_systems(OnEnter(GameState::None), (
                despawn_card_infos,
                despawn_player,
            ))
            .add_systems(OnExit(GameState::Playing), (
                despawn_tiles,
                despawn_cards,
            ))
            .add_systems(OnEnter(GameState::Loaded), (
                prepare_for_new_level,
                spawn_cards,
                spawn_tiles,
            ).chain())
            .add_systems(Update,
                (|mut next_state: ResMut<NextState<GameState>>| { next_state.set(GameState::Playing); })
                    .run_if(in_state(AppState::Game))
                    .run_if(in_state(GameState::Loaded))
            )
            .add_systems(OnTransition { from: GameState::Loaded, to: GameState::Playing }, (
                shuffle_deck, 
                schedule_transition::<NextTurnState>
            ))
            .add_systems(OnEnter(TurnState::Starting), (
                fill_hand_with_cards, 
                restore_energy,
                )
                .run_if(in_state(GameState::Playing)
            ))
            .add_systems(Update, (
                start_turn,
                )
                .run_if(in_state(GameState::Playing))
                .run_if(in_state(TurnState::Starting)))
            .add_systems(OnEnter(TurnState::Started), (
                update_playability,
                check_for_turn_over,
                ).chain().run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(TurnState::Animating), (
                animate_cards,
            ))
            .add_systems(Update, (
                transition::<GameState, NextGameState>,
                transition::<TurnState, NextTurnState>
            ))
            .add_systems(OnEnter(TurnState::Ended), (
                propagate_flames,
                grow_flames,
                |mut next_state: ResMut<NextState<TurnState>>| next_state.set(TurnState::Starting),
            ).chain())
            .add_systems(Update, (
                apply_change::<GamePosition>, 
                apply_change::<Energy>, 
                apply_change::<Water>, 
                apply_card,
                apply_card_actions, 
                check_for_level_end,
                sync_deck, 
                sync_hand, 
                update_tiles,
                put_flames_out,
                ).run_if(in_state(GameState::Playing)))
            ;
    }
}
