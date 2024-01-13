use super::*;
pub mod energy;
pub mod hand;

pub struct UIPlugins;

impl Plugin for UIPlugins {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, |mut commands: Commands| commands.add(SpawnEnergyUI::default()))
            .add_systems(Startup, |mut commands: Commands| commands.add(SpawnHandUI::default()))
            .add_systems(Update, (update_energy_ui, update_hand_ui))
        ;
    }
}