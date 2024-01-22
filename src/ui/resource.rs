use super::*;

#[derive(Component)]
pub struct EnergyUI;

#[derive(Component)]
pub struct WaterUI;

#[derive(Component)]
pub struct ResourceUI;

#[derive(Default)]
pub struct SpawnResourceUI;

#[derive(Resource)]
pub struct IconSpriteSheet(pub Handle<TextureAtlas>);

impl FromWorld for IconSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world
            .get_resource_mut::<AssetServer>()
            .expect("Failed get the `AssetServer` resource from the `World`");
        let texture_handle = asset_server.load("icons.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(64.0, 32.0), 5, 1, None, None);
        let mut texture_atlases = world
            .get_resource_mut::<Assets<TextureAtlas>>()
            .expect("Failed get the `Assets<TextureAtlas>` resource from the `World`");
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        Self(texture_atlas_handle)
    }
}

impl bevy::ecs::system::Command for SpawnResourceUI {
    fn apply(self, world: &mut World) {
        let container_id = world
            .spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                },
                ResourceUI,
            ))
            .id();
        world
            .spawn((AtlasImageBundle {
                style: Style {
                    width: Val::Px(64.0),
                    height: Val::Px(32.0),
                    left: Val::Px(6.0),
                    top: Val::Px(40.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::YELLOW.into(),
                texture_atlas: world.get_resource::<IconSpriteSheet>().unwrap().0.clone(),
                texture_atlas_image: UiTextureAtlasImage {
                    index: 0,
                    ..default()
                },
                ..default()
            },))
            .with_children(|icon| {
                icon.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::YELLOW,
                            ..default()
                        },
                    )
                    .with_style(Style {
                        left: Val::Percent(10.0),
                        width: Val::Percent(80.0),
                        ..default()
                    }),
                    EnergyUI,
                ));
            })
            .set_parent(container_id);
        world
            .spawn((AtlasImageBundle {
                style: Style {
                    width: Val::Px(64.0),
                    height: Val::Px(32.0),
                    left: Val::Px(6.0),
                    top: Val::Px(40.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::rgb(2.0 / 255.0, 204.0 / 255.0, 254.0 / 255.0).into(),
                texture_atlas: world.get_resource::<IconSpriteSheet>().unwrap().0.clone(),
                texture_atlas_image: UiTextureAtlasImage {
                    index: 1,
                    ..default()
                },
                ..default()
            },))
            .with_children(|icon| {
                icon.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::rgb(2.0 / 255.0, 204.0 / 255.0, 254.0 / 255.0),
                            ..default()
                        },
                    )
                    .with_style(Style {
                        left: Val::Percent(10.0),
                        width: Val::Percent(80.0),
                        ..default()
                    }),
                    WaterUI,
                ));
            })
            .set_parent(container_id);
    }
}

pub fn update_energy_ui(
    energy: Query<&Energy, (With<Player>, Changed<Energy>)>,
    mut text: Query<&mut Text, With<EnergyUI>>,
) {
    if energy.is_empty() || text.is_empty() {
        return;
    }
    let energy = energy
        .get_single()
        .expect("Found more than one player energy");
    let mut text = text
        .get_single_mut()
        .expect("Found more than one energy UI text");
    text.sections[0].value = format!("{}/{}", energy.current, energy.maxium);
}

pub fn update_water_ui(
    water: Query<&Water, (With<Player>, Changed<Water>)>,
    mut text: Query<&mut Text, With<WaterUI>>,
) {
    if water.is_empty() || text.is_empty() {
        return;
    }
    let water = water
        .get_single()
        .expect("Found more than one player energy");
    let mut text = text
        .get_single_mut()
        .expect("Found more than one water UI text");
    text.sections[0].value = format!("{}/{}", water.current, water.maxium);
}
