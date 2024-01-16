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
                    border: UiRect::all(Val::Px(4.0)),
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

pub fn update_playable_indicator(
    playables: Query<Entity, With<Playable>>,
    hand: Query<&Hand>,
    mut card_uis: Query<(&CardUISlot, &mut BorderColor)>,
) {

    if hand.is_empty() { return; }
    let hand = hand.get_single().expect("There should only be one player hand");

    for (slot, mut border) in card_uis.iter_mut() {
        if let Some(card_id) = hand.0[slot.0] {
            if playables.get(card_id).is_ok() {
                border.0 = Color::GREEN.into();
            } else {
                border.0 = Color::RED.into();
            }
        } else {
            border.0 = Color::NONE.into();
        }
    }
}

pub fn update_hand_ui(
    hands: Query<&Hand, (With<Player>, Changed<Hand>)>,
    mut card_uis: Query<(&CardUISlot, &mut BackgroundColor)>,
) {
    if hands.is_empty() { return; }
    let hand = hands.get_single().expect("There should only be one player hand");
    for (slot, mut background) in card_uis.iter_mut() {
        match hand.0[slot.0] {
            Some(_) => {
                background.0 = Color::WHITE.into();
            },
            None => background.0 = Color::PINK.into(),
        }
    }
}