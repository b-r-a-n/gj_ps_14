use super::*;

pub use actions::*;
use bevy::utils::HashMap;
pub use card::*;
pub use cards::*;
pub use game::*;
pub use items::*;
pub use player::*;
use rand::Rng;
pub use stats::*;
pub use tiles::*;

mod actions;
mod card;
mod cards;
mod game;
mod items;
mod player;
mod stats;
mod tiles;

pub fn shuffle_deck(mut deck: Query<&mut Deck, With<Player>>) {
    deck.get_single_mut().expect("Should have 1 deck").shuffle();
}

#[derive(Resource)]
pub struct DeckList(pub Vec<ContentID>);

impl Default for DeckList {
    fn default() -> Self {
        Self(
            vec![1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4]
                .iter()
                .map(|id| ContentID(*id))
                .collect(),
        )
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
        let card_instance_id = commands
            .spawn((
                content_id.clone(),
                card_sprites.0.clone(),
                InDeck,
                CardStatus::Unknown,
            ))
            .id();
        deck.add(card_instance_id);
    }
}

pub fn despawn_cards(mut commands: Commands, cards: Query<Entity, With<ContentID>>) {
    for card_id in cards.iter() {
        commands.entity(card_id).despawn_recursive();
    }
}

const SPAWN_POINTS: [[Option<(i32, i32)>; 5]; 5] = [
    [Some((1, 3)), None, None, None, None],
    [Some((2, 2)), None, None, None, None],
    [Some((3, 3)), None, None, None, None],
    [Some((4, 1)), Some((3, 3)), None, None, None],
    [Some((2, 2)), Some((3, 2)), Some((3, 1)), None, None],
];
const MAP_SIZES: [(i32, i32); 5] = [(1, 3), (3, 3), (3, 3), (5, 5), (6, 6)];
const DECK_LISTS: [[Option<usize>; 16]; 5] = [
    [
        Some(1),
        Some(5),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(1),
        Some(5),
        Some(3),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(1),
        Some(5),
        Some(3),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(1),
        Some(5),
        Some(3),
        Some(4),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(1),
        Some(5),
        Some(3),
        Some(4),
        Some(8),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
];

#[derive(Default, Resource)]
pub enum GameMode {
    #[default]
    Puzzle,
    Rogue,
}
fn prepare_for_rogue_level(map: &mut MapParameters, _deck_list: &mut DeckList, level_index: i32) {
    info!("Prepping for rogue level with index {}", level_index);
    let (c, r) = (map.columns.max(1), map.rows.max(1));
    *map = MapParameters {
        columns: c + 1,
        rows: r + 1,
        flame_spawner: Spawner::Chance(0.1, 1, level_index.max(1)),
        item_spawner: Spawner::Chance(0.5, 1, 1),
    };
}

fn prepare_for_puzzle_level(map: &mut MapParameters, deck_list: &mut DeckList, level_index: i32) {
    let level_index = (level_index as usize) % SPAWN_POINTS.len();
    *map = MapParameters {
        columns: MAP_SIZES[level_index as usize].0,
        rows: MAP_SIZES[level_index as usize].1,
        flame_spawner: Spawner::Static(
            SPAWN_POINTS[level_index as usize]
                .iter()
                .cloned()
                .flatten()
                .collect(),
        ),
        item_spawner: Spawner::Static(vec![]),
    };
    deck_list.0 = DECK_LISTS[level_index]
        .iter()
        .flatten()
        .cloned()
        .map(|id| ContentID(id))
        .collect();
}

#[derive(Resource)]
pub struct LevelIndex(pub i32);

impl Default for LevelIndex {
    fn default() -> Self {
        Self(0)
    }
}

pub fn prepare_for_new_level(
    mode: Res<GameMode>,
    mut map: ResMut<MapParameters>,
    mut deck_list: ResMut<DeckList>,
    mut deck: Query<&mut Deck, With<Player>>,
    mut hand: Query<&mut Hand, With<Player>>,
    mut position: Query<&mut GamePosition, With<Player>>,
    level_index: Res<LevelIndex>,
) {
    match *mode {
        GameMode::Puzzle => {
            prepare_for_puzzle_level(&mut map, &mut deck_list, level_index.0);
        }
        GameMode::Rogue => prepare_for_rogue_level(&mut map, &mut deck_list, level_index.0),
    }
    // Reset the transitory player state
    let mut position = position
        .get_single_mut()
        .expect("Should be exactly 1 player");
    position.x = 1;
    position.y = 1;
    position.d = GameDirection::Up;

    // Update the deck list

    // Reset the transitory card state
    deck.get_single_mut()
        .expect("Should be exactly 1 deck")
        .reset();
    hand.get_single_mut()
        .expect("Should be exactly 1 deck")
        .reset();
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

fn restore_resources(mut energy: Query<(&mut Energy, &mut Water), With<Player>>) {
    let (mut energy, mut water) = energy.get_single_mut().expect("Should be exactly 1 energy");
    energy.current = energy.maxium / 2;
    water.current = water.maxium / 2;
}

fn grow_flames(mut tiles: Query<&mut Tile>) {
    for mut tile in tiles.iter_mut() {
        match *tile {
            Tile::Fire(Intensity::Low) => {
                *tile = Tile::Fire(Intensity::Medium);
            }
            Tile::Fire(Intensity::Medium) => {
                *tile = Tile::Fire(Intensity::High);
            }
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
            let position = positions.get(tile_id).expect("Failed to get position");
            for neighbor in grid.neighbors(position) {
                if let Ok((neighbor_id, neighbor_tile)) = tiles.get(neighbor) {
                    if let Tile::Empty = *neighbor_tile {
                        commands
                            .entity(neighbor_id)
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
    mut level_index: ResMut<LevelIndex>,
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
        level_index.0 += 1;
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
    let pos = pos
        .get_single()
        .expect("Should only be one player position");
    let grid = grid.get_single().expect("Failed to get grid");
    let tile_id = grid.get(pos).expect("Failed to get tile id");
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

fn play_clicked_card(
    mut commands: Commands,
    mut events: EventReader<CardClicked>,
    deck: Query<(Entity, &Deck), With<Player>>,
    hand: Query<(Entity, &Hand), With<Player>>,
    status: Query<&CardStatus>,
) {
    for event in events.read() {
        let ((deck_id, _), (hand_id, _)) = (
            deck.get_single().expect("Should be exactly 1 deck"),
            hand.get_single().expect("Should be exactly 1 hand"),
        );
        if let Some(card_instance_id) = event.card_instance.0 {
            if let Ok(card_status) = status.get(card_instance_id) {
                if let CardStatus::Playable = card_status {
                    commands.spawn(CardActionType::Play(Play {
                        card: card_instance_id,
                        deck: deck_id,
                        hand: hand_id,
                    }));
                }
            }
        }
    }
}

fn end_turn_clicked(
    mut events: EventReader<EndTurnClicked>,
    mut turn_state: ResMut<NextState<TurnState>>,
) {
    let mut turn_end_event = false;
    for _ in events.read() {
        turn_end_event = true;
    }
    if turn_end_event {
        turn_state.set(TurnState::Ended);
    }
}

fn update_playability(
    player_info: Query<(&GamePosition, &Energy, &Water, &Hand), With<Player>>,
    mut card_instances: Query<(&ContentID, &mut CardStatus)>,
    card_info: Res<CardInfoMap>,
    tile_grid: Query<&Grid>,
    blocked_tiles: Query<&BlockedTile>,
) {
    let (position, energy, water, hand) = player_info
        .get_single()
        .expect("Should be exactly 1 player");
    let tile_grid = tile_grid.get_single().expect("Failed to get tile grid");
    for card_instance_id in hand.0.iter().flatten() {
        let (content_id, mut status) = card_instances
            .get_mut(*card_instance_id)
            .expect("Failed to get card instance");
        let card_info = card_info
            .0
            .get(&*content_id)
            .expect("Failed to get card info");
        if card_info.resource_cost.energy > energy.current
            || card_info.resource_cost.water > water.current
        {
            *status = CardStatus::Unplayable;
            continue;
        }
        match &card_info.position_change {
            MovementInfo {
                position: TileTarget::FacingDist(dist),
                rotation: rot,
            } => {
                let target_pos = position.rotated(rot).offset((*dist, 0));
                let tile_id = tile_grid.get(&target_pos);
                if tile_id.is_none() || blocked_tiles.get(tile_id.unwrap()).is_ok() {
                    *status = CardStatus::Unplayable;
                    continue;
                }
            }
            _ => {}
        }
        *status = CardStatus::Playable;
    }
}

fn reset_game(
    mut deck_list: ResMut<DeckList>,
    mut level_index: ResMut<LevelIndex>,
    mut map_parameters: ResMut<MapParameters>,
) {
    *deck_list = DeckList::default();
    *level_index = LevelIndex::default();
    *map_parameters = MapParameters::default();
}
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSpriteSheet>()
            .init_resource::<CardSpriteSheet>()
            .init_resource::<TileSpriteSheet>()
            .init_resource::<IconSpriteSheet>()
            .init_resource::<ItemSpriteSheet>()
            .init_resource::<CardInfoMap>()
            .init_resource::<DeckList>()
            .init_resource::<MapParameters>()
            .init_resource::<GameMode>()
            .init_resource::<LevelIndex>()
            .add_plugins(ui::GameUIPlugin)
            .add_state::<GameState>()
            .add_state::<TurnState>()
            .add_systems(
                OnTransition {
                    from: GameState::None,
                    to: GameState::Loading,
                },
                (
                    load_card_infos,
                    spawn_player,
                    schedule_transition::<NextGameState>,
                ),
            )
            .add_systems(
                OnEnter(GameState::None),
                (
                    despawn_card_infos,
                    despawn_player,
                    despawn_tiles_and_items,
                    despawn_cards,
                    reset_game,
                ),
            )
            .add_systems(Update, reset_game.run_if(resource_changed::<GameMode>()))
            .add_systems(OnExit(GameState::Playing), (despawn_tiles_and_items, despawn_cards))
            .add_systems(
                OnEnter(GameState::Loaded),
                (prepare_for_new_level, spawn_cards, spawn_tiles).chain(),
            )
            .add_systems(
                Update,
                (|mut next_state: ResMut<NextState<GameState>>| {
                    next_state.set(GameState::Playing);
                })
                .run_if(in_state(AppState::Game))
                .run_if(in_state(GameState::Loaded)),
            )
            .add_systems(
                OnTransition {
                    from: GameState::Loaded,
                    to: GameState::Playing,
                },
                (shuffle_deck, schedule_transition::<NextTurnState>),
            )
            .add_systems(
                OnEnter(TurnState::Starting),
                (fill_hand_with_cards, restore_resources).run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (start_turn,)
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(TurnState::Starting)),
            )
            .add_systems(
                OnEnter(TurnState::Started),
                (update_playability, check_for_turn_over)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnEnter(TurnState::Animating), (animate_cards,))
            .add_systems(
                Update,
                (
                    transition::<GameState, NextGameState>,
                    transition::<TurnState, NextTurnState>,
                ),
            )
            .add_systems(
                OnEnter(TurnState::Ended),
                (
                    propagate_flames,
                    grow_flames,
                    |mut next_state: ResMut<NextState<TurnState>>| {
                        next_state.set(TurnState::Starting)
                    },
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    apply_change::<GamePosition>,
                    apply_change::<Energy>,
                    apply_change::<Water>,
                    apply_change::<Tile>,
                    add_item_sprite,
                    apply_item,
                    apply_card,
                    apply_card_actions,
                    check_for_level_end,
                    sync_deck,
                    sync_hand,
                    update_tiles,
                    put_flames_out,
                    play_clicked_card.run_if(in_state(TurnState::WaitingForInput)),
                    end_turn_clicked.run_if(in_state(TurnState::WaitingForInput)),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
