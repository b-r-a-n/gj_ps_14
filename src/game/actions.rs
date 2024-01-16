use super::*;

#[derive(Clone, Debug)]
pub struct Draw {
    pub deck: Entity,
    pub hand: Entity,
}

#[derive(Clone, Debug)]
pub struct Recycle {
    pub card: Entity,
    pub hand: Entity,
}

#[derive(Clone, Debug)]
pub struct Discard {
    pub card: Entity,
    pub deck: Entity,
    pub hand: Entity,
}

#[derive(Clone, Debug)]
pub struct Play {
    pub card: Entity,
    pub deck: Entity,
    pub hand: Entity,
}

#[derive(Clone, Component, Debug)]
pub enum ActionType {
    Draw(Draw),
    Recycle(Recycle),
    Discard(Discard),
    Play(Play),
}

impl ActionType {
    pub fn to_will_event(&self, entity: Entity) -> CardEvent {
        match self {
            Self::Draw(action) => CardEvent::WillDraw(entity, action.clone()),
            Self::Recycle(action) => CardEvent::WillRecycle(entity, action.clone()),
            Self::Discard(action) => CardEvent::WillDiscard(entity, action.clone()),
            Self::Play(action) => CardEvent::WillPlay(entity, action.clone()),
        }
    }
    pub fn to_did_event(&self, entity: Entity) -> CardEvent {
        match self {
            Self::Draw(action) => CardEvent::DidDraw(entity, action.clone()),
            Self::Recycle(action) => CardEvent::DidRecycle(entity, action.clone()),
            Self::Discard(action) => CardEvent::DidDiscard(entity, action.clone()),
            Self::Play(action) => CardEvent::DidPlay(entity, action.clone()),
        }
    }
    pub fn from_event(event: &CardEvent) -> Self {
        match event {
            CardEvent::WillDraw(_, action) => Self::Draw(action.clone()),
            CardEvent::DidDraw(_, action) => Self::Draw(action.clone()),
            CardEvent::WillRecycle(_, action) => Self::Recycle(action.clone()),
            CardEvent::DidRecycle(_, action) => Self::Recycle(action.clone()),
            CardEvent::WillDiscard(_, action) => Self::Discard(action.clone()),
            CardEvent::DidDiscard(_, action) => Self::Discard(action.clone()),
            CardEvent::WillPlay(_, action) => Self::Play(action.clone()),
            CardEvent::DidPlay(_, action) => Self::Play(action.clone()),
        }
    }
}

#[derive(Debug, Event)]
pub enum CardEvent {
    WillDraw(Entity, Draw),
    DidDraw(Entity, Draw),
    WillRecycle(Entity, Recycle),
    DidRecycle(Entity, Recycle),
    WillDiscard(Entity, Discard),
    DidDiscard(Entity, Discard),
    WillPlay(Entity, Play),
    DidPlay(Entity, Play),
}

impl CardEvent {
    pub fn get_entity(&self) -> Entity {
        match self {
            CardEvent::WillDraw(entity, _) => *entity,
            CardEvent::DidDraw(entity, _) => *entity,
            CardEvent::WillRecycle(entity, _) => *entity,
            CardEvent::DidRecycle(entity, _) => *entity,
            CardEvent::WillDiscard(entity, _) => *entity,
            CardEvent::DidDiscard(entity, _) => *entity,
            CardEvent::WillPlay(entity, _) => *entity,
            CardEvent::DidPlay(entity, _) => *entity,
        }
    }
}

#[derive(Component)]
pub enum ActionState {
    Announced,
    Applied,
}

pub fn handle_card_events(
    mut commands: Commands,
    mut events: EventReader<CardEvent>,
) {

    let mut cleanup_ent = None;
    for event in events.read() {
        match event {
            CardEvent::DidDiscard(entity, _) => {
                cleanup_ent = Some(*entity);
            },
            CardEvent::DidDraw(entity, _) => {
                cleanup_ent = Some(*entity);
            },
            CardEvent::DidRecycle(entity, _) => {
                cleanup_ent = Some(*entity);
            },
            CardEvent::DidPlay(entity, _) => {
                cleanup_ent = Some(*entity);
            },
            _ => {},
        }
    }
    if let Some(entity) = cleanup_ent {
        // TODO: Maybe despawn?
        commands.entity(entity)
            .remove::<ActionState>()
            .remove::<ActionType>();
    }
}

pub fn announce_card_actions (
    mut commands: Commands,
    actions: Query<(Entity, &ActionType), Without<ActionState>>,
    mut events: EventWriter<CardEvent>
) {
    for (entity, action) in actions.iter() {
        events.send(action.to_will_event(entity));
        commands.entity(entity).insert(ActionState::Announced);
    }
}

pub fn apply_card_actions (
    mut commands: Commands,
    actions: Query<(Entity, &ActionType, &ActionState)>,
    mut decks: Query<&mut Deck>,
    mut hands: Query<&mut Hand>,
    mut events: EventWriter<CardEvent>,
) {
    for (entity, action, state) in actions.iter() {
        if match (action, state) {
            (ActionType::Draw(action), ActionState::Announced) => {
                let mut deck = decks.get_mut(action.deck)
                    .expect("Failed to get the deck");
                let mut hand = hands.get_mut(action.hand)
                    .expect("Failed to get the hand");
                let card = deck.draw()
                    .expect("Failed to draw a card");
                hand.add(card);
                true
            },
            (ActionType::Recycle(action), ActionState::Announced) => {
                let mut hand = hands.get_single_mut()
                    .expect("Failed to get the hand");
                hand.remove(action.card);
                let mut deck = decks.get_single_mut()
                    .expect("Failed to get the deck");
                deck.recycle(action.card);
                true
            },
            (ActionType::Discard(action), ActionState::Announced) => {
                let mut hand = hands.get_mut(action.hand)
                    .expect("Failed to get the hand");
                let mut deck = decks.get_mut(action.deck)
                    .expect("Failed to get the deck");
                hand.remove(action.card);
                deck.discard(action.card);
                true
            },
            (ActionType::Play(action), ActionState::Announced) => {
                let mut hand = hands.get_mut(action.hand)
                    .expect("Failed to get the hand");
                let mut deck = decks.get_mut(action.deck)
                    .expect("Failed to get the deck");
                hand.remove(action.card);
                deck.recycle(action.card);
                commands.spawn(WasPlayed(action.card));
                true
            },
            _ => false,
        } {
            commands.entity(entity).insert(ActionState::Applied);
            events.send(action.to_did_event(entity));
        }
    }
}

#[derive(Component)]
pub struct Change<T: Component + Clone> {
    pub entity: Entity,
    pub updated_value: T,
}

pub fn apply_change<T: Component + Clone>(
    mut commands: Commands,
    actions: Query<&Change<T>>, 
) {
    for action in actions.iter() {
        commands.entity(action.entity)
            .remove::<Change<T>>()
            .insert(action.updated_value.clone());
    }
}

#[derive(Component)]
pub struct WasPlayed(pub Entity);

pub fn apply_card (
    mut commands: Commands,
    player: Query<(Entity, &Energy, &Water), With<Player>>,
    played_cards: Query<(Entity, &WasPlayed)>,
    card_instances: Query<(Option<&NeedsEnergy>, Option<&NeedsWater>, Option<&NeedsMoveable>)>,
    game_positions: Query<&GamePosition>,
) {
    let (player_id, energy, water) = player.get_single().expect("There should only be one player");
    for (was_played_id, played_card) in played_cards.iter() {
        let card_instance_id = played_card.0;
        let (energy_need, water_need, moveable_tiles) = card_instances.get(card_instance_id)
            .expect("Failed to get card instance");
        println!("Card {:?} {:?} {:?} {:?} was played", card_instance_id, energy_need, water_need, moveable_tiles);
        if let Some(energy_need) = energy_need {
            commands.spawn(Change {
                entity: player_id,
                updated_value: Energy {
                    current: energy.current - energy_need.0,
                    ..energy.clone()
                }
            });
        }
        if let Some(water_need) = water_need {
            commands.spawn(Change {
                entity: player_id,
                updated_value: Water {
                    current: water.current - water_need.0,
                    ..water.clone()
                }
            });
        }
        if let Some(tiles) = moveable_tiles {
            if let Some(tile_id) = tiles.0.first() {
                let tile_pos = game_positions.get(*tile_id)
                    .expect("Failed to get tile position");
                commands.spawn(Change {
                    entity: player_id,
                    updated_value: GamePosition {
                        x: tile_pos.x,
                        y: tile_pos.y,
                        d: tile_pos.d.clone(),
                    }
                });
            }
        }
        commands.entity(was_played_id).remove::<WasPlayed>();
    }
}