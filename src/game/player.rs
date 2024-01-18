use super::*;

#[derive(Component)]
pub struct Player;

#[derive(Default)]
pub struct SpawnPlayer {
    pub x: i32,
    pub y: i32,
    pub facing_direction: GameDirection,
    pub max_energy: i32,
}

pub fn spawn_player(
    mut commands: Commands
) {
    commands.add(SpawnPlayer { max_energy: 100, ..default() });
}

#[derive(Resource)]
pub struct PlayerSpriteSheet(Handle<TextureAtlas>);

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
            GamePosition {
                x: self.x,
                y: self.y,
                d: self.facing_direction,
            },
            Energy {
                current: self.max_energy/2,
                maxium: self.max_energy,
            },
            Water {
                current: self.max_energy/2,
                maxium: self.max_energy,
            },
            SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: sprite_sheet.0.clone(),
                ..default()
            },
            Player,
            Deck {
                cards: vec![],
                recycled: vec![],
                discarded: vec![],
            },
            Hand([None; 5]),
        ));
    }
}