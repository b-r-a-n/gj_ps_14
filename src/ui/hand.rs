use super::*;

#[derive(Component)]
pub struct HandUI;

#[derive(Default)]
pub struct SpawnHandUI;

#[derive(Component)]
pub struct CardUISlot(usize);

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
        let cards: Vec<Entity> = world.spawn_batch((0..5).map(|i| {(
            NodeBundle {
                style: Style {
                    width: Val::Percent(20.0),
                    height: Val::Percent(100.0),
                    margin: UiRect { left: Val::Px(8.0), bottom: Val::Px(8.0), ..default() },
                    ..default()
                },
                background_color: Color::PINK.into(),
                ..default()
            },
            CardUISlot(i),
        )})).collect();
        let mut hand = world.get_entity_mut(hand_id).unwrap();
        hand.push_children(&cards);
    }
}

pub fn update_hand_ui(
    hands: Query<&Hand, (With<Player>, Changed<Hand>)>,
    mut card_uis: Query<(&CardUISlot, &mut BackgroundColor)>,
) {
    for hand in hands.iter() {
        for (slot, _) in hand.0.iter().enumerate() {
            for (ui_slot, mut background) in card_uis.iter_mut() {
                if slot == ui_slot.0 {
                    background.0 = Color::WHITE.into();
                }
            }

        }
    }
}