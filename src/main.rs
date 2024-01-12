use bevy::prelude::*;

#[derive(Component)]
struct Energy {
    current: i32,
    maxium: i32,
}

#[derive(Default)]
enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Default)]
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
                current: 0,
                maxium: self.max_energy,
            },
            SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: sprite_sheet.0.clone(),
                ..default()
            }
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

fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Position>
) {
    match keyboard_input.get_just_released().last() {
        Some(KeyCode::Up) => {
            for mut position in query.iter_mut() {
                position.y += 1;
                position.d = Direction::Up;
            }
        },
        Some(KeyCode::Down) => {
            for mut position in query.iter_mut() {
                position.y -= 1;
                position.d = Direction::Down;
            }
        },
        Some(KeyCode::Left) => {
            for mut position in query.iter_mut() {
                position.x -= 1;
                position.d = Direction::Left;
            }
        },
        Some(KeyCode::Right) => {
            for mut position in query.iter_mut() {
                position.x += 1;
                position.d = Direction::Right;
            }
        },
        _ => {},
    }
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<PlayerSpriteSheet>()
        .add_systems(Startup, |mut commands: Commands| commands.add(SpawnCamera::default()))
        .add_systems(Startup, |mut commands: Commands| commands.add(SpawnPlayer::default()))
        .add_systems(Update, handle_input)
        .add_systems(PostUpdate, update_position_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}
