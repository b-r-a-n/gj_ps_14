use super::*;

#[derive(Component)]
pub struct InHand;

#[derive(Component)]
pub struct InDeck;

#[derive(Component)]
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
) {
    for (_, hand) in hands.iter() {
        for card in hand.0.iter() {
            if let Some(card) = card {
                commands.entity(*card).insert(InHand);
            }
        }
    }
}

#[derive(Component)]
pub struct Deck {
    pub cards: Vec<Entity>,
    pub recycled: Vec<Entity>,
    pub discarded: Vec<Entity>,
}

impl Deck {
    pub fn add(&mut self, card: Entity) {
        self.cards.push(card);
    }
    pub fn draw(&mut self) -> Option<Entity> {
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

#[derive(Component)]
pub struct BlockedTile;

pub fn mark_playable(
    mut commands: Commands,
    cards: Query<(Entity, Option<&NeedsEnergy>, Option<&NeedsWater>, Option<&NeedsMoveable>, &BaseCardInfo), With<InHand>>,
    energy: Query<&Energy, With<Player>>,
    water: Query<&Water, With<Player>>,
    blocked_tiles: Query<&GamePosition, With<BlockedTile>>,
    position: Query<&GamePosition, With<Player>>,
) {
    let energy = energy.get_single().expect("Should be exactly 1 energy");
    let water = water.get_single().expect("Should be exactly 1 energy");
    let _ = position.get_single().expect("Should be exactly 1 position");
    for (card_instance_id, energy_cost, water_cost, moveable, _) in cards.iter() {
        if energy_cost.is_some_and(|c| c.0 > energy.current) 
        || water_cost.is_some_and(|c| c.0 > water.current) {
            continue;
        }
        // Need at least one moveable tile
        if let Some(moves) = moveable {
            if moves.0.is_empty() {
                continue;
            }
            let mut unblocked_tile = false;
            for tile_id in moves.0.iter() {
                if blocked_tiles.get(*tile_id).is_err() {
                    unblocked_tile = true;
                    break;
                }
            }
            if !unblocked_tile {
                continue;
            }
        }
        commands.entity(card_instance_id)
            .insert(Playable);
    }
}

#[derive(Component)]
pub struct Playable;

pub struct ResourceInfo {
    pub energy: u32,
    pub water: u32,
}

pub struct MovementInfo {
    pub destination_target: TileTarget,
    pub with_path: bool,
    pub pre_condition: bool,
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
    pub damage: DamageInfo,
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

pub fn realize_card_instances(
    mut commands: Commands,
    player_pos: Query<&GamePosition, With<Player>>,
    tile_grid: Query<&Grid>,
    card_instances: Query<(Entity, &BaseCardInfo), With<InHand>>,
    card_infos: Query<&CardInfo>,
) {
    // TODO: This is running every frame
    // Hopefully, introducing some states will allow more efficient scheduling
    for (entity, base_card_info) in card_instances.iter() {
        let card_info = card_infos.get(base_card_info.0).expect("Missing card info");
        let origin = player_pos.get_single().expect("Should be exactly 1 player position");
        if card_info.resource_cost.energy > 0 {
            commands.entity(entity).insert(NeedsEnergy(card_info.resource_cost.energy as i32));
        }
        if card_info.resource_cost.water > 0 {
            commands.entity(entity).insert(NeedsWater(card_info.resource_cost.energy as i32));
        }
        match card_info.position_change.destination_target {
            TileTarget::None => {},
            TileTarget::Offset(dist) => { 
                let tile_id = tile_grid.single().get(&origin.offset(dist));
                commands.entity(entity)
                    .insert(NeedsMoveable(vec![tile_id])); 
            },
            TileTarget::Adjacent(radius) => {
                let adjacent_tile_positions = origin.adjacent(radius as i32);
                let tile_ids: Vec<Entity> = adjacent_tile_positions.iter()
                    .map(|pos| tile_grid.single().get(pos))
                    .collect();
                commands.entity(entity)
                    .insert(NeedsMoveable(tile_ids));
            },
            _ => {},
        }
    }
}


pub enum TileTarget {
    Offset(i32),
    Adjacent(u32),
    SelectedOffset(i32),
    SelectedAdjacent(u32),
    None
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
    // Doze
    commands.spawn((
        CardInfo {
            resource_cost: ResourceInfo {
                energy: 1,
                water: 0,
            },
            position_change: MovementInfo {
                destination_target: TileTarget::Offset(1),
                with_path: true,
                pre_condition: true,
            },
            damage: DamageInfo {
                damage_target: TileTarget::Offset(1),
                amount: 1,
                pre_condition: false,
            },
        },
    ));

}