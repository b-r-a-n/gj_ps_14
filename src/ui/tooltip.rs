use super::*;

#[derive(Component)]
pub struct Hovered(pub f32, pub bool);

#[derive(Component)]
pub struct Tooltip {
    pub text: String,
    pub threshold: f32,
}

#[derive(Component)]
pub struct TooltipContainer;

pub fn spawn_tooltip_container(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(8.0)),
                    max_width: Val::Vw(20.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                z_index: ZIndex::Global(1),
                border_color: Color::WHITE.into(),
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.95).into(),
                ..default()
            },
            TooltipContainer,
        ))
        .with_children(|container| {
            container.spawn((TextBundle::from_section(
                "",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),));
        });
}

pub fn update_hovered(
    mut commands: Commands,
    time: Res<Time>,
    interactions: Query<(Entity, &Interaction), Changed<Interaction>>,
) {
    for (entity, interaction) in interactions.iter() {
        match interaction {
            Interaction::Hovered => {
                commands
                    .entity(entity)
                    .insert(Hovered(time.elapsed_seconds(), false));
            }
            Interaction::None => {
                commands.entity(entity).remove::<Hovered>();
            }
            _ => {}
        }
    }
}

pub fn handle_hover_removed(
    mut removed: RemovedComponents<Hovered>,
    mut tooltip: Query<&mut Style, With<TooltipContainer>>,
) {
    for entity in removed.read() {
        info!("Hover removed from {:?}", entity);
        tooltip
            .get_single_mut()
            .expect("Should be only one tooltip container")
            .display = Display::None;
    }
}

pub fn trigger_tooltip(
    mut hovered: Query<(Entity, &mut Hovered, &Node, &Tooltip, &GlobalTransform)>,
    mut texts: Query<&mut Text>,
    mut tooltip_containers: Query<(&Children, &mut Style), With<TooltipContainer>>,
    time: Res<Time>,
) {
    for (entity, mut hovered, node, tooltip, transform) in hovered.iter_mut() {
        if !hovered.1 && time.elapsed_seconds() - hovered.0 > tooltip.threshold {
            info!(
                "Hovered {:?} for {:?} seconds",
                entity,
                time.elapsed_seconds() - hovered.0
            );
            hovered.1 = true;
            let (children, mut container) = tooltip_containers
                .get_single_mut()
                .expect("Should be only one tooltip container");
            container.display = Display::Flex;
            info!("Tooltip at: {:?}", transform.translation());
            let (width, height) = node.size().into();
            container.position_type = PositionType::Absolute;
            container.top = Val::Px(transform.translation().y - height / 2.0 - 24.0);
            container.left = Val::Px(transform.translation().x - width / 2.0);

            let child = children[0];
            texts
                .get_mut(child)
                .expect("Tooltip container should have a text node")
                .sections[0]
                .value = tooltip.text.clone();
        }
    }
}

pub fn add_tooltips_to_cards(
    mut commands: Commands,
    card_info: Res<CardInfoMap>,
    content_ids: Query<&ContentID>,
    card_uis: Query<(Entity, &CardInstance), (With<Interaction>, Without<Tooltip>)>,
) {
    for (card_ui_id, card_instance) in card_uis.iter() {
        if let Some(card_instance_id) = card_instance.0 {
            if let Ok(content_id) = content_ids.get(card_instance_id) {
                if let Some(card_info) = card_info.0.get(content_id) {
                    commands.entity(card_ui_id).insert(Tooltip {
                        text: card_info.description.clone(),
                        threshold: 1.0,
                    });
                }
            }
        }
    }
}
