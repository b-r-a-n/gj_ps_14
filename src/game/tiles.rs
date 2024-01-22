use super::*;

#[derive(Clone, PartialEq, Eq)]
pub enum Intensity {
    Low = 1,
    Medium,
    High,
}

#[derive(Clone, PartialEq, Eq, Component)]
pub enum Tile {
    Empty,
    Wall,
    Fire(Intensity),
}

#[derive(Default)]
pub struct SpawnTiles;

#[allow(dead_code)]
#[derive(Clone)]
pub enum FlameSpawner {
    Chance(f32, i32, i32),
    Static(Vec<(i32, i32)>),
}

impl Default for FlameSpawner {
    fn default() -> Self {
        Self::Static(vec![])
    }
}

#[derive(Clone, Default, Resource)]
pub struct MapParameters {
    pub columns: i32,
    pub rows: i32,
    pub flame_spawner: FlameSpawner,
}

pub fn spawn_tiles(mut commands: Commands) {
    commands.add(SpawnTiles);
}

pub fn despawn_tiles(mut commands: Commands, query: Query<Entity, Or<(With<Tile>, With<Grid>)>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Resource)]
pub struct TileSpriteSheet(Handle<TextureAtlas>);

impl FromWorld for TileSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world
            .get_resource_mut::<AssetServer>()
            .expect("Failed get the `AssetServer` resource from the `World`");
        let texture_handle = asset_server.load("tiles.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 5, 1, None, None);
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlas>>()
            .expect("Failed get the `Assets<TextureAtlas>` resource from the `World`");
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        Self(texture_atlas_handle)
    }
}

fn tile_is_wall(x: i32, y: i32, map: &MapParameters) -> bool {
    x == 0 || y == 0 || x == map.columns + 1 || y == map.rows + 1
}

fn tile_is_flame(x: i32, y: i32, map: &MapParameters) -> bool {
    match &map.flame_spawner {
        FlameSpawner::Chance(chance, _min_count, _max_count) => {
            let mut rng = rand::thread_rng();
            rng.gen_bool((1.0 - chance).into())
        }
        FlameSpawner::Static(positions) => positions.contains(&(x, y)),
    }
}

impl bevy::ecs::system::Command for SpawnTiles {
    fn apply(self, world: &mut World) {
        let sprite_sheet = world
            .get_resource::<TileSpriteSheet>()
            .expect("Failed get the `TileSpriteSheet` resource from the `World`");
        let map: MapParameters = world
            .get_resource::<MapParameters>()
            .expect("Failed get the `MapParameters` resource from the `World`")
            .clone();
        let atlas = sprite_sheet.0.clone();
        let mut is_wall;
        let mut is_flame;
        let mut entities: Vec<Vec<Entity>> = Vec::new();
        for y in 0..=map.rows + 1 {
            entities.push(Vec::new());
            for x in 0..=map.columns + 1 {
                is_wall = tile_is_wall(x, y, &map);
                is_flame = tile_is_flame(x, y, &map);
                let mut ec = world.spawn((
                    GamePosition { x, y, ..default() },
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(if is_wall {
                            4
                        } else {
                            if is_flame {
                                1
                            } else {
                                0
                            }
                        }),
                        texture_atlas: atlas.clone(),
                        ..default()
                    },
                    if is_wall {
                        Tile::Wall
                    } else {
                        if is_flame {
                            Tile::Fire(Intensity::Low)
                        } else {
                            Tile::Empty
                        }
                    },
                ));
                if is_wall {
                    ec.insert(BlockedTile);
                }
                entities[y as usize].push(ec.id());
            }
        }
        world.spawn(Grid(entities));
    }
}

pub fn update_tiles(
    mut commands: Commands,
    mut tiles: Query<(Entity, &Tile, &mut TextureAtlasSprite)>,
) {
    for (tile_id, tile, mut sprite_index) in tiles.iter_mut() {
        match tile {
            Tile::Fire(level) => {
                sprite_index.index = match level {
                    Intensity::Low => 1,
                    Intensity::Medium => 2,
                    Intensity::High => 3,
                };
            }
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
