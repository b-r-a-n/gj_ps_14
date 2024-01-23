use super::*;
use bevy::ui::RelativeCursorPosition;

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

#[derive(Component)]
pub struct EndTurnButton;

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

pub fn spawn_resource_ui(world: &mut World) -> Entity {
    let container_id = world
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceEvenly,
                    width: Val::Px(112.0),
                    height: Val::Px(CARD_HEIGHT),
                    margin: UiRect {
                        left: Val::Px(8.0),
                        bottom: Val::Px(8.0),
                        top: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            ResourceUI,
        ))
        .id();
    world
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(64.0),
                    border: UiRect::all(Val::Px(2.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                border_color: Color::WHITE.into(),
                background_color: Color::NONE.into(),
                ..default()
            },
            EndTurnButton,
            RelativeCursorPosition::default(),
            Tooltip {
                text: "End your turn".to_string(),
                threshold: 1.5,
            },
        ))
        .with_children(|button| {
            button.spawn((TextBundle::from_section(
                "End Turn",
                TextStyle {
                    font_size: 14.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),));
        })
        .set_parent(container_id);
    world
        .spawn((AtlasImageBundle {
            style: Style {
                width: Val::Px(64.0),
                height: Val::Px(32.0),
                left: Val::Px(6.0),
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
    container_id
}

impl bevy::ecs::system::Command for SpawnResourceUI {
    fn apply(self, world: &mut World) {
        spawn_resource_ui(world);
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
