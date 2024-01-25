use bevy::ui::RelativeCursorPosition;

use super::*;

struct SpawnMenuUI;

#[derive(Component)]
pub struct ResultMenu;

#[derive(Component, Debug)]
pub enum ResultMenuOption {
    NextLevel,
    TryAgain,
    MainMenu,
}

#[derive(Event)]
pub enum ResultMenuEvent {
    NextLevelPressed,
    TryAgainPressed,
    MainMenuPressed,
}

pub fn despawn(mut commands: Commands, menu_query: Query<Entity, With<ResultMenu>>) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spawn(mut commands: Commands) {
    commands.add(SpawnMenuUI);
}

pub fn handle_interactions(
    mut events: EventWriter<ResultMenuEvent>,
    interaction_query: Query<(&Interaction, &ResultMenuOption), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, option) in interaction_query.iter() {
        match (*interaction, option) {
            (Interaction::Pressed, ResultMenuOption::NextLevel) => {
                events.send(ResultMenuEvent::NextLevelPressed);
            }
            (Interaction::Pressed, ResultMenuOption::TryAgain) => {
                events.send(ResultMenuEvent::TryAgainPressed);
            }
            (Interaction::Pressed, ResultMenuOption::MainMenu) => {
                events.send(ResultMenuEvent::MainMenuPressed);
            }
            _ => {}
        }
    }
}

impl bevy::ecs::system::Command for SpawnMenuUI {
    fn apply(self, world: &mut World) {
        let was_win = false;
        let game_mode = world.get_resource::<GameMode>().unwrap().clone();
        world
            .spawn((
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceEvenly,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::BLACK.into(),
                    ..default()
                },
                ResultMenu,
            ))
            .with_children(|parent| {
                parent.spawn((TextBundle::from_section(
                    if was_win { "Level Complete" } else { "Level Failed" },
                    TextStyle {
                        font_size: 80.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),));
                if !was_win && game_mode == GameMode::Puzzle {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style { 
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    min_height: Val::Px(64.0),
                                    min_width: Val::Vw(30.0),
                                    ..default() 
                                },
                                background_color: Color::TEAL.into(),
                                ..default()
                            },
                            RelativeCursorPosition::default(),
                            Tooltip {
                                text: "Try the same puzzle again".to_string(),
                                threshold: 0.5,
                            },
                            ResultMenuOption::TryAgain,
                        ))
                        .with_children(|button| {
                            button.spawn((TextBundle::from_section(
                                "Try Again",
                                TextStyle {
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),));
                        });
                }
                if was_win {
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style { 
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    min_height: Val::Px(64.0),
                                    min_width: Val::Vw(30.0),
                                    ..default() 
                                },
                                background_color: Color::TEAL.into(),
                                ..default()
                            },
                            RelativeCursorPosition::default(),
                            Tooltip {
                                text: "Play the next level"
                                    .to_string(),
                                threshold: 0.5,
                            },
                            ResultMenuOption::NextLevel,
                        ))
                        .with_children(|button| {
                            button.spawn((TextBundle::from_section(
                                "Next Level",
                                TextStyle {
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),));
                        });
                }
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style { 
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                min_height: Val::Px(64.0),
                                min_width: Val::Vw(30.0),
                                ..default() 
                            },
                            background_color: Color::MAROON.into(),
                            ..default()
                        },
                        ResultMenuOption::MainMenu,
                    ))
                    .with_children(|button| {
                        button.spawn((TextBundle::from_section(
                            "Main Menu",
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
