use super::*;

#[derive(Default)]
struct SpawnCamera;

#[derive(Component)]
pub struct MainCamera;

impl bevy::ecs::system::Command for SpawnCamera {
    fn apply(self, world: &mut World) {
        world.spawn((Camera2dBundle::default(), MainCamera));
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, |mut commands: Commands| commands.add(SpawnCamera));
    }
}