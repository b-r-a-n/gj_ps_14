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

fn center_camera_on_grid(
    mut cameras: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    added_grids: Query<&Grid, Added<Grid>>,
) {
    if added_grids.is_empty() {
        return;
    }
    let grid = added_grids.get_single().expect("Should only be one grid");
    for (mut transform, mut projection) in cameras.iter_mut() {
        let z = transform.translation.z;
        transform.translation.x = grid.0.len() as f32 * 64.0 / 2.0;
        transform.translation.y = grid.0[0].len() as f32 * 64.0 / 2.0 - 128.0;
        match grid.0.len() {
            0..=4 => {
                projection.scale = 1.0;
            }
            5..=8 => {
                projection.scale = 1.2;
            },
            _ => {
                projection.scale = 1.5;
            },
        }
        transform.translation.z = z;
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, |mut commands: Commands| commands.add(SpawnCamera))
            .add_systems(Update, center_camera_on_grid);
    }
}
