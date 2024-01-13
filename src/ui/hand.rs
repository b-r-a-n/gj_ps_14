use super::*;

#[derive(Component)]
pub struct HandUI;

#[derive(Default)]
pub struct SpawnHandUI;

impl bevy::ecs::system::Command for SpawnHandUI {
    fn apply(self, world: &mut World) {
        let hand_id = world.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(125.0*5.0),
                    height: Val::Px(150.0),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::End,
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    right: Val::Px(0.0),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            HandUI,
        )).id();
        let cards: Vec<Entity> = world.spawn_batch((0..5).map(|_| {
            NodeBundle {
                style: Style {
                    width: Val::Percent(20.0),
                    height: Val::Percent(100.0),
                    margin: UiRect { left: Val::Px(8.0), bottom: Val::Px(8.0), ..default() },
                    ..default()
                },
                background_color: Color::PINK.into(),
                ..default()
            }
        })).collect();
        let mut hand = world.get_entity_mut(hand_id).unwrap();
        hand.push_children(&cards);
    }
}