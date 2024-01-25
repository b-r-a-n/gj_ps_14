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
pub enum Spawner {
    Chance(f32, i32, i32),
    Static(Vec<(i32, i32)>),
}

impl Default for Spawner {
    fn default() -> Self {
        Self::Static(vec![])
    }
}

#[derive(Clone, Default, Resource)]
pub struct MapParameters {
    pub columns: i32,
    pub rows: i32,
    pub flame_spawner: Spawner,
    pub item_spawner: Spawner,
}

pub fn spawn_tiles(mut commands: Commands) {
    commands.add(SpawnTiles);
}

pub fn despawn_tiles_and_items(
    mut commands: Commands, 
    query: Query<Entity, Or<(With<Tile>, With<Grid>, With<Item>, With<Animation>)>>) {
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
            TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 7, 1, None, None);
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

fn tile_is_flame(
    x: i32,
    y: i32,
    map: &MapParameters,
    flame_count: i32,
    pre_flames: &HashMap<(i32, i32), bool>,
) -> bool {
    match &map.flame_spawner {
        Spawner::Chance(chance, _min_count, max_count) => {
            if pre_flames.contains_key(&(x, y)) {
                return true;
            }
            if flame_count >= *max_count {
                return false;
            }
            let mut rng = rand::thread_rng();
            rng.gen_bool((1.0 - chance).into())
        }
        Spawner::Static(positions) => positions.contains(&(x, y)),
    }
}

fn random_non_wall_tile(map: &MapParameters) -> (i32, i32) {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(1..=map.columns);
    // Hack to prevent the flame from spawning on the start tile
    if x == 1 {
        (x, rng.gen_range(2..=map.rows))
    } else {
        (x, rng.gen_range(1..=map.rows))
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
        let mut flame_count = 0;
        let mut pre_def_flames = HashMap::new();
        let mut items: Vec<(Item, GamePosition)> = Vec::new();
        match map.flame_spawner {
            Spawner::Chance(_, min_count, _) => {
                for _ in 0..min_count {
                    // TODO this can hit the same tile more than once
                    let (x, y) = random_non_wall_tile(&map);
                    pre_def_flames.insert((x, y), true);
                }
            }
            _ => {}
        }
        for y in 0..=map.rows + 1 {
            entities.push(Vec::new());
            for x in 0..=map.columns + 1 {
                is_wall = tile_is_wall(x, y, &map);
                is_flame = tile_is_flame(x, y, &map, flame_count, &pre_def_flames);
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
                            flame_count += 1;
                            Tile::Fire(Intensity::Low)
                        } else {
                            match map.item_spawner {
                                Spawner::Chance(c, _, _) => {
                                    let mut rng = rand::thread_rng();
                                    if rng.gen_bool(c.into()) {
                                        // TODO the content_range param should check the actual card_infos
                                        items.push((Item::random(20), GamePosition { x, y, ..default() }));
                                    }
                                },
                                Spawner::Static(ref positions) => {
                                    if positions.contains(&(x, y)) {
                                        info!("Item would spawn at {}, {} due to static spawn config.", x, y);
                                    }
                                }
                            }
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
        world.spawn_batch(items);
    }
}

pub fn update_tiles(
    mut commands: Commands,
    mut tiles: Query<(Entity, &Tile, &mut TextureAtlasSprite, Option<&Animating>)>,
    animations: Query<&Animation>,
) {
    for (tile_id, tile, mut sprite_index, animating) in tiles.iter_mut() {
        if let Some(animating) = animating {
            let animation = animations.get(animating.0)
                .expect("Animation should exist");
            match animation.animation_type {
                AnimationType::Blue(_) => {
                    sprite_index.index = 5;
                    continue;
                }
                AnimationType::Smoke(_) => {
                    sprite_index.index = 6;
                    continue;
                }
                _ => {}
            }
        }
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
