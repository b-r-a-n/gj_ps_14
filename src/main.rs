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
    game_state: Res<State<GameState>>,
    turn_state: Res<State<TurnState>>,
    card_infos: Query<(Entity, &CardInfo)>
) {
    if keyboard_input.get_just_released().last().is_none() {
        return;
    }
    match keyboard_input.get_just_released().last() {
        Some(KeyCode::Return) => {
            /*
            let card_info_id = card_infos.get_single().expect("Should be exactly 1 card info").0;
            decks.get_mut(entity)
                .expect("Failed to get the deck")
                .add(commands.spawn(BaseCardInfo(card_info_id)).id());
            */
            if game_state.get() == &GameState::Menu {
                commands.spawn(NextGameState);
            } else if turn_state.get() == &TurnState::WaitingForInput {
                commands.spawn(NextTurnState);
            }
        }
        Some(KeyCode::Space) => {
            let entity = player_state.get_single().expect("Should be exactly 1 player").0;
            commands.spawn(CardActionType::Draw(Draw {
                deck: entity,
                hand: entity,
            }));
        }

        Some(x) if x < &KeyCode::Key6 => {
            let (entity, _, _, hand) = player_state.get_single().expect("Should be exactly 1 player");
            let index = x.clone() as usize - KeyCode::Key1 as usize;
            if let Some(card) = hand.0[index] {
                commands.spawn(CardActionType::Play(Play {
                    card,
                    deck: entity,
                    hand: entity,
                }));
            }
        }
        /*
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
        */
        _ => {},
    }
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

#[derive(Component)]
pub struct Completed;

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

pub fn handle_resource_regeneration(
    mut commands: Commands,
    mut energy: Query<&mut Energy, With<Player>>,
    mut water: Query<&mut Water, With<Player>>,
    stats: Query<&Stats, With<Player>>,
    regenerate_resource: Query<(Entity, &EffectActionType), Without<Completed>>,
) {
    let stats = stats.get_single().expect("Should be exactly 1 stats");
    let mut energy_bonus = stats.energy_regeneration;
    let mut water_bonus = stats.water_regeneration;
    for (effect_instance_id, effect) in regenerate_resource.iter() {
        match effect {
            EffectActionType::RegenerateResource(regenerate_resource) => {
                energy_bonus += regenerate_resource.energy_bonus;
                water_bonus += regenerate_resource.water_bonus;
            },
            _ => {},
        }
        commands.entity(effect_instance_id).insert(Completed);
    }
    let mut energy = energy.get_single_mut().expect("Failed to get energy");
    let mut water = water.get_single_mut().expect("Failed to get water");
    energy.current = (energy.current + energy_bonus).min(energy.maxium);
    water.current = (water.current + water_bonus).min(water.maxium);
}

pub fn cleanup_completed_effects(
    mut commands: Commands,
    completed_effects: Query<Entity, (With<EffectActionType>, With<Completed>)>,
) {
    for entity in completed_effects.iter() {
        commands.entity(entity).despawn();
    }
}

fn print_state_change<T: States>(
    state: Res<State<T>>,
) {
    info!("{:?} changed to: {:?}", std::any::type_name::<T>(), state.get());
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraPlugin)
        .add_plugins(UIPlugins)
        .add_plugins(GamePlugin)
        .add_systems(Update, print_state_change::<TurnState>.run_if(state_changed::<TurnState>()))
        .add_systems(Update, print_state_change::<GameState>.run_if(state_changed::<GameState>()))
        //.add_systems(Update, (handle_resource_regeneration, cleanup_completed_effects))
        .add_systems(Update, handle_input)
        .add_systems(PostUpdate, update_position_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}
