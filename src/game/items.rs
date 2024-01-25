use super::*;

#[derive(Component)]
pub enum Item {
    Water,
    Energy,
    Card(ContentID),
}

impl Item {
    pub fn random(content_range: usize) -> Self {
        match rand::random::<u8>() % 3 {
            0 => Self::Water,
            1 => Self::Energy,
            2 => Self::Card(ContentID((rand::random::<usize>() % content_range) + 1)),
            _ => unreachable!(),
        }
    }
}

pub fn add_item_sprite(
    mut commands: Commands,
    sprite_sheet: Res<ItemSpriteSheet>,
    query: Query<(Entity, &Item), (Added<Item>, With<GamePosition>)>,
) {
    for (item_id, item) in query.iter() {
        commands.entity(item_id).insert(SpriteSheetBundle {
            texture_atlas: sprite_sheet.0.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            sprite: match item {
                Item::Water => TextureAtlasSprite::new(0),
                Item::Energy => TextureAtlasSprite::new(1),
                Item::Card(_) => TextureAtlasSprite::new(2),
            },
            ..default()
        });
    }
}

pub fn apply_item(
    mut commands: Commands,
    mut deck_list: ResMut<DeckList>,
    mut deck: Query<&mut Deck, With<Player>>,
    card_sprites: Res<CardSpriteSheet>,
    players: Query<(Entity, &Energy, &Water), (With<Player>, Changed<GamePosition>)>,
    items: Query<(Entity, &Item), With<GamePosition>>,
    positions: Query<&GamePosition>,
) {
    if players.is_empty() || items.is_empty() {
        return;
    }
    let (player_id, energy, water) = players.get_single().expect("Should be exactly one player");
    let player_position = positions
        .get(player_id)
        .expect("Player should have position");
    for (item_id, item) in items.iter() {
        let item_position = positions.get(item_id).expect("Item should have position");
        if item_position.x == player_position.x && item_position.y == player_position.y {
            commands.entity(item_id).despawn_recursive();
            match item {
                Item::Water => {
                    commands.spawn(Change {
                        entity: player_id,
                        updated_value: Water {
                            current: water.current + 1,
                            ..water.clone()
                        },
                    });
                }
                Item::Energy => {
                    commands.spawn(Change {
                        entity: player_id,
                        updated_value: Energy {
                            current: energy.current + 1,
                            ..energy.clone()
                        },
                    });
                }
                Item::Card(content_id) => {
                    let mut deck = deck.get_single_mut().expect("Should be exactly one deck");
                    let card_instance_id = commands
                        .spawn((
                            content_id.clone(),
                            card_sprites.0.clone(),
                            InDeck,
                            CardStatus::Unknown,
                        ))
                        .id();
                    deck.add(card_instance_id);
                    deck_list.0.push(content_id.clone());
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct ItemSpriteSheet(Handle<TextureAtlas>);

impl FromWorld for ItemSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world
            .get_resource_mut::<AssetServer>()
            .expect("Failed get the `AssetServer` resource from the `World`");
        let texture_handle = asset_server.load("items.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 64.0), 3, 1, None, None);
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlas>>()
            .expect("Failed get the `Assets<TextureAtlas>` resource from the `World`");
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        Self(texture_atlas_handle)
    }
}
