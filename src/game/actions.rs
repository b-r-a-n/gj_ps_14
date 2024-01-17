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
pub enum CardActionType {
    Draw(Draw),
    Recycle(Recycle),
    Discard(Discard),
    Play(Play),
}

pub fn apply_card_actions (
    mut commands: Commands,
    actions: Query<(Entity, &CardActionType)>,
    mut decks: Query<&mut Deck>,
    mut hands: Query<&mut Hand>,
) {
    for (entity, action) in actions.iter() {
        match action {
            CardActionType::Draw(action) => {
                let mut deck = decks.get_mut(action.deck)
                    .expect("Failed to get the deck");
                let mut hand = hands.get_mut(action.hand)
                    .expect("Failed to get the hand");
                let card = deck.draw()
                    .expect("Failed to draw a card");
                hand.add(card);
            },
            CardActionType::Recycle(action) => {
                let mut hand = hands.get_single_mut()
                    .expect("Failed to get the hand");
                hand.remove(action.card);
                let mut deck = decks.get_single_mut()
                    .expect("Failed to get the deck");
                deck.recycle(action.card);
            },
            CardActionType::Discard(action) => {
                let mut hand = hands.get_mut(action.hand)
                    .expect("Failed to get the hand");
                let mut deck = decks.get_mut(action.deck)
                    .expect("Failed to get the deck");
                hand.remove(action.card);
                deck.discard(action.card);
            },
            CardActionType::Play(action) => {
                let mut hand = hands.get_mut(action.hand)
                    .expect("Failed to get the hand");
                let mut deck = decks.get_mut(action.deck)
                    .expect("Failed to get the deck");
                hand.remove(action.card);
                deck.recycle(action.card);
                commands.spawn(WasPlayed(action.card));
            },
        }
        commands.entity(entity).despawn_recursive();
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
    mut turn_state: ResMut<NextState<TurnState>>,
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
        turn_state.set(TurnState::Animating);
        commands.entity(was_played_id).despawn_recursive();
    }
}

pub fn animate_cards(
    mut commands: Commands,
) {
    commands.spawn(NextTurnState);
}