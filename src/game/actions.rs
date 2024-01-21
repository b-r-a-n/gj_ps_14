use super::*;

#[derive(Clone, Component, Debug)]
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
                if let Some(card) = deck.draw() {
                    hand.add(card);
                }
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
    actions: Query<(Entity, &Change<T>)>, 
) {
    for (action_id, action) in actions.iter() {
        commands.entity(action_id)
            .remove::<Change<T>>();
        commands.entity(action.entity)
            .insert(action.updated_value.clone());
    }
}

#[derive(Component)]
pub struct WasPlayed(pub Entity);

pub fn apply_card (
    mut commands: Commands,
    mut turn_state: ResMut<NextState<TurnState>>,
    card_infos: Res<CardInfoMap>,
    player: Query<(Entity, &Energy, &Water), With<Player>>,
    played_cards: Query<(Entity, &WasPlayed)>,
    card_instances: Query<&ContentID>,
    game_positions: Query<&GamePosition>,
    grid: Query<&Grid>,
    tiles: Query<&Tile>,
) {
    let (player_id, energy, water) = player.get_single().expect("There should only be one player");
    for (was_played_id, played_card) in played_cards.iter() {
        let card_instance_id = played_card.0;
        let card_info_id = card_instances.get(card_instance_id)
            .expect("Failed to get card instance");
        let card_info = card_infos.0.get(&*card_info_id)
            .expect("Failed to get card info");
        commands.spawn(Change {
            entity: player_id,
            updated_value: Energy {
                current: energy.current - card_info.resource_cost.energy,
                ..energy.clone()
            }
        });
        commands.spawn(Change {
            entity: player_id,
            updated_value: Water {
                current: water.current - card_info.resource_cost.water,
                ..water.clone()
            }
        });
        match &card_info.position_change {
            MovementInfo { position: TileTarget::FacingDist(dist), rotation: rot} => {
                let base_pos = game_positions.get(player_id)
                    .expect("Failed to get player position");
                let new_pos = base_pos.rotated(&rot).offset((*dist, 0));
                commands.spawn(Change {
                    entity: player_id,
                    updated_value: new_pos
                });
            }
            _ => {},
        }
        match &card_info.water_damage {
            DamageInfo { damage_target: target, amount: _} => {
                let grid = grid.get_single()
                    .expect("Failed to get grid");
                let base_pos = game_positions.get(player_id)
                    .expect("Failed to get player position");
                let target_positions = target.get_positions(base_pos);
                for pos in target_positions.iter() {
                    if let Some(tile_id) = grid.get(pos) {
                        if let Tile::Fire(_) = tiles.get(tile_id).unwrap() {
                            commands.spawn(Change {
                                entity: tile_id,
                                updated_value: Tile::Empty
                            });
                        }
                    }
                }
            },
        }
        turn_state.set(TurnState::Animating);
        commands.entity(was_played_id).despawn_recursive();
    }
}

pub fn animate_cards(
    mut next_turn_state: ResMut<NextState<TurnState>>,
) {
    next_turn_state.set(TurnState::Started);
}