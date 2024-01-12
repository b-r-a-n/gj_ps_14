use bevy::prelude::*;

#[derive(Component, Clone)]
struct Energy {
    current: i32,
    maxium: i32,
}

#[derive(Component)]
struct EnergyUI;

#[derive(Default)]
struct SpawnEnergyUI;

impl bevy::ecs::system::Command for SpawnEnergyUI {
    fn apply(self, world: &mut World) {
        world.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 100.0,
                    color: Color::YELLOW,
                    ..default()
                }
            )
                .with_text_alignment(TextAlignment::Left)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                }
            ),
            EnergyUI,
        ));
    }
}

fn update_energy_ui(
    energy: Query<&Energy, (With<Player>, Changed<Energy>)>,
    mut text: Query<&mut Text, With<EnergyUI>>,
) {
    if energy.is_empty() || text.is_empty() {
        return;
    }
    let energy = energy.get_single()
        .expect("Found more than one player energy");
    let mut text = text.get_single_mut()
        .expect("Found more than one energy UI text");
    text.sections[0].value = format!("Energy: {}/{}", energy.current, energy.maxium);
}

#[derive(Clone, Default)]
enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Component, Default)]
struct Position {
    x: i32,
    y: i32,
    d: Direction,
}

#[derive(Default)]
struct SpawnPlayer {
    x: i32,
    y: i32,
    facing_direction: Direction,
    max_energy: i32,
}

#[derive(Resource)]
struct PlayerSpriteSheet(Handle<TextureAtlas>);

impl FromWorld for PlayerSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>()
            .expect("Failed get the `AssetServer` resource from the `World`");
        let texture_handle = asset_server.load("player.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle, 
            Vec2::new(64.0, 64.0), 
            1, 
            1, 
            None, 
            None
        );
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>()
            .expect("Failed get the `Assets<TextureAtlas>` resource from the `World`");
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        Self(texture_atlas_handle)
    }
}

impl bevy::ecs::system::Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        let sprite_sheet = world.get_resource::<PlayerSpriteSheet>()
            .expect("Failed get the `PlayerSpriteSheet` resource from the `World`");
        world.spawn((
            Position {
                x: self.x,
                y: self.y,
                d: self.facing_direction,
            },
            Energy {
                current: self.max_energy/2,
                maxium: self.max_energy,
            },
            SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: sprite_sheet.0.clone(),
                ..default()
            },
            Player,
        ));
    }
}

#[derive(Default)]
struct SpawnCamera;

impl bevy::ecs::system::Command for SpawnCamera {
    fn apply(self, world: &mut World) {
        world.spawn(Camera2dBundle::default());
    }
}

fn update_position_transforms(
    mut query: Query<(&Position, &mut Transform)>
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x as f32 * 64.0;
        transform.translation.y = position.y as f32 * 64.0;
        transform.rotation = match position.d {
            Direction::Up => Quat::from_rotation_z(0.0),
            Direction::Down => Quat::from_rotation_z(std::f32::consts::PI),
            Direction::Left => Quat::from_rotation_z(std::f32::consts::PI * 0.5),
            Direction::Right => Quat::from_rotation_z(std::f32::consts::PI * 1.5),
        };
    }
}

#[derive(Component)]
struct ChangeAction<T: Component + Clone> {
    entity: Entity,
    updated_value: T,
}

#[derive(Component)]
struct Player;

fn apply_change_actions<T: Component + Clone>(
    mut commands: Commands,
    actions: Query<&ChangeAction<T>>, 
) {
    for action in actions.iter() {
        commands.entity(action.entity)
            .remove::<ChangeAction<T>>()
            .insert(action.updated_value.clone());
    }
}

fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    player_state: Query<(Entity, &Position, &Energy), With<Player>>,
) {
    if keyboard_input.get_just_released().last().is_none() {
        return;
    }
    let (entity, position, energy) = player_state.get_single()
        .expect("Found more than one player position");

    let energy_change = ChangeAction {
        entity,
        updated_value: Energy {
            current: energy.current - 1,
            ..energy.clone()
        },
    };

    match keyboard_input.get_just_released().last() {
        Some(KeyCode::Up) => {
            commands.spawn((
                ChangeAction {
                    entity,
                    updated_value: Position {
                        y: position.y + 1,
                        d: Direction::Up,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        Some(KeyCode::Down) => {
            commands.spawn((
                ChangeAction {
                    entity,
                    updated_value: Position {
                        y: position.y - 1,
                        d: Direction::Down,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        Some(KeyCode::Left) => {
            commands.spawn((
                ChangeAction {
                    entity,
                    updated_value: Position {
                        x: position.x - 1,
                        d: Direction::Left,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        Some(KeyCode::Right) => {
            commands.spawn((
                ChangeAction {
                    entity,
                    updated_value: Position {
                        x: position.x + 1,
                        d: Direction::Right,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        _ => {},
    }
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<PlayerSpriteSheet>()
        .add_systems(Startup, |mut commands: Commands| commands.add(SpawnCamera::default()))
        .add_systems(Startup, |mut commands: Commands| commands.add(SpawnPlayer { max_energy: 100, ..default() }))
        .add_systems(Startup, |mut commands: Commands| commands.add(SpawnEnergyUI::default()))
        .add_systems(Update, (handle_input, update_energy_ui))
        .add_systems(Update, (apply_change_actions::<Position>, apply_change_actions::<Energy>))
        .add_systems(PostUpdate, update_position_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}
