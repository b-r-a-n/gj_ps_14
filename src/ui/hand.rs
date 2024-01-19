use super::*;

#[derive(Component)]
pub struct HandUI;

#[derive(Default)]
pub struct SpawnHandUI;

#[derive(Component)]
pub struct CardUISlot(usize);

const CARD_WIDTH: f32 = 140.0;
const CARD_HEIGHT: f32 = 200.0;

impl bevy::ecs::system::Command for SpawnHandUI {
    fn apply(self, world: &mut World) {
        let hand_id = world.spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::End,
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    right: Val::Px(0.0),
                    border: UiRect::right(Val::Px(8.0)),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            HandUI,
        )).id();
        let cards: Vec<Entity> = (0..5).map(|i| {
            world.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(CARD_WIDTH),
                        height: Val::Px(CARD_HEIGHT),
                        margin: UiRect { left: Val::Px(8.0), bottom: Val::Px(8.0), top: Val::Px(8.0), ..default() },
                        border: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: Color::PINK.into(),
                    ..default()
                },
            )).with_children(|parent| {
                parent.spawn((
                    AtlasImageBundle {
                        style: Style {
                            width: Val::Px(120.0),
                            height: Val::Px(120.0),
                            left: Val::Px(6.0),
                            top: Val::Px(40.0),
                            ..default()
                        },
                        ..default()
                    },
                    CardUISlot(i),
                ));
            })
            .id()
        }).collect();
        let mut hand = world.get_entity_mut(hand_id).unwrap();
        hand.push_children(&cards);
    }
}

pub fn update_playable_indicator(
    playables: Query<Entity, With<Playable>>,
    hand: Query<&Hand>,
    mut card_uis: Query<(&Parent, &CardUISlot)>,
    mut borders: Query<&mut BorderColor>
) {

    if hand.is_empty() { return; }
    let hand = hand.get_single().expect("There should only be one player hand");

    for (parent, slot) in card_uis.iter_mut() {
        if let Some(card_instance_id) = hand.0[slot.0] {
            if playables.get(card_instance_id).is_ok() {
                borders.get_mut(parent.get()).unwrap().0 = Color::GREEN.into();
            } else {
                borders.get_mut(parent.get()).unwrap().0 = Color::RED.into();
            }
        } else {
            borders.get_mut(parent.get()).unwrap().0 = Color::NONE.into();
        }
    }
}

pub fn update_hand_ui(
    hands: Query<(&Hand, &GamePosition), Or<(Changed<Hand>, Changed<GamePosition>)>>,
    card_sprites: Res<CardSpriteSheet>,
    card_info: Query<&CardInfo>,
    base_card_info: Query<&BaseCardInfo>,
    mut card_uis: Query<(&CardUISlot, &mut BackgroundColor, &mut Transform, &mut Handle<TextureAtlas>, &mut UiTextureAtlasImage)>,
) {
    if hands.is_empty() { return; }
    let (hand, position) = hands.get_single().expect("There should only be one player hand");
    for (slot, mut background, mut transform, mut atlas, mut image) in card_uis.iter_mut() {
        match hand.0[slot.0] {
            Some(card_instance_id) => {
                let base_card_id = base_card_info.get(card_instance_id).expect("Card without base card info").0;
                let card_info = card_info.get(base_card_id).expect("Card without info");
                background.0 = Color::WHITE.into();
                *atlas = card_sprites.0.clone();
                image.index = card_info.texture_index;
                transform.rotation = match position.d {
                    GameDirection::Up => Quat::from_rotation_z(0.0),
                    GameDirection::Down => Quat::from_rotation_z(std::f32::consts::PI),
                    GameDirection::Left => Quat::from_rotation_z(std::f32::consts::PI * 1.5),
                    GameDirection::Right => Quat::from_rotation_z(std::f32::consts::PI * 0.5),
                }
            },
            None => {
                background.0 = Color::PINK.into();
                *atlas = Handle::<TextureAtlas>::default();
                image.index = 0;
            }
        }
    }
}