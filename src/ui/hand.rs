use bevy::ui::RelativeCursorPosition;

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

#[derive(Component)]
pub struct CardTitle;

#[derive(Component)]
pub struct EnergyText;

#[derive(Component)]
pub struct WaterText;

#[derive(Event)]
pub struct CardClicked {
    pub card_instance: CardInstance,
}

#[derive(Event)]
pub struct EndTurnClicked;

pub(in crate::ui) const CARD_WIDTH: f32 = 140.0;
pub(in crate::ui) const CARD_HEIGHT: f32 = 200.0;

pub fn handle_click(
    button_input: Res<Input<MouseButton>>,
    cursor_positions: Query<(
        Entity,
        &RelativeCursorPosition,
        Option<&CardInstance>,
        Option<&Button>,
    )>,
    mut down_on_entity: Local<Option<Entity>>,
    mut card_clicked: EventWriter<CardClicked>,
    mut end_turn_clicked: EventWriter<EndTurnClicked>,
) {
    if button_input.just_pressed(MouseButton::Left) {
        for (entity, cursor_position, _, _) in cursor_positions.iter() {
            if cursor_position.mouse_over() {
                *down_on_entity = Some(entity);
            }
        }
    }
    if button_input.just_released(MouseButton::Left) {
        for (entity, cursor_position, card, button) in cursor_positions.iter() {
            if cursor_position.mouse_over() {
                if down_on_entity.as_ref().is_some() && entity == *down_on_entity.as_ref().unwrap()
                {
                    if let Some(card_instance) = card {
                        card_clicked.send(CardClicked {
                            card_instance: card_instance.clone(),
                        });
                    }
                    if let Some(_) = button {
                        end_turn_clicked.send(EndTurnClicked);
                    }
                }
            }
        }
        *down_on_entity = None;
    }
}

#[derive(Clone, Component)]
pub struct CardInstance(pub Option<Entity>);

impl bevy::ecs::system::Command for SpawnHandUI {
    fn apply(self, world: &mut World) {
        let bottom_container_id = world
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                HandUI,
            ))
            .id();
        let hand_id = world
            .spawn((NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::End,
                    border: UiRect::right(Val::Px(8.0)),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },))
            .id();
        let icon_atlas = world.get_resource::<IconSpriteSheet>().unwrap().0.clone();
        let deck_dock = world
            .spawn((NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Baseline,
                    justify_content: JustifyContent::SpaceEvenly,
                    width: Val::Px(112.0),
                    height: Val::Px(CARD_HEIGHT),
                    margin: UiRect {
                        left: Val::Px(8.0),
                        bottom: Val::Px(8.0),
                        top: Val::Px(8.0),
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },))
            .with_children(|parent| {
                (2..=4).for_each(|i| {
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            Interaction::default(),
                            Tooltip {
                                text: match i {
                                    2 => "Number of cards in Deck".to_string(),
                                    3 => "Number of cards that will be Recycled".to_string(),
                                    4 => "Number oc cards that have been Discarded".to_string(),
                                    _ => panic!("Invalid icon index"),
                                },
                                threshold: 0.0,
                            },
                        ))
                        .with_children(|icon_container| {
                            icon_container.spawn((AtlasImageBundle {
                                style: Style {
                                    width: Val::Px(64.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                                texture_atlas: icon_atlas.clone(),
                                texture_atlas_image: UiTextureAtlasImage {
                                    index: i,
                                    ..default()
                                },
                                ..default()
                            },));
                            let mut icon_text = icon_container.spawn((TextBundle::from_section(
                                "",
                                TextStyle {
                                    font_size: 32.0,
                                    ..default()
                                },
                            ),));
                            match i {
                                2 => {
                                    icon_text.insert(DeckUIText);
                                }
                                3 => {
                                    icon_text.insert(RecycledUIText);
                                }
                                4 => {
                                    icon_text.insert(DiscardedUIText);
                                }
                                _ => panic!("Invalid icon index"),
                            }
                        });
                })
            })
            .id();
        let cards: Vec<Entity> = (0..5)
            .map(|i| {
                world
                    .spawn((
                        NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::FlexStart,
                                row_gap: Val::Px(8.0),
                                width: Val::Px(CARD_WIDTH),
                                height: Val::Px(CARD_HEIGHT),
                                margin: UiRect {
                                    left: Val::Px(8.0),
                                    bottom: Val::Px(8.0),
                                    top: Val::Px(8.0),
                                    ..default()
                                },
                                border: UiRect::all(Val::Px(4.0)),
                                ..default()
                            },
                            background_color: Color::DARK_GRAY.into(),
                            ..default()
                        },
                        CardInstance(None),
                        CardUISlot(i),
                        Interaction::default(),
                        RelativeCursorPosition::default(),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font_size: 24.0,
                                    color: Color::WHITE,
                                    ..default()
                                },
                            ),
                            CardTitle,
                            CardUISlot(i),
                        ));
                        parent.spawn((
                            AtlasImageBundle {
                                style: Style {
                                    width: Val::Px(120.0),
                                    height: Val::Px(120.0),
                                    ..default()
                                },
                                ..default()
                            },
                            CardUISlot(i),
                        ));
                        parent
                            .spawn((NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    width: Val::Percent(90.0),
                                    ..default()
                                },
                                ..default()
                            },))
                            .with_children(|node| {
                                node.spawn((
                                    TextBundle::from_section(
                                        "",
                                        TextStyle {
                                            font_size: 24.0,
                                            color: Color::YELLOW,
                                            ..default()
                                        },
                                    ),
                                    EnergyText,
                                    CardUISlot(i),
                                ));
                                node.spawn((
                                    TextBundle::from_section(
                                        "",
                                        TextStyle {
                                            font_size: 24.0,
                                            color: Color::rgb(
                                                2.0 / 255.0,
                                                204.0 / 255.0,
                                                254.0 / 255.0,
                                            ),
                                            ..default()
                                        },
                                    ),
                                    WaterText,
                                    CardUISlot(i),
                                ));
                            });
                    })
                    .id()
            })
            .collect();
        let resource_dock = spawn_resource_ui(world);
        let mut hand = world.get_entity_mut(hand_id).unwrap();
        hand.set_parent(bottom_container_id);
        hand.push_children(&vec![deck_dock]);
        hand.push_children(&cards);
        hand.push_children(&vec![resource_dock]);
    }
}

pub fn update_playable_indicator(
    statuses: Query<&CardStatus>,
    hand: Query<&Hand>,
    mut card_uis: Query<(Entity, &CardUISlot, &CardInstance)>,
    end_turn_button: Query<Entity, (With<Button>, With<EndTurnButton>)>,
    mut borders: Query<&mut BorderColor>,
) {
    let mut button_id = None;
    if !end_turn_button.is_empty() {
        button_id = Some(
            end_turn_button
                .get_single()
                .expect("Should only be one end turn button"),
        );
    }
    if hand.is_empty() {
        if let Some(button_id) = button_id {
            borders.get_mut(button_id).unwrap().0 = Color::YELLOW_GREEN.into();
        }
        return;
    }
    let hand = hand
        .get_single()
        .expect("There should only be one player hand");

    let mut playable_count = 0;
    for (ui_id, slot, _) in card_uis.iter_mut() {
        if let Some(card_instance_id) = hand.0[slot.0] {
            let status = statuses.get(card_instance_id).expect("Card without status");
            if status.is_playable() {
                playable_count += 1;
                borders.get_mut(ui_id).unwrap().0 = Color::GREEN.into();
            } else {
                borders.get_mut(ui_id).unwrap().0 = Color::RED.into();
            }
        } else {
            borders.get_mut(ui_id).unwrap().0 = Color::NONE.into();
        }
    }
    if let Some(button_id) = button_id {
        if playable_count > 0 {
            borders.get_mut(button_id).unwrap().0 = Color::WHITE.into();
        } else {
            borders.get_mut(button_id).unwrap().0 = Color::YELLOW_GREEN.into();
        }
    }
}

pub fn update_deck_ui(
    changed_decks: Query<&Deck, Changed<Deck>>,
    mut deck_text: Query<&mut Text, With<DeckUIText>>,
) {
    if changed_decks.is_empty() {
        return;
    }
    let deck = changed_decks
        .get_single()
        .expect("There should only be one deck");
    deck_text
        .get_single_mut()
        .expect("There should only be one deck text")
        .sections[0]
        .value = format!("{}", deck.cards.len());
}

pub fn update_recycled_ui(
    changed_decks: Query<&Deck, Changed<Deck>>,
    mut recycled_text: Query<&mut Text, With<RecycledUIText>>,
) {
    if changed_decks.is_empty() {
        return;
    }
    let deck = changed_decks
        .get_single()
        .expect("There should only be one deck");
    recycled_text
        .get_single_mut()
        .expect("There should only be one deck text")
        .sections[0]
        .value = format!("{}", deck.recycled.len());
}

pub fn update_discarded_ui(
    changed_decks: Query<&Deck, Changed<Deck>>,
    mut discarded_text: Query<&mut Text, With<DiscardedUIText>>,
) {
    if changed_decks.is_empty() {
        return;
    }
    let deck = changed_decks
        .get_single()
        .expect("There should only be one deck");
    discarded_text
        .get_single_mut()
        .expect("There should only be one deck text")
        .sections[0]
        .value = format!("{}", deck.discarded.len());
}

pub fn update_hand_title_texts(
    hands: Query<&Hand, Or<(Changed<Hand>, Changed<GamePosition>)>>,
    card_info: Res<CardInfoMap>,
    base_card_info: Query<&ContentID>,
    mut card_texts: Query<(&CardUISlot, &mut Text), With<CardTitle>>,
) {
    if hands.is_empty() {
        return;
    }
    let hand = hands
        .get_single()
        .expect("There should only be one player hand");
    for (slot, mut text) in card_texts.iter_mut() {
        match hand.0[slot.0] {
            Some(card_instance_id) => {
                let base_card_id = base_card_info
                    .get(card_instance_id)
                    .expect("Card without base card info");
                let card_info = card_info.0.get(&*base_card_id).expect("Card without info");
                text.sections[0].value = card_info.name.clone();
            }
            None => {
                text.sections[0].value = "".to_string();
            }
        }
    }
}

pub fn update_hand_energy_texts(
    hands: Query<&Hand, Or<(Changed<Hand>, Changed<GamePosition>)>>,
    card_info: Res<CardInfoMap>,
    base_card_info: Query<&ContentID>,
    mut card_texts: Query<(&CardUISlot, &mut Text), With<EnergyText>>,
) {
    if hands.is_empty() {
        return;
    }
    let hand = hands
        .get_single()
        .expect("There should only be one player hand");
    for (slot, mut text) in card_texts.iter_mut() {
        match hand.0[slot.0] {
            Some(card_instance_id) => {
                let base_card_id = base_card_info
                    .get(card_instance_id)
                    .expect("Card without base card info");
                let card_info = card_info.0.get(&*base_card_id).expect("Card without info");
                let energy_cost = card_info.resource_cost.energy;
                if energy_cost > 0 {
                    text.sections[0].value = energy_cost.to_string();
                } else {
                    text.sections[0].value = "".to_string();
                }
            }
            None => {
                text.sections[0].value = "".to_string();
            }
        }
    }
}

pub fn update_hand_water_texts(
    hands: Query<&Hand, Or<(Changed<Hand>, Changed<GamePosition>)>>,
    card_info: Res<CardInfoMap>,
    base_card_info: Query<&ContentID>,
    mut card_texts: Query<(&CardUISlot, &mut Text), With<WaterText>>,
) {
    if hands.is_empty() {
        return;
    }
    let hand = hands
        .get_single()
        .expect("There should only be one player hand");
    for (slot, mut text) in card_texts.iter_mut() {
        match hand.0[slot.0] {
            Some(card_instance_id) => {
                let base_card_id = base_card_info
                    .get(card_instance_id)
                    .expect("Card without base card info");
                let card_info = card_info.0.get(&*base_card_id).expect("Card without info");
                let water_cost = card_info.resource_cost.water;
                if water_cost > 0 {
                    text.sections[0].value = water_cost.to_string();
                } else {
                    text.sections[0].value = "".to_string();
                }
            }
            None => {
                text.sections[0].value = "".to_string();
            }
        }
    }
}

pub fn update_interactions(
    hands: Query<&Hand, Changed<Hand>>,
    mut card_uis: Query<(&CardUISlot, &mut CardInstance)>,
) {
    if hands.is_empty() {
        return;
    }
    let hand = hands
        .get_single()
        .expect("There should only be one player hand");
    for (slot, mut card_instance) in card_uis.iter_mut() {
        card_instance.0 = hand.0[slot.0];
    }
}

pub fn update_hand_images(
    hands: Query<(&Hand, &GamePosition), Or<(Changed<Hand>, Changed<GamePosition>)>>,
    card_sprites: Res<CardSpriteSheet>,
    card_info: Res<CardInfoMap>,
    base_card_info: Query<&ContentID>,
    mut card_images: Query<(
        &CardUISlot,
        &mut BackgroundColor,
        &mut Transform,
        &mut Handle<TextureAtlas>,
        &mut UiTextureAtlasImage,
    )>,
) {
    if hands.is_empty() {
        return;
    }
    let (hand, position) = hands
        .get_single()
        .expect("There should only be one player hand");
    for (slot, mut background, mut transform, mut atlas, mut image) in card_images.iter_mut() {
        match hand.0[slot.0] {
            Some(card_instance_id) => {
                let base_card_id = base_card_info
                    .get(card_instance_id)
                    .expect("Card without base card info");
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
            }
            None => {
                background.0 = Color::PINK.into();
                *atlas = Handle::<TextureAtlas>::default();
                image.index = 0;
            }
        }
    }
}
