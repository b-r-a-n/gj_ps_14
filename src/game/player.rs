use super::*;

#[derive(Component)]
pub struct Player;

pub fn spawn_player(
    mut commands: Commands
) {
    commands.spawn(PlayerBundle::new());
}

pub fn despawn_player(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
) {
    for player_id in players.iter() {
        commands.entity(player_id).despawn_recursive();
    }
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

#[derive(Bundle)]
pub struct PlayerBundle {
    player: Player,
    game_position: GamePosition,
    energy: Energy,
    water: Water,
    deck: Deck,
    hand: Hand,
}

impl PlayerBundle {
    pub fn new() -> Self {
        Self {
            player: Player,
            game_position: GamePosition { x: 1, y: 1, d: GameDirection::Up },
            energy: Energy { current: 0, maxium: 10 },
            water: Water { current: 0, maxium: 10 },
            deck: Deck {
                cards: vec![],
                recycled: vec![],
                discarded: vec![],
            },
            hand: Hand([None; 5]),
        }
    }
}

pub fn add_sprite(
    mut commands: Commands,
    sprite_sheet: Res<PlayerSpriteSheet>,
    player_query: Query<Entity, With<Player>>,
) {
    for player_id in player_query.iter() {
        commands.entity(player_id).insert(SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: sprite_sheet.0.clone(),
            ..default()
        });
    }
}