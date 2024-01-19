use super::*;

struct SpawnMenuUI;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component, Debug)]
pub enum MainMenuOption {
    NewGame,
    Exit,
}

#[derive(Event)]
pub enum MainMenuEvent {
    NewGamePressed,
    ExitPressed,
}

pub fn despawn(
    mut commands: Commands,
    menu_query: Query<Entity, With<MainMenu>>,
) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn spawn(
    mut commands: Commands,
) {
    commands.add(SpawnMenuUI);
}

pub fn handle_interactions(
    mut events: EventWriter<MainMenuEvent>,
    interaction_query: Query<(&Interaction, &MainMenuOption), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, option) in interaction_query.iter() {
        match (*interaction, option) {
            (Interaction::Pressed, MainMenuOption::NewGame) => {
                events.send(MainMenuEvent::NewGamePressed);
            },
            (Interaction::Pressed, MainMenuOption::Exit) => {
                events.send(MainMenuEvent::ExitPressed);
            },
            _ => {},
        }
    }
}

impl bevy::ecs::system::Command for SpawnMenuUI {

    fn apply(self, world: &mut World) {
        world.spawn((
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
            MainMenu,
        )).with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "Flame Fighters",
                    TextStyle {
                        font_size: 80.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
            ));
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        ..default()
                    },
                    background_color: Color::TEAL.into(),
                    ..default()
                },
                MainMenuOption::NewGame,
            )).with_children(|button| {
                button.spawn((
                    TextBundle::from_section(
                        "New Game",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                ));
            });
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        ..default()
                    },
                    background_color: Color::MAROON.into(),
                    ..default()
                },
                MainMenuOption::Exit,
            )).with_children(|button| {
                button.spawn((
                    TextBundle::from_section(
                        "Exit",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    ),
                ));
            });
        });
    }

}