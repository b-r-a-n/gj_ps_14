use super::*;

pub struct SpawnMenuUI;

#[derive(Component)]
pub struct MainMenu;

#[derive(Component, Debug)]
pub enum MenuOption {
    Play,
    Back,
}

pub fn despawn(
    mut commands: Commands,
    menu_query: Query<Entity, With<MainMenu>>,
) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn handle_interactions(
    mut commands: Commands,
    mut app_state: ResMut<NextState<AppState>>,
    interaction_query: Query<(&Interaction, &MenuOption), (Changed<Interaction>, With<Button>)>,
    windows: Query<(Entity, &Window)>,
) {
    for (interaction, option) in interaction_query.iter() {
        match (*interaction, option) {
            (Interaction::Pressed, MenuOption::Play) => {
                app_state.set(AppState::Game);
            },
            (Interaction::Pressed, MenuOption::Back) => {
                app_state.set(AppState::MainMenu);
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
                    "Next Level",
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
                MenuOption::Play,
            )).with_children(|button| {
                button.spawn((
                    TextBundle::from_section(
                        "Play",
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
                MenuOption::Back,
            )).with_children(|button| {
                button.spawn((
                    TextBundle::from_section(
                        "Back",
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