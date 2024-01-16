use bevy::prelude::*;
use game::*;
use ui::{UIPlugins, energy::*, hand::*};
use camera::*;

mod camera;
mod game;
mod ui;

fn update_position_transforms(
    mut query: Query<(&GamePosition, &mut Transform)>
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x as f32 * 64.0;
        transform.translation.y = position.y as f32 * 64.0;
        transform.rotation = match position.d {
            GameDirection::Up => Quat::from_rotation_z(0.0),
            GameDirection::Down => Quat::from_rotation_z(std::f32::consts::PI),
            GameDirection::Left => Quat::from_rotation_z(std::f32::consts::PI * 0.5),
            GameDirection::Right => Quat::from_rotation_z(std::f32::consts::PI * 1.5),
        };
    }
}

fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    mut decks: Query<&mut Deck>,
    player_state: Query<(Entity, &GamePosition, &Energy, &Hand), With<Player>>,
    card_infos: Query<(Entity, &CardInfo)>
) {
    if keyboard_input.get_just_released().last().is_none() {
        return;
    }
    let (entity, position, energy, hand) = player_state.get_single()
        .expect("Found more than one player position");

    let energy_change = Change {
        entity,
        updated_value: Energy {
            current: energy.current - 1,
            ..energy.clone()
        },
    };

    match keyboard_input.get_just_released().last() {
        Some(KeyCode::Return) => {
            let card_info_id = card_infos.get_single().expect("Should be exactly 1 card info").0;
            decks.get_mut(entity)
                .expect("Failed to get the deck")
                .add(commands.spawn(BaseCardInfo(card_info_id)).id());
        }
        Some(KeyCode::Space) => {
            commands.spawn(CardActionType::Draw(Draw {
                deck: entity,
                hand: entity,
            }));
        }

        Some(x) if x < &KeyCode::Key6 => {
            let index = x.clone() as usize - KeyCode::Key1 as usize;
            if let Some(card) = hand.0[index] {
                commands.spawn(CardActionType::Play(Play {
                    card,
                    deck: entity,
                    hand: entity,
                }));
            }
        }
        Some(KeyCode::Up) => {
            commands.spawn((
                Change {
                    entity,
                    updated_value: GamePosition {
                        y: position.y + 1,
                        d: GameDirection::Up,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        Some(KeyCode::Down) => {
            commands.spawn((
                Change {
                    entity,
                    updated_value: GamePosition {
                        y: position.y - 1,
                        d: GameDirection::Down,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        Some(KeyCode::Left) => {
            commands.spawn((
                Change {
                    entity,
                    updated_value: GamePosition {
                        x: position.x - 1,
                        d: GameDirection::Left,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        Some(KeyCode::Right) => {
            commands.spawn((
                Change {
                    entity,
                    updated_value: GamePosition {
                        x: position.x + 1,
                        d: GameDirection::Right,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        _ => {},
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum TurnState {
    #[default]
    Starting,
    WaitingForInput,
    Ending,
}

#[derive(Component)]
pub struct Stats {
    pub energy_regeneration: i32,
    pub water_regeneration: i32,
}

#[derive(Clone, Debug)]
pub struct RegenerateResource {
    pub energy_bonus: i32,
    pub water_bonus: i32,
}

#[derive(Component)]
pub enum EffectActionType {
    RegenerateResource(RegenerateResource)
}

pub fn fill_hand_with_cards(
    mut commands: Commands,
    deck: Query<(Entity, &Deck), With<Player>>,
    hand: Query<(Entity, &Hand), With<Player>>,
) {
    let (hand_id, hand) = hand.get_single().expect("Should be exactly 1 hand");
    let (deck_id, _) = deck.get_single().expect("Should be exactly 1 deck");

    println!("Filling hand");

    (0..hand.empty_slots()).for_each(|_| {
        commands.spawn(CardActionType::Draw(Draw {
            deck: deck_id,
            hand: hand_id,
        }));
    });
}

pub fn restore_resources(
    mut commands: Commands,
) {
    commands.spawn(EffectActionType::RegenerateResource(RegenerateResource {
        energy_bonus: 0,
        water_bonus: 0,
    }));
}

#[derive(Component)]
pub struct NextTurnState;

pub fn next_turn_state(
    mut commands: Commands,
) {
    commands.spawn(NextTurnState);
}

pub fn transition_turn_state(
    mut commands: Commands,
    transition_tag: Query<Entity, With<NextTurnState>>,
    current: Res<State<TurnState>>,
    mut next: ResMut<NextState<TurnState>>,
) {
    if transition_tag.is_empty() {
        return;
    }

    for entity in transition_tag.iter() {
        commands.entity(entity).despawn();
    }
    
    match current.get() {
        TurnState::Starting => {
            next.set(TurnState::WaitingForInput);
        },
        TurnState::WaitingForInput => {
            next.set(TurnState::Ending);
        },
        TurnState::Ending => {
            next.set(TurnState::Starting);
        },
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraPlugin)
        .add_plugins(UIPlugins)
        .add_plugins(GamePlugin)
        .add_state::<TurnState>()
        .add_systems(OnEnter(GameState::Playing), |mut turn_state: ResMut<NextState<TurnState>>| turn_state.set(TurnState::Starting))
        .add_systems(OnEnter(TurnState::Starting), (fill_hand_with_cards, restore_resources, next_turn_state))
        .add_systems(Update, (handle_input, transition_turn_state))
        .add_systems(PostUpdate, update_position_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}
