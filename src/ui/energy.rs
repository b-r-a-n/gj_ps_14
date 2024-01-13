use super::*;

#[derive(Component)]
pub struct EnergyUI;

#[derive(Default)]
pub struct SpawnEnergyUI;

impl bevy::ecs::system::Command for SpawnEnergyUI {
    fn apply(self, world: &mut World) {
        world.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 100.0,
                    color: Color::YELLOW,
                    ..default()
                }
            )
                .with_text_alignment(TextAlignment::Left)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                }
            ),
            EnergyUI,
        ));
    }
}

pub fn update_energy_ui(
    energy: Query<&Energy, (With<Player>, Changed<Energy>)>,
    mut text: Query<&mut Text, With<EnergyUI>>,
) {
    if energy.is_empty() || text.is_empty() {
        return;
    }
    let energy = energy.get_single()
        .expect("Found more than one player energy");
    let mut text = text.get_single_mut()
        .expect("Found more than one energy UI text");
    text.sections[0].value = format!("Energy: {}/{}", energy.current, energy.maxium);
}