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
    if hands.is_empty() {
        return;
    }
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
        if self.cards.is_empty() {
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

pub fn sync_deck(mut commands: Commands, decks: Query<(Entity, &Deck), Changed<Deck>>) {
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

#[derive(PartialEq, Eq, Debug)]
pub enum Rotation {
    None,
    Left,
    Right,
    Reverse,
}

pub struct MovementInfo {
    pub position: TileTarget,
    pub rotation: Rotation,
}

pub struct DamageInfo {
    pub damage_target: TileTarget,
    pub amount: u32,
}

impl DamageInfo {
    pub fn none() -> Self {
        Self {
            damage_target: TileTarget::FacingDist(0),
            amount: 0,
        }
    }
}

#[derive(Component)]
pub struct CardInfo {
    pub resource_cost: ResourceInfo,
    pub position_change: MovementInfo,
    pub water_damage: DamageInfo,
    pub texture_index: usize,
    pub name: String,
    pub description: String,
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
    pub fn get(&self, pos: &GamePosition) -> Option<Entity> {
        if pos.y < 0 || pos.x < 0 {
            return None;
        }
        if pos.y >= self.0.len() as i32 || pos.x >= self.0[0].len() as i32 {
            return None;
        }
        Some(self.0[pos.y as usize][pos.x as usize])
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
                if (x.abs() + y.abs()) > 1 {
                    continue;
                }
                if let Some(ent) = self.get(&neighbor) {
                    entities.push(ent);
                }
            }
        }
        entities
    }
}

#[derive(Clone)]
pub struct Offset {
    pub facing: i32,
    pub tangent: i32,
}
pub enum TileTarget {
    FacingDist(i32),
    FacingOffsets(Vec<Offset>),
}

impl TileTarget {
    pub fn get_positions(&self, base: &GamePosition) -> Vec<GamePosition> {
        let mut positions = Vec::new();
        match self {
            TileTarget::FacingDist(dist) => {
                positions.push(base.offset((*dist, 0)));
            }
            TileTarget::FacingOffsets(offsets) => {
                for offset in offsets.iter() {
                    positions.push(base.offset((offset.facing, offset.tangent)));
                }
            }
        }
        positions
    }
}

// ContentID is useful when trying to serialize/deserialize the game state
#[derive(Clone, Component, Debug, Eq, PartialEq, Hash)]
pub struct ContentID(pub usize);

pub fn despawn_card_infos(mut commands: Commands, card_infos: Query<Entity, With<ContentID>>) {
    for entity in card_infos.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Default, Resource)]
pub struct CardInfoMap(pub HashMap<ContentID, CardInfo>);

pub fn load_card_infos(mut map: ResMut<CardInfoMap>) {
    let mut card_infos = HashMap::new();
    // Forward
    card_infos.insert(
        ContentID(1),
        CardInfo {
            name: "Forward".to_string(),
            description: "Move forward 1 tile".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 0,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(1),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo::none(),
            texture_index: 0,
        },
    );
    // Backward
    card_infos.insert(
        ContentID(2),
        CardInfo {
            name: "Backward".to_string(),
            description: "Move backward 1 tile".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 0,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(-1),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo::none(),
            texture_index: 1,
        },
    );
    // Face Right
    card_infos.insert(
        ContentID(3),
        CardInfo {
            name: "Right".to_string(),
            description: "Rotate facing direction to the right".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 0,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::Right,
            },
            water_damage: DamageInfo::none(),
            texture_index: 2,
        },
    );
    // Face Left
    card_infos.insert(
        ContentID(4),
        CardInfo {
            name: "Left".to_string(),
            description: "Rotate facing direction to the left".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 0,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::Left,
            },
            water_damage: DamageInfo::none(),
            texture_index: 3,
        },
    );
    card_infos.insert(
        ContentID(5),
        CardInfo {
            name: "Squirt".to_string(),
            description: "Extinguish fire 1 tile away in facing direction".to_string(),
            resource_cost: ResourceInfo {
                energy: 0,
                water: 1,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingDist(1),
                amount: 1,
            },
            texture_index: 4,
        },
    );
    card_infos.insert(
        ContentID(6),
        CardInfo {
            name: "Splash".to_string(),
            description:
                "Extinguish up to 3 fires in a row that are 2 tiles away in facing direction"
                    .to_string(),
            resource_cost: ResourceInfo {
                energy: 0,
                water: 1,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: -1,
                        facing: 2,
                    },
                    Offset {
                        tangent: 0,
                        facing: 2,
                    },
                    Offset {
                        tangent: 1,
                        facing: 2,
                    },
                ]),
                amount: 1,
            },
            texture_index: 5,
        },
    );
    card_infos.insert(
        ContentID(7),
        CardInfo {
            name: "Sprinkle".to_string(),
            description: "Extinguish fire 1 tile away in each cardinal direction".to_string(),
            resource_cost: ResourceInfo {
                energy: 0,
                water: 2,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: -1,
                        facing: 0,
                    },
                    Offset {
                        tangent: 0,
                        facing: 1,
                    },
                    Offset {
                        tangent: 1,
                        facing: 0,
                    },
                    Offset {
                        tangent: 0,
                        facing: -1,
                    },
                ]),
                amount: 1,
            },
            texture_index: 6,
        },
    );
    card_infos.insert(
        ContentID(8),
        CardInfo {
            name: "Spray".to_string(),
            description: "Extinguish fire in a cone in the facing direction".to_string(),
            resource_cost: ResourceInfo {
                energy: 0,
                water: 3,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: 0,
                        facing: 1,
                    },
                    Offset {
                        tangent: -1,
                        facing: 2,
                    },
                    Offset {
                        tangent: 0,
                        facing: 2,
                    },
                    Offset {
                        tangent: 1,
                        facing: 2,
                    },
                    Offset {
                        tangent: -2,
                        facing: 3,
                    },
                    Offset {
                        tangent: 0,
                        facing: 3,
                    },
                    Offset {
                        tangent: 2,
                        facing: 3,
                    },
                ]),
                amount: 1,
            },
            texture_index: 7,
        },
    );
    card_infos.insert(
        ContentID(9),
        CardInfo {
            name: "Slosh".to_string(),
            description: "Move forward and extinguish 2 tiles adjacent to the destination".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 1,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(1),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: 1,
                        facing: 1,
                    },
                    Offset {
                        tangent: -1,
                        facing: 1,
                    },
                ]),
                amount: 1,
            },
            texture_index: 8,
        },
    );
    card_infos.insert(
        ContentID(10),
        CardInfo {
            name: "Water Jet".to_string(),
            description: "Water propels you forward and extinguishes some trailing tiles".to_string(),
            resource_cost: ResourceInfo {
                energy: 0,
                water: 2,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(1),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: 1,
                        facing: -1,
                    },
                    Offset {
                        tangent: 2,
                        facing: -2,
                    },
                    Offset {
                        tangent: -1,
                        facing: -1,
                    },
                    Offset {
                        tangent: -2,
                        facing: -2,
                    },
                ]),
                amount: 1,
            },
            texture_index: 9,
        },
    );
    card_infos.insert(
        ContentID(11),
        CardInfo {
            name: "Wave Turn Right".to_string(),
            description: "Turn right and extinguish a row of tiles in your original facing direction".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 1,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::Right,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: 1,
                        facing: 1,
                    },
                    Offset {
                        tangent: 0,
                        facing: 1,
                    },
                    Offset {
                        tangent: -1,
                        facing: 1,
                    },
                ]),
                amount: 1,
            },
            texture_index: 10,
        },
    );
    card_infos.insert(
        ContentID(12),
        CardInfo {
            name: "Wave Turn Left".to_string(),
            description: "Turn left and extinguish a row of tiles in your original facing direction".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 1,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::Left,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: 1,
                        facing: 1,
                    },
                    Offset {
                        tangent: 0,
                        facing: 1,
                    },
                    Offset {
                        tangent: -1,
                        facing: 1,
                    },
                ]),
                amount: 1,
            },
            texture_index: 11,
        },
    );
    card_infos.insert(
        ContentID(13),
        CardInfo {
            name: "Spin and Spray".to_string(),
            description: "Reverse facing direction and extinguish tiles at each corner".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 1,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::Reverse,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: 1,
                        facing: 1,
                    },
                    Offset {
                        tangent: 1,
                        facing: -1,
                    },
                    Offset {
                        tangent: -1,
                        facing: 1,
                    },
                    Offset {
                        tangent: -1,
                        facing: -1,
                    },
                ]),
                amount: 1,
            },
            texture_index: 12,
        },
    );
    card_infos.insert(
        ContentID(14),
        CardInfo {
            name: "Back Blast".to_string(),
            description: "Extinguish 3 tiles in facing direction and move backward one tile".to_string(),
            resource_cost: ResourceInfo {
                energy: 0,
                water: 2,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(-1),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: 0,
                        facing: 1,
                    },
                    Offset {
                        tangent: 0,
                        facing: 2,
                    },
                    Offset {
                        tangent: 0,
                        facing: 3,
                    },
                ]),
                amount: 1,
            },
            texture_index: 13,
        },
    );
    card_infos.insert(
        ContentID(15),
        CardInfo {
            name: "Expell".to_string(),
            description: "Extinguish 2 tiles from each diagonal".to_string(),
            resource_cost: ResourceInfo {
                energy: 0,
                water: 2,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: 1,
                        facing: 1,
                    },
                    Offset {
                        tangent: 2,
                        facing: 2,
                    },
                    Offset {
                        tangent: -1,
                        facing: 1,
                    },
                    Offset {
                        tangent: -2,
                        facing: 2,
                    },
                    Offset {
                        tangent: 1,
                        facing: -1,
                    },
                    Offset {
                        tangent: 2,
                        facing: -2,
                    },
                    Offset {
                        tangent: -1,
                        facing: -1,
                    },
                    Offset {
                        tangent: -2,
                        facing: -2,
                    },
                ]),
                amount: 1,
            },
            texture_index: 14,
        },
    );
    card_infos.insert(
        ContentID(16),
        CardInfo {
            name: "Cross Crash".to_string(),
            description: "Extinguish 2 tiles in each cardinal direction".to_string(),
            resource_cost: ResourceInfo {
                energy: 0,
                water: 3,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingDist(0),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo {
                damage_target: TileTarget::FacingOffsets(vec![
                    Offset {
                        tangent: 0,
                        facing: 1,
                    },
                    Offset {
                        tangent: 0,
                        facing: 2,
                    },
                    Offset {
                        tangent: -1,
                        facing: 0,
                    },
                    Offset {
                        tangent: -2,
                        facing: 0,
                    },
                    Offset {
                        tangent: 1,
                        facing: 0,
                    },
                    Offset {
                        tangent: 2,
                        facing: 0,
                    },
                    Offset {
                        tangent: 0,
                        facing: -1,
                    },
                    Offset {
                        tangent: 0,
                        facing: -2,
                    },
                ]),
                amount: 1,
            },
            texture_index: 15,
        },
    );
    card_infos.insert(
        ContentID(17),
        CardInfo {
            name: "Forward Right".to_string(),
            description: "Move forward and right".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 0,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingOffsets(vec![Offset{ tangent: 1, facing: 1}]),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo::none(),
            texture_index: 16,
        },
    );
    card_infos.insert(
        ContentID(18),
        CardInfo {
            name: "Forward Left".to_string(),
            description: "Move forward and left".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 0,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingOffsets(vec![Offset{ tangent: -1, facing: 1}]),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo::none(),
            texture_index: 17,
        },
    );
    card_infos.insert(
        ContentID(19),
        CardInfo {
            name: "Back Left".to_string(),
            description: "Move backward and left".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 0,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingOffsets(vec![Offset{ tangent: -1, facing: -1}]),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo::none(),
            texture_index: 18,
        },
    );
    card_infos.insert(
        ContentID(20),
        CardInfo {
            name: "Back Right".to_string(),
            description: "Move backward and right".to_string(),
            resource_cost: ResourceInfo {
                energy: 1,
                water: 0,
            },
            position_change: MovementInfo {
                position: TileTarget::FacingOffsets(vec![Offset{ tangent: 1, facing: -1}]),
                rotation: Rotation::None,
            },
            water_damage: DamageInfo::none(),
            texture_index: 19,
        },
    );
    *map = CardInfoMap(card_infos);
}
