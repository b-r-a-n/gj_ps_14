use super::*;

#[derive(Component)]
pub struct HandUI;

#[derive(Default)]
pub struct SpawnHandUI;

#[derive(Component)]
pub struct CardUISlot(usize);


#[derive(Component)]
pub struct DeckUIText;

#[derive(Component)]
pub struct RecycledUIText;

#[derive(Component)]
pub struct DiscardedUIText;

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
        let icon_atlas = world.get_resource::<IconSpriteSheet>().unwrap().0.clone();
        let dock = world.spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Baseline,
                    justify_content: JustifyContent::SpaceEvenly,
                    width: Val::Px(112.0),
                    height: Val::Px(CARD_HEIGHT),
                    margin: UiRect { left: Val::Px(8.0), bottom: Val::Px(8.0), top: Val::Px(8.0), ..default() },
                    ..default()
                },
                ..default()

            },
        )).with_children(|parent| {
            (2..=4).for_each(|i| {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                )).with_children(|icon_container| {
                    icon_container.spawn((
                        AtlasImageBundle {
                            style: Style {
                                width: Val::Px(64.0),
                                height: Val::Px(32.0),
                                ..default()
                            },
                            texture_atlas:icon_atlas.clone(), 
                            texture_atlas_image: UiTextureAtlasImage {index: i, ..default()},
                            ..default()
                        },
                    ));
                    let mut icon_text = icon_container.spawn((
                        TextBundle::from_section(
                            "", 
                            TextStyle { 
                                font_size: 32.0,
                                ..default() 
                            }
                        ),
                    ));
                    match i { 
                        2 => {icon_text.insert(DeckUIText); },
                        3 => {icon_text.insert(RecycledUIText); }, 
                        4 => {icon_text.insert(DiscardedUIText); }, 
                        _ => panic!("Invalid icon index") 
                    }
                });
            })
        })
        .id();
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
        hand.push_children(&vec![dock]);
        hand.push_children(&cards);
    }
}

pub fn update_playable_indicator(
    statuses: Query<&CardStatus>,
    hand: Query<&Hand>,
    mut card_uis: Query<(&Parent, &CardUISlot)>,
    mut borders: Query<&mut BorderColor>
) {

    if hand.is_empty() { return; }
    let hand = hand.get_single().expect("There should only be one player hand");

    for (parent, slot) in card_uis.iter_mut() {
        if let Some(card_instance_id) = hand.0[slot.0] {
            let status = statuses.get(card_instance_id).expect("Card without status");
            if status.is_playable() {
                borders.get_mut(parent.get()).unwrap().0 = Color::GREEN.into();
            } else {
                borders.get_mut(parent.get()).unwrap().0 = Color::RED.into();
            }
        } else {
            borders.get_mut(parent.get()).unwrap().0 = Color::NONE.into();
        }
    }
}

pub fn update_deck_ui(
    changed_decks: Query<&Deck, Changed<Deck>>,
    mut deck_text: Query<&mut Text, With<DeckUIText>>,
) {
    if changed_decks.is_empty() { return; }
    let deck = changed_decks.get_single().expect("There should only be one deck");
    deck_text.get_single_mut().expect("There should only be one deck text")
        .sections[0].value = format!("{}", deck.cards.len());
}

pub fn update_recycled_ui(
    changed_decks: Query<&Deck, Changed<Deck>>,
    mut recycled_text: Query<&mut Text, With<RecycledUIText>>,
) {
    if changed_decks.is_empty() { return; }
    let deck = changed_decks.get_single().expect("There should only be one deck");
    recycled_text.get_single_mut().expect("There should only be one deck text")
        .sections[0].value = format!("{}", deck.recycled.len());
}

pub fn update_discarded_ui(
    changed_decks: Query<&Deck, Changed<Deck>>,
    mut discarded_text: Query<&mut Text, With<DiscardedUIText>>,
) {
    if changed_decks.is_empty() { return; }
    let deck = changed_decks.get_single().expect("There should only be one deck");
    discarded_text.get_single_mut().expect("There should only be one deck text")
        .sections[0].value = format!("{}", deck.discarded.len());
}

pub fn update_hand_ui(
    hands: Query<(&Hand, &GamePosition), Or<(Changed<Hand>, Changed<GamePosition>)>>,
    card_sprites: Res<CardSpriteSheet>,
    card_info: Res<CardInfoMap>,
    base_card_info: Query<&ContentID>,
    mut card_uis: Query<(&CardUISlot, &mut BackgroundColor, &mut Transform, &mut Handle<TextureAtlas>, &mut UiTextureAtlasImage)>,
) {
    if hands.is_empty() { return; }
    let (hand, position) = hands.get_single().expect("There should only be one player hand");
    for (slot, mut background, mut transform, mut atlas, mut image) in card_uis.iter_mut() {
        match hand.0[slot.0] {
            Some(card_instance_id) => {
                let base_card_id = base_card_info.get(card_instance_id).expect("Card without base card info");
                let card_info = card_info.0.get(&*base_card_id).expect("Card without info");
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