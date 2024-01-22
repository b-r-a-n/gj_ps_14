use bevy::prelude::*;
use camera::*;
use game::*;
use ui::*;

mod camera;
mod game;
mod ui;

fn update_position_transforms(mut query: Query<(&GamePosition, &mut Transform)>) {
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
        Some(KeyCode::Space) => {}
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
    windows: Query<(Entity, &Window)>,
) {
    for event in events.read() {
        match event {
            MainMenuEvent::NewGamePressed => {
                app_state.set(AppState::LevelMenu);
                game_state.set(GameState::Loading);
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
        .add_state::<AppState>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(CameraPlugin)
        .add_plugins(MenuUIPlugin)
        .add_plugins(LevelUIPlugin)
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
            update_position_transforms.before(bevy::transform::TransformSystem::TransformPropagate),
        )
        .run();
}
