use super::*;

#[derive(PartialEq, Eq)]
pub enum Intensity {
    Low = 1,
    Medium,
    High,
}

#[derive(PartialEq, Eq, Component)]
pub enum Tile {
    Empty,
    Wall,
    Fire(Intensity),
}

#[derive(Default)]
pub struct SpawnTiles {
    pub columns: i32,
    pub rows: i32,
}

pub enum FlameSpawner {
    Chance(f32, i32, i32),
    Static(Vec<(i32, i32)>),
}

#[derive(Resource)]
pub struct MapParameters {
    pub columns: i32,
    pub rows: i32,
    pub flame_spawner: FlameSpawner,
}

pub fn spawn_tiles(
    mut commands: Commands,
    map_parameters: Res<MapParameters>,
) {
    commands.add(SpawnTiles { columns: map_parameters.columns, rows: map_parameters.rows });
}

pub fn despawn_tiles(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Tile>, With<Grid>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
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

pub fn spawn_fires( mut commands: Commands,
    parameters: Res<MapParameters>,
    tiles: Query<(Entity, &Tile)>,
    grid: Query<&Grid>,
) {
    match &parameters.flame_spawner {
        FlameSpawner::Chance(chance, min_count, max_count) => {
            let mut rng = rand::thread_rng();
            let mut tile_count = 0;
            let mut potential_tiles = Vec::new();
            for (tile_id, tile) in tiles.iter() {
                let prob = 1.0 - chance;
                if tile != &Tile::Wall {
                    potential_tiles.push(tile_id);
                }
                if tile == &Tile::Wall || rng.gen_bool(prob.into()) || tile_count >= *max_count {
                    continue;
                }
                tile_count += 1;
                commands.entity(tile_id)
                    .insert(Tile::Fire(Intensity::Low));
            }
            if tile_count < *min_count {
                // Pick a random non-wall tile and set it on fire
                let random_index = rng.gen_range(0..potential_tiles.len());
                commands.entity(potential_tiles[random_index])
                    .insert(Tile::Fire(Intensity::Low));
            }
        },
        FlameSpawner::Static(positions) => {
            let grid = grid.single();
            for position in positions.iter() {
                let tile_id = grid.get(&GamePosition{x: position.0, y: position.1, ..default()});
                commands.entity(tile_id)
                    .insert(Tile::Fire(Intensity::Low));
            }
        }
    }
}

pub fn update_tiles(
    mut commands: Commands,
    mut tiles: Query<(Entity, &Tile, &mut TextureAtlasSprite)>,
) {
    for (tile_id, tile, mut sprite_index) in tiles.iter_mut() {
        match tile {
            Tile::Fire(level) => {
                sprite_index.index = match level { Intensity::Low => 1, Intensity:: Medium => 2, Intensity::High => 3 };
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