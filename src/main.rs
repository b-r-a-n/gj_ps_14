use bevy::prelude::*;
use camera::*;
use game::*;
use ui::*;

mod camera;
mod game;
mod ui;

fn update_position_transforms(
    mut query: Query<(&GamePosition, &mut Transform, Option<&Animating>)>,
    time: Res<Time>,
    animations: Query<&Animation>,
) {
    for (position, mut transform, animating) in query.iter_mut() {
        if animating.is_none() {
            transform.translation.x = position.x as f32 * 64.0;
            transform.translation.y = position.y as f32 * 64.0;
            transform.rotation = position.d.get_quat();
        } else {
            let animation = animations
                .get(animating.unwrap().0)
                .expect("Animation should exist");
            let remaining_distance = match &animation.animation_type {
                AnimationType::Move(_, vec) => {
                    let distance = Vec2::new(vec.x * 64.0, vec.y * 64.0);
                    distance - transform.translation.truncate()
                }
                AnimationType::Rotate(_, rotation) => {
                    transform.rotation = transform
                        .rotation
                        .lerp(*rotation, time.delta_seconds() / animation.duration);
                    Vec2::ZERO // TODO: Implement rotation animation
                }
                _ => Vec2::ZERO,
            };
            let velocity = remaining_distance / animation.duration;
            transform.translation.x += velocity.x * time.delta_seconds();
            transform.translation.y += velocity.y * time.delta_seconds();
        }
    }
}

fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    turn_state: Res<State<TurnState>>,
    mut next_turn_state: ResMut<NextState<TurnState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut camera_transform: Query<&mut Transform, With<MainCamera>>,
    player_state: Query<(Entity, &GamePosition, &Energy, &Hand), With<Player>>,
    statuses: Query<&CardStatus>,
) {
    if keyboard_input.get_just_released().last().is_none() {
        return;
    }
    match keyboard_input.get_just_released().last() {
        Some(KeyCode::Return) => {
            if turn_state.get() != &TurnState::WaitingForInput {
                return;
            }
            next_turn_state.set(TurnState::Ended);
        }
        Some(KeyCode::Escape) => {
            if turn_state.get() != &TurnState::WaitingForInput {
                return;
            }
            next_turn_state.set(TurnState::None);
            next_app_state.set(AppState::MainMenu);
            next_game_state.set(GameState::None);
        }
        Some(x) if x < &KeyCode::Key6 => {
            if turn_state.get() != &TurnState::WaitingForInput {
                return;
            }
            let (entity, _, _, hand) = player_state
                .get_single()
                .expect("Should be exactly 1 player");
            let index = x.clone() as usize - KeyCode::Key1 as usize;
            if let Some(card) = hand.0[index] {
                if !statuses
                    .get(card)
                    .expect("Card in hand should have status")
                    .is_playable()
                {
                    return;
                }
                commands.spawn(CardActionType::Play(Play {
                    card,
                    deck: entity,
                    hand: entity,
                }));
            }
        }

        Some(dir)
            if vec![KeyCode::Up, KeyCode::Right, KeyCode::Down, KeyCode::Left].contains(dir) =>
        {
            // Move the camera with the arrow keys
            if dir == &KeyCode::Up {
                camera_transform.single_mut().translation.y += 64.0;
            } else if dir == &KeyCode::Down {
                camera_transform.single_mut().translation.y -= 64.0;
            } else if dir == &KeyCode::Left {
                camera_transform.single_mut().translation.x -= 64.0;
            } else if dir == &KeyCode::Right {
                camera_transform.single_mut().translation.x += 64.0;
            }
        }
        _ => {}
    }
}

fn print_state_change<T: States>(state: Res<State<T>>) {
    info!(
        "{:?} changed to: {:?}",
        std::any::type_name::<T>(),
        state.get()
    );
}

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
enum AppState {
    #[default]
    MainMenu,
    LevelMenu,
    Game,
}

fn handle_main_menu_events(
    mut commands: Commands,
    mut events: EventReader<MainMenuEvent>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut game_mode: ResMut<GameMode>,
    windows: Query<(Entity, &Window)>,
) {
    for event in events.read() {
        match event {
            MainMenuEvent::PuzzlePressed => {
                app_state.set(AppState::LevelMenu);
                game_state.set(GameState::Loading);
                *game_mode = GameMode::Puzzle;
            }
            MainMenuEvent::RoguePressed => {
                app_state.set(AppState::LevelMenu);
                game_state.set(GameState::Loading);
                *game_mode = GameMode::Rogue;
            }
            MainMenuEvent::ExitPressed => {
                for (window_id, window) in windows.iter() {
                    if !window.focused {
                        continue;
                    }
                    commands.entity(window_id).despawn_recursive();
                }
            }
        }
    }
}

fn handle_level_menu_events(
    mut events: EventReader<LevelMenuEvent>,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        match event {
            LevelMenuEvent::PlayPressed => {
                app_state.set(AppState::Game);
            }
            LevelMenuEvent::BackPressed => {
                app_state.set(AppState::MainMenu);
                game_state.set(GameState::None);
            }
        }
    }
}

fn main() {
    App::new()
        .insert_resource(bevy::asset::AssetMetaCheck::Never)
        .add_state::<AppState>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(CameraPlugin)
        .add_plugins(MenuUIPlugin)
        .add_plugins(LevelUIPlugin)
        .add_plugins(TooltipPlugin)
        .add_plugins(GamePlugin)
        .add_systems(
            Update,
            (handle_main_menu_events).run_if(in_state(AppState::MainMenu)),
        )
        .add_systems(OnEnter(AppState::MainMenu), main_menu::spawn)
        .add_systems(OnExit(AppState::MainMenu), main_menu::despawn)
        .add_systems(
            Update,
            (handle_level_menu_events).run_if(in_state(AppState::LevelMenu)),
        )
        .add_systems(OnEnter(AppState::LevelMenu), level_menu::spawn)
        .add_systems(OnExit(AppState::LevelMenu), level_menu::despawn)
        .add_systems(OnEnter(AppState::Game), spawn_game_ui)
        .add_systems(OnExit(AppState::Game), despawn_game_ui)
        .add_systems(
            Update,
            (
                handle_input,
                print_state_change::<AppState>.run_if(state_changed::<AppState>()),
                print_state_change::<GameState>.run_if(state_changed::<GameState>()),
                print_state_change::<TurnState>.run_if(state_changed::<TurnState>()),
            ),
        )
        .add_systems(
            PostUpdate,
            (update_position_transforms,)
                .before(bevy::transform::TransformSystem::TransformPropagate),
        )
        .run();
}
