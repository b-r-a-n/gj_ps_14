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
    turn_state: Res<State<TurnState>>,
    mut next_turn_state: ResMut<NextState<TurnState>>,
    player_state: Query<(Entity, &GamePosition, &Energy, &Hand), With<Player>>,
    game_state: Res<State<GameState>>,
    playables: Query<&Playable>
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
                next_turn_state.set(TurnState::Ended);
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
            if turn_state.get() != &TurnState::WaitingForInput {
                return;
            }
            let (entity, _, _, hand) = player_state.get_single().expect("Should be exactly 1 player");
            let index = x.clone() as usize - KeyCode::Key1 as usize;
            if let Some(card) = hand.0[index] {
                if playables.get(card).is_err() {
                    return;
                }
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


fn print_state_change<T: States>(
    state: Res<State<T>>,
) {
    info!("{:?} changed to: {:?}", std::any::type_name::<T>(), state.get());
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(CameraPlugin)
        .add_plugins(UIPlugins)
        .add_plugins(GamePlugin)
        .add_systems(Update, print_state_change::<TurnState>.run_if(state_changed::<TurnState>()))
        .add_systems(Update, print_state_change::<GameState>.run_if(state_changed::<GameState>()))
        .add_systems(Update, handle_input)
        .add_systems(PostUpdate, update_position_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}
