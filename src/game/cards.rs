use super::*;

#[derive(Component)]
pub struct CardState {
    pub hand: Option<Entity>,
    pub deck: Option<Entity>,
}

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
}

pub fn sync_hand(
    mut commands: Commands,
    hands: Query<(Entity, &Hand), Changed<Hand>>,
    states: Query<&CardState>,
) {
    for (entity, hand) in hands.iter() {
        for card in hand.0.iter() {
            if let Some(card) = card {
                let state = states.get(*card)
                    .expect("Card in hand without state");
                commands.entity(*card)
                    .insert(CardState {
                        hand: Some(entity),
                        ..*state
                    });
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
    states: Query<&CardState>,
) {
    for (entity, deck) in decks.iter() {
        for card in deck.cards.iter() {
            let state = states.get(*card).unwrap_or(&CardState {
                hand: None,
                deck: None,
            });
            commands.entity(*card)
                .insert(CardState {
                    deck: Some(entity),
                    ..*state
                });
        }
    }
}

pub fn mark_playable(
    mut commands: Commands,
    hands: Query<&Hand>,
    cards: Query<&BaseCardInfo>,
    card_infos: Query<&CardInfo>,
    energy: Query<&Energy, With<Player>>,
    position: Query<&GamePosition, With<Player>>,
) {
    if hands.is_empty() { return; }
    let hand = hands.get_single().expect("Should be exactly 1 hand");
    let energy = energy.get_single().expect("Should be exactly 1 energy");
    let _ = position.get_single().expect("Should be exactly 1 position");
    for card in hand.0.iter() {
        if let Some(card_id) = card {
            let base_card_info = cards.get(*card_id).expect("Card without info");
            let card_info = card_infos.get(base_card_info.0).expect("Card without info");
            if card_info.resource_cost.energy as i32 > energy.current {
                continue;
            }
            commands.entity(*card_id)
                .insert(Playable);
        }
    }
}

#[derive(Component)]
pub struct Playable;

pub struct ResourceInfo {
    pub energy: u32,
    pub water: u32,
}

pub enum MovementType {
    Offset(i32),
    //Targeted(u32)
}

pub struct MovementInfo {
    pub movement_type: MovementType,
    pub with_path: bool,
    pub pre_condition: bool,
}

pub enum TargetType {
    Offset(i32),
    //Radius(u32),
    //Area(i32, u32)
}

pub struct DamageInfo {
    pub target_type: TargetType,
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

#[derive(Component)]
pub struct Moveable(pub Vec<Entity>);

#[derive(Component)]
pub struct Damageable(pub Vec<Entity>);

#[derive(Component)]
pub struct Grid(pub Vec<Vec<Entity>>);

impl Grid {
    pub fn get(&self, pos: GamePosition) -> Entity {
        self.0[pos.x as usize][pos.y as usize]
    }
}

pub fn expand_card_info (
    mut commands: Commands,
    hands: Query<&Hand, With<Player>>,
    position: Query<&GamePosition, With<Player>>,
    base_card_info: Query<&BaseCardInfo>,
    card_infos: Query<&CardInfo>,
    grid: Query<&Grid>,
) {
    if hands.is_empty() { return; }
    let hand = hands.get_single().expect("Should be exactly 1 hand");
    for card_instance in hand.0.iter() {
        if let Some(card_instance_id) = card_instance {
            let base_card_info = base_card_info.get(*card_instance_id).expect("Card without base info");
                if let Ok(card_info) = card_infos.get(base_card_info.0) {
                    let mut builder = commands.spawn(PreCondition(*card_instance_id));
                    if card_info.resource_cost.energy > 0 {
                        builder.insert(HasEnergy(card_info.resource_cost.energy));
                    }
                    if card_info.resource_cost.water > 0 {
                        builder.insert(HasWater(card_info.resource_cost.water));
                    }
                    match card_info.position_change {
                        MovementInfo { movement_type: MovementType::Offset(dist), with_path: full_path, pre_condition: true } => {
                            if full_path {
                                let player_pos = position.get_single().expect("Should be exactly 1 position");
                                let mut tile_ents = Vec::new();
                                for i in 1..=dist {
                                    let tile_pos = player_pos.offset(i);
                                    let tile_ent = grid.single().get(tile_pos);
                                    tile_ents.push(tile_ent);
                                }
                                builder.insert(Moveable(tile_ents));
                            } else {
                                let player_pos = position.get_single().expect("Should be exactly 1 position");
                                let tile_pos = player_pos.offset(dist);
                                let tile_ent = grid.single().get(tile_pos);
                                builder.insert(Moveable(vec![tile_ent]));
                            }
                        },
                        _ => {},
                    }
                    match card_info.damage {
                        DamageInfo { target_type: TargetType::Offset(dist), amount, pre_condition: true } => {
                            let player_pos = position.get_single().expect("Should be exactly 1 position");
                            let tile_pos = player_pos.offset(dist);
                            let tile_ent = grid.single().get(tile_pos);
                            builder.insert(Damageable(vec![tile_ent]));
                        },
                        _ => {},
                    }
                }
        }
    }
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
                movement_type: MovementType::Offset(1),
                with_path: true,
                pre_condition: true,
            },
            damage: DamageInfo {
                target_type: TargetType::Offset(1),
                amount: 1,
                pre_condition: false,
            },
        },
    ));

}