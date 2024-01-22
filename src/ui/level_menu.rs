use super::*;

struct SpawnMenuUI;

#[derive(Component)]
pub struct LevelMenu;

#[derive(Component, Debug)]
pub enum LevelMenuOption {
    Play,
    Back,
}

#[derive(Event)]
pub enum LevelMenuEvent {
    PlayPressed,
    BackPressed,
}

pub fn despawn(mut commands: Commands, menu_query: Query<Entity, With<LevelMenu>>) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spawn(mut commands: Commands) {
    commands.add(SpawnMenuUI);
}

pub fn handle_interactions(
    mut events: EventWriter<LevelMenuEvent>,
    interaction_query: Query<
        (&Interaction, &LevelMenuOption),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, option) in interaction_query.iter() {
        match (*interaction, option) {
            (Interaction::Pressed, LevelMenuOption::Play) => {
                events.send(LevelMenuEvent::PlayPressed);
            }
            (Interaction::Pressed, LevelMenuOption::Back) => {
                events.send(LevelMenuEvent::BackPressed);
            }
            _ => {}
        }
    }
}

impl bevy::ecs::system::Command for SpawnMenuUI {
    fn apply(self, world: &mut World) {
        world
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Column,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::BLACK.into(),
                    ..default()
                },
                LevelMenu,
            ))
            .with_children(|parent| {
                parent.spawn((TextBundle::from_section(
                    "Next Level",
                    TextStyle {
                        font_size: 80.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),));
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style { ..default() },
                            background_color: Color::TEAL.into(),
                            ..default()
                        },
                        LevelMenuOption::Play,
                    ))
                    .with_children(|button| {
                        button.spawn((TextBundle::from_section(
                            "Play",
                            TextStyle {
                                font_size: 40.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),));
                    });
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style { ..default() },
                            background_color: Color::MAROON.into(),
                            ..default()
                        },
                        LevelMenuOption::Back,
                    ))
                    .with_children(|button| {
                        button.spawn((TextBundle::from_section(
                            "Back",
                            TextStyle {
                                font_size: 40.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),));
                    });
            });
    }
}
