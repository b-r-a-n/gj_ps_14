use self::ui::energy;

use super::*;

#[derive(Component, Debug)]
pub struct InHand;

#[derive(Component)]
pub struct InDeck;

#[derive(Component, Default)]
pub struct Hand(pub [Option<Entity>; 5]);

impl Hand {
    pub fn add(&mut self, card: Entity) {
        for slot in self.0.iter_mut() {
            if slot.is_none() {
                *slot = Some(card);
                return;
            }
        }
    }
    pub fn remove(&mut self, card: Entity) {
        for slot in self.0.iter_mut() {
            if *slot == Some(card) {
                *slot = None;
                return;
            }
        }
    }
    pub fn empty_slots(&self) -> usize {
        self.0.iter().filter(|&slot| slot.is_none()).count()
    }
}

pub fn sync_hand(
    mut commands: Commands,
    hands: Query<(Entity, &Hand), Changed<Hand>>,
    mut previous_hand: Local<Hand>,
) {
    for (_, hand) in hands.iter() {
        for card in previous_hand.0.iter() {
            if let Some(card) = card {
                commands.entity(*card).remove::<InHand>();
            }
        }
        for card in hand.0.iter() {
            if let Some(card) = card {
                commands.entity(*card).insert(InHand);
            }
        }
        previous_hand.0 = hand.0;
    }
}

#[derive(Component)]
pub struct Deck {
    pub cards: Vec<Entity>,
    pub recycled: Vec<Entity>,
    pub discarded: Vec<Entity>,
}

use rand::seq::SliceRandom;
use rand::thread_rng;

impl Deck {
    pub fn add(&mut self, card: Entity) {
        self.cards.push(card);
    }
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }
    pub fn draw(&mut self) -> Option<Entity> {
        if self.cards.is_empty(){
            self.cards = self.recycled.drain(..).collect();
            self.shuffle();
        }
        self.cards.pop()
    }
    pub fn recycle(&mut self, card: Entity) {
        self.cards.retain(|&c| c != card);
        self.recycled.push(card);
    }
    pub fn discard(&mut self, card: Entity) {
        self.cards.retain(|&c| c != card);
        self.recycled.retain(|&c| c != card);
        self.discarded.push(card);
    }
}

pub fn sync_deck(
    mut commands: Commands,
    decks: Query<(Entity, &Deck), Changed<Deck>>,
) {
    for (_, deck) in decks.iter() {
        for card in deck.cards.iter() {
            commands.entity(*card).insert(InDeck);
        }
    }
}

pub fn fill_hand_with_cards(
    mut commands: Commands,
    deck: Query<(Entity, &Deck), With<Player>>,
    hand: Query<(Entity, &Hand), With<Player>>,
) {
    let (hand_id, hand) = hand.get_single().expect("Should be exactly 1 hand");
    let (deck_id, _) = deck.get_single().expect("Should be exactly 1 deck");
    (0..hand.empty_slots()).for_each(|_| {
        commands.spawn(CardActionType::Draw(Draw {
            deck: deck_id,
            hand: hand_id,
        }));
    });
}

#[derive(Component)]
pub struct BlockedTile;

pub fn log_playability_changes(
    added_playable: Query<Entity, Added<Playable>>,
    mut removed_playable: RemovedComponents<Playable>,
) {
    for entity in added_playable.iter() {
        info!("Card {:?} is now playable", entity);
    }
    for entity in removed_playable.read() {
        info!("Card {:?} is no longer playable", entity);
    }
}

pub fn update_playable(
    mut commands: Commands,
    card_instances: Query<(Entity, Option<&InHand>), With<BaseCardInfo>>,
    needs_something: Query<Entity, Or<(With<NeedsEnergy>, With<NeedsWater>, With<NeedsMoveable>)>>,
) {
    for (card_instance_id, in_hand) in card_instances.iter() {
        if in_hand.is_none() {
            commands.entity(card_instance_id).remove::<Playable>();
            continue;
        }
        if needs_something.contains(card_instance_id) {
            commands.entity(card_instance_id).remove::<Playable>();
            continue;
        }
        commands.entity(card_instance_id).insert(Playable);
    }
}

#[derive(Component)]
pub struct Playable;

pub struct ResourceInfo {
    pub energy: i32,
    pub water: i32,
}

#[derive(Debug)]
pub enum Rotation {
    None,
    Left,
    Right,
}

pub struct MovementInfo {
    pub position: TileTarget,
    pub rotation: Rotation,
}

pub struct DamageInfo {
    pub damage_target: TileTarget,
    pub amount: u32,
    pub pre_condition: bool,
}

#[derive(Component)]
pub struct CardInfo {
    pub resource_cost: ResourceInfo,
    pub position_change: MovementInfo,
    pub texture_index: usize,
}

#[derive(Component)]
pub struct AssetInfo {
    pub sprite_sheet_name: String,
    pub sprite_index: usize,
}

#[derive(Component)]
pub struct PreCondition(pub Entity);

#[derive(Component)]
pub struct HasEnergy(pub u32);

#[derive(Component)]
pub struct HasWater(pub u32);

#[derive(Component)]
pub struct BaseCardInfo(pub Entity);

#[derive(Component, Debug)]
pub struct Moveable;

#[derive(Component)]
pub struct Damageable(pub Vec<Entity>);

#[derive(Component)]
pub struct Grid(pub Vec<Vec<Entity>>);

impl Grid {
    pub fn get(&self, pos: &GamePosition) -> Entity {
        self.0[pos.x as usize][pos.y as usize]
    }
}

#[derive(Component, Debug)]
pub struct NeedsEnergy(pub i32);

#[derive(Component, Debug)]
pub struct NeedsWater(pub i32);

#[derive(Component, Debug)]
pub struct NeedsMoveable(pub Vec<Entity>);

pub fn update_movement_needs(
    commands: &mut Commands,
    card_instance_id: Entity,
    base_card_infos: &Query<&BaseCardInfo>,
    card_infos: &Query<&CardInfo>,
    player_pos: &GamePosition,
    tile_grid: &Grid,
    blocked_tiles: &Query<&BlockedTile>,
) {
    let base_card_info = base_card_infos.get(card_instance_id).expect("Missing base card info");
    let card_info = card_infos.get(base_card_info.0).expect("Missing card info");
    match &card_info.position_change {
        MovementInfo { position: TileTarget::Offset(dist), rotation: rot} => {
            let target_pos = player_pos.rotated(rot).offset(*dist);
            let tile_id = tile_grid.get(&target_pos);
            if blocked_tiles.get(tile_id).is_ok() {
                commands.entity(card_instance_id)
                    .insert(NeedsMoveable(vec![tile_id]));
            } else {
                commands.entity(card_instance_id)
                    .remove::<NeedsMoveable>();
            }
        }
        _ => {}
    }
}

pub fn update_resource_needs(
    commands: &mut Commands,
    card_instance_id: Entity,
    base_card_infos: &Query<&BaseCardInfo>,
    card_infos: &Query<&CardInfo>,
    player_energy: &Energy,
    player_water: &Water,
) {
    let base_card_info = base_card_infos.get(card_instance_id).expect("Missing base card info");
    let card_info = card_infos.get(base_card_info.0).expect("Missing card info");
    if card_info.resource_cost.energy > player_energy.current {
        commands.entity(card_instance_id)
            .insert(NeedsEnergy(card_info.resource_cost.energy - player_energy.current));
    } else {
        commands.entity(card_instance_id)
            .remove::<NeedsEnergy>();
    }
    if card_info.resource_cost.water > player_water.current {
        commands.entity(card_instance_id)
            .insert(NeedsWater(card_info.resource_cost.water - player_water.current));
    } else {
        commands.entity(card_instance_id)
            .remove::<NeedsWater>();
    }
}

pub fn handle_resource_change(
    mut commands: Commands,
    card_instances_to_check: Query<Entity, With<InHand>>,
    base_card_infos: Query<&BaseCardInfo>,
    card_infos: Query<&CardInfo>,
    player_energy_changes: Query<&Energy, (With<Player>, Changed<Energy>)>,
    player_water_changes: Query<&Water, (With<Player>, Changed<Water>)>,
) {
    if player_energy_changes.is_empty() && player_water_changes.is_empty() {
        return;
    }
    for card_instance_id in card_instances_to_check.iter() {
        update_resource_needs(
            &mut commands,
            card_instance_id,
            &base_card_infos,
            &card_infos,
            player_energy_changes.get_single().expect("Should be exactly 1 player energy"),
            player_water_changes.get_single().expect("Should be exactly 1 player water"));
    }
}

pub fn handle_card_added_to_hand(
    mut commands: Commands,
    card_instances_to_check: Query<Entity, Added<InHand>>,
    base_card_infos: Query<&BaseCardInfo>,
    card_infos: Query<&CardInfo>,
    player_energy: Query<&Energy, With<Player>>,
    player_water: Query<&Water, With<Player>>,
    player_pos: Query<&GamePosition, With<Player>>,
    tile_grid: Query<&Grid>,
    blocked_tiles: Query<&BlockedTile>,
) {

    for card_instance_id in card_instances_to_check.iter() {
        update_resource_needs(
            &mut commands,
            card_instance_id,
            &base_card_infos,
            &card_infos,
            player_energy.get_single().expect("Should be exactly 1 player energy"),
            player_water.get_single().expect("Should be exactly 1 player water"));
        update_movement_needs(
            &mut commands,
            card_instance_id,
            &base_card_infos,
            &card_infos,
            player_pos.get_single().expect("Should be exactly 1 player position"),
            tile_grid.get_single().expect("Should be exactly 1 tile grid"),
            &blocked_tiles,
        );
    }
}

pub fn handle_position_change(
    mut commands: Commands,
    card_instances_to_check: Query<Entity, With<InHand>>,
    base_card_infos: Query<&BaseCardInfo>,
    card_infos: Query<&CardInfo>,
    player_position_changes: Query<&GamePosition, (With<Player>, Changed<GamePosition>)>,
    tile_grid: Query<&Grid>,
    blocked_tiles: Query<&BlockedTile>,
) {
    if player_position_changes.is_empty() {
        return;
    }
    for card_instance_id in card_instances_to_check.iter() {
        update_movement_needs(
            &mut commands,
            card_instance_id,
            &base_card_infos,
            &card_infos,
            player_position_changes.get_single().expect("Should be exactly 1 player position"),
            tile_grid.get_single().expect("Should be exactly 1 tile grid"),
            &blocked_tiles,
        );
    }
}

pub enum TileTarget {
    Offset(i32),
}

pub fn make_grid(
    mut commands: Commands,
) {
    let mut grid = Vec::new();
    for x in 0..100 {
        let mut row = Vec::new();
        for y in 0..100 {
            row.push(
                commands.spawn((GamePosition { x, y, ..default() }, Moveable)).id()
            );
        }
        grid.push(row);
    }
    commands.spawn(Grid(grid));
}

pub fn load_card_infos(
    mut commands: Commands,
) {
    commands.spawn_batch(vec![
        (
            CardInfo {
                resource_cost: ResourceInfo {
                    energy: 1,
                    water: 0,
                },
                position_change: MovementInfo {
                    position: TileTarget::Offset(1),
                    rotation: Rotation::None,
                },
                texture_index: 0,
            },
        ),
        (
            CardInfo {
                resource_cost: ResourceInfo {
                    energy: 1,
                    water: 0,
                },
                position_change: MovementInfo {
                    position: TileTarget::Offset(-1),
                    rotation: Rotation::None,
                },
                texture_index: 1,
            },
        ),
        (
            CardInfo {
                resource_cost: ResourceInfo {
                    energy: 1,
                    water: 0,
                },
                position_change: MovementInfo {
                    position: TileTarget::Offset(0),
                    rotation: Rotation::Right,
                },
                texture_index: 2,
            },
        ),
        (
            CardInfo {
                resource_cost: ResourceInfo {
                    energy: 1,
                    water: 0,
                },
                position_change: MovementInfo {
                    position: TileTarget::Offset(0),
                    rotation: Rotation::Left,
                },
                texture_index: 3,
            },
        ),
    ]);

}