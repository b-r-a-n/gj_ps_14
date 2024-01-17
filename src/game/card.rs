use super::*;

pub struct SpawnCard {
    pub base_card_info: Entity,
}

#[derive(Resource)]
pub struct CardSpriteSheet(pub Handle<TextureAtlas>);

impl FromWorld for CardSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>()
            .expect("Failed get the `AssetServer` resource from the `World`");
        let texture_handle = asset_server.load("cards.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle, 
            Vec2::new(160.0, 160.0), 
            4, 
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

impl bevy::ecs::system::Command for SpawnCard {
    fn apply(self, world: &mut World) {
        world.spawn((
            BaseCardInfo(self.base_card_info),
        ));
    }
}