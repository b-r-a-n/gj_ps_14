use super::*;

#[derive(PartialEq, Eq)]
pub enum Level {
    Low = 1,
    Medium,
    High,
}

#[derive(PartialEq, Eq, Component)]
pub enum Tile {
    Empty,
    Wall,
    Fire(Level),
}

#[derive(Default)]
pub struct SpawnTiles {
    pub columns: i32,
    pub rows: i32,
}

pub fn spawn_tiles(
    mut commands: Commands
) {
    commands.add(SpawnTiles { columns: 10, rows: 10, ..default() });
}

#[derive(Resource)]
pub struct TileSpriteSheet(Handle<TextureAtlas>);

impl FromWorld for TileSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>()
            .expect("Failed get the `AssetServer` resource from the `World`");
        let texture_handle = asset_server.load("tiles.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle, 
            Vec2::new(64.0, 64.0), 
            5, 
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

impl bevy::ecs::system::Command for SpawnTiles {
    fn apply(self, world: &mut World) {
        let sprite_sheet = world.get_resource::<TileSpriteSheet>()
            .expect("Failed get the `TileSpriteSheet` resource from the `World`");
        let atlas = sprite_sheet.0.clone();
        let mut is_wall;
        let mut entities: Vec<Vec<Entity>> = Vec::new();
        for y in 0..=self.rows+1 {
            entities.push(Vec::new());
            for x in 0..=self.columns+1 {
                if x == 0 || y == 0 || x == self.columns+1 || y == self.rows+1 {
                    is_wall = true;
                } else {
                    is_wall = false;
                }
                let mut ec = world.spawn((
                    GamePosition {
                        x,
                        y,
                        ..default()
                    },
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(if is_wall { 2 } else { 0 }),
                        texture_atlas: atlas.clone(),
                        ..default()
                    },
                    if is_wall { Tile::Wall} else { Tile::Empty },
                ));
                if is_wall { ec.insert(BlockedTile); }
                entities[y as usize].push(ec.id());
            }
        }
        world.spawn(Grid(entities));
    }
}

pub fn add_random_fire_tiles( mut commands: Commands,
    tiles: Query<(Entity, &Tile)>,
) {
    let mut rng = rand::thread_rng();
    for (tile_id, tile) in tiles.iter() {
        if tile == &Tile::Wall || rng.gen_bool(0.9) {
            continue;
        }
        commands.entity(tile_id)
            .insert(Tile::Fire(Level::Low));
    }
}

pub fn update_tiles(
    mut commands: Commands,
    mut tiles: Query<(Entity, &Tile, &mut TextureAtlasSprite)>,
) {
    for (tile_id, tile, mut sprite_index) in tiles.iter_mut() {
        match tile {
            Tile::Fire(level) => {
                sprite_index.index = match level { Level::Low => 1, Level:: Medium => 2, Level::High => 3 };
            },
            Tile::Wall => {
                sprite_index.index = 4;
                commands.entity(tile_id).insert(BlockedTile);
            }
            _ => {
                sprite_index.index = 0;
                commands.entity(tile_id).remove::<BlockedTile>();
            }
        }
    }
}