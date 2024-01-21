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

    pub fn reset(&mut self) {
        self.0 = [None; 5];
    }
}

pub fn sync_hand(
    mut commands: Commands,
    hands: Query<(Entity, &Hand), Changed<Hand>>,
    cards: Query<Entity, With<ContentID>>,
) {
    if hands.is_empty() { return ; }
    let (_, hand) = hands.get_single().expect("Should be exactly 1 hand");
    for card_instance_id in cards.iter() {
        if hand.0.contains(&Some(card_instance_id)) {
            commands.entity(card_instance_id).insert(InHand);
        } else {
            commands.entity(card_instance_id).remove::<InHand>();
        }
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
    pub fn reset(&mut self) {
        self.cards.clear();
        self.recycled.clear();
        self.discarded.clear();
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

#[derive(Component, Debug)]
pub struct Moveable;

#[derive(Component)]
pub struct Damageable(pub Vec<Entity>);

#[derive(Component)]
pub struct Grid(pub Vec<Vec<Entity>>);

impl Grid {
    pub fn get(&self, pos: &GamePosition) -> Entity {
        self.0[pos.y as usize][pos.x as usize]
    }

    pub fn neighbors(&self, pos: &GamePosition) -> Vec<Entity> {
        let mut entities = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                if x == 0 && y == 0 {
                    continue;
                }
                let neighbor = GamePosition {
                    x: pos.x + x,
                    y: pos.y + y,
                    ..pos.clone()
                };
                if neighbor.x < 0 || neighbor.y < 0 {
                    continue;
                }
                if neighbor.x as usize >= self.0.len() || neighbor.y as usize >= self.0[0].len() {
                    continue;
                }
                entities.push(self.get(&neighbor));
            }
        }
        entities
    }
}

pub enum TileTarget {
    Offset(i32),
}

// ContentID is useful when trying to serialize/deserialize the game state
#[derive(Clone, Component, Eq, PartialEq, Hash)]
pub struct ContentID(pub usize);

pub fn despawn_card_infos(
    mut commands: Commands,
    card_infos: Query<Entity, With<ContentID>>,
) {
    for entity in card_infos.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Default, Resource)]
pub struct CardInfoMap(pub HashMap<ContentID, CardInfo>);

pub fn load_card_infos(
    mut map: ResMut<CardInfoMap>,
) {
    // TODO: This info will eventually come from some sort of asset stored on disk
    let mut card_infos = HashMap::new();
    card_infos.insert(ContentID(1),
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
        }
    );
    card_infos.insert(ContentID(2),
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
        }
    );
    card_infos.insert(ContentID(3),
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
        }
    );
    card_infos.insert(ContentID(4),
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
        }
    );
    *map = CardInfoMap(card_infos);
}