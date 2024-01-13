use bevy::prelude::*;
use ui::{UIPlugins, energy::*, hand::*};

mod ui;

#[derive(Component, Clone)]
struct Energy {
    current: i32,
    maxium: i32,
}

#[derive(Clone, Default)]
enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Component, Default)]
struct Position {
    x: i32,
    y: i32,
    d: Direction,
}

#[derive(Default)]
struct SpawnPlayer {
    x: i32,
    y: i32,
    facing_direction: Direction,
    max_energy: i32,
}

#[derive(Resource)]
struct PlayerSpriteSheet(Handle<TextureAtlas>);

impl FromWorld for PlayerSpriteSheet {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource_mut::<AssetServer>()
            .expect("Failed get the `AssetServer` resource from the `World`");
        let texture_handle = asset_server.load("player.png");
        let texture_atlas = TextureAtlas::from_grid(
            texture_handle, 
            Vec2::new(64.0, 64.0), 
            1, 
            1, 
            None, 
            None
        );
        let mut texture_atlases = world.get_resource_mut::<Assets<TextureAtlas>>()
            .expect("Failed get the `Assets<TextureAtlas>` resource from the `World`");
        let texture_atlas_handle = texture_atlases.add(texture_atlas);
        Self(texture_atlas_handle)
    }
}

impl bevy::ecs::system::Command for SpawnPlayer {
    fn apply(self, world: &mut World) {
        let sprite_sheet = world.get_resource::<PlayerSpriteSheet>()
            .expect("Failed get the `PlayerSpriteSheet` resource from the `World`");
        world.spawn((
            Position {
                x: self.x,
                y: self.y,
                d: self.facing_direction,
            },
            Energy {
                current: self.max_energy/2,
                maxium: self.max_energy,
            },
            SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: sprite_sheet.0.clone(),
                ..default()
            },
            Player,
            Deck {
                cards: vec![],
                recycled: vec![],
                discarded: vec![],
            },
            Hand([None; 5]),
        ));
    }
}

#[derive(Default)]
struct SpawnCamera;

impl bevy::ecs::system::Command for SpawnCamera {
    fn apply(self, world: &mut World) {
        world.spawn(Camera2dBundle::default());
    }
}

fn update_position_transforms(
    mut query: Query<(&Position, &mut Transform)>
) {
    for (position, mut transform) in query.iter_mut() {
        transform.translation.x = position.x as f32 * 64.0;
        transform.translation.y = position.y as f32 * 64.0;
        transform.rotation = match position.d {
            Direction::Up => Quat::from_rotation_z(0.0),
            Direction::Down => Quat::from_rotation_z(std::f32::consts::PI),
            Direction::Left => Quat::from_rotation_z(std::f32::consts::PI * 0.5),
            Direction::Right => Quat::from_rotation_z(std::f32::consts::PI * 1.5),
        };
    }
}

#[derive(Component)]
struct ChangeAction<T: Component + Clone> {
    entity: Entity,
    updated_value: T,
}

#[derive(Component)]
struct Player;

fn apply_change_actions<T: Component + Clone>(
    mut commands: Commands,
    actions: Query<&ChangeAction<T>>, 
) {
    for action in actions.iter() {
        commands.entity(action.entity)
            .remove::<ChangeAction<T>>()
            .insert(action.updated_value.clone());
    }
}

#[derive(Clone, Debug)]
struct DrawAction {
    deck: Entity,
    hand: Entity,
}

#[derive(Clone, Debug)]
struct RecycleAction {
    card: Entity,
    hand: Entity,
}

#[derive(Clone, Debug)]
struct DiscardAction {
    card: Entity,
    deck: Entity,
    hand: Entity,
}

#[derive(Clone, Debug)]
struct PlayAction {
    card: Entity,
    deck: Entity,
    hand: Entity,
}

#[derive(Component)]
struct Card;

#[derive(Component)]
struct CardState {
    hand: Option<Entity>,
    deck: Option<Entity>,
}

#[derive(Component)]
struct Hand([Option<Entity>; 5]);

impl Hand {
    fn add(&mut self, card: Entity) {
        for slot in self.0.iter_mut() {
            if slot.is_none() {
                *slot = Some(card);
                return;
            }
        }
    }
    fn remove(&mut self, card: Entity) {
        for slot in self.0.iter_mut() {
            if *slot == Some(card) {
                *slot = None;
                return;
            }
        }
    }
}

fn sync_hand(
    mut commands: Commands,
    hands: Query<(Entity, &Hand), Changed<Hand>>,
    states: Query<&CardState>,
) {
    // TODO: This doesn't work for removed cards
    for (entity, hand) in hands.iter() {
        for card in hand.0.iter() {
            if let Some(card) = card {
                let state = states.get(*card)
                    .expect("Card in hand without state");
                commands.entity(*card)
                    .insert(CardState {
                        hand: Some(entity),
                        ..*state
                    });
            }
        }
    }
}

fn sync_deck(
    mut commands: Commands,
    decks: Query<(Entity, &Deck), Changed<Deck>>,
    states: Query<&CardState>,
) {
    for (entity, deck) in decks.iter() {
        for card in deck.cards.iter() {
            let state = states.get(*card).unwrap_or(&CardState {
                hand: None,
                deck: None,
            });
            commands.entity(*card)
                .insert(CardState {
                    deck: Some(entity),
                    ..*state
                });
        }
    }
}

#[derive(Clone, Component, Debug)]
enum CardAction {
    Draw(DrawAction),
    Recycle(RecycleAction),
    Discard(DiscardAction),
    Play(PlayAction),
}

#[derive(Debug, Event)]
enum CardEvent {
    WillDraw(Entity, DrawAction),
    DidDraw(Entity, DrawAction),
    WillRecycle(Entity, RecycleAction),
    DidRecycle(Entity, RecycleAction),
    WillDiscard(Entity, DiscardAction),
    DidDiscard(Entity, DiscardAction),
    WillPlay(Entity, PlayAction),
    DidPlay(Entity, PlayAction),
}

impl CardAction {
    fn to_will_event(&self, entity: Entity) -> CardEvent {
        match self {
            Self::Draw(action) => CardEvent::WillDraw(entity, action.clone()),
            Self::Recycle(action) => CardEvent::WillRecycle(entity, action.clone()),
            Self::Discard(action) => CardEvent::WillDiscard(entity, action.clone()),
            Self::Play(action) => CardEvent::WillPlay(entity, action.clone()),
        }
    }
    fn to_did_event(&self, entity: Entity) -> CardEvent {
        match self {
            Self::Draw(action) => CardEvent::DidDraw(entity, action.clone()),
            Self::Recycle(action) => CardEvent::DidRecycle(entity, action.clone()),
            Self::Discard(action) => CardEvent::DidDiscard(entity, action.clone()),
            Self::Play(action) => CardEvent::DidPlay(entity, action.clone()),
        }
    }
    fn from_event(event: &CardEvent) -> Self {
        match event {
            CardEvent::WillDraw(_, action) => Self::Draw(action.clone()),
            CardEvent::DidDraw(_, action) => Self::Draw(action.clone()),
            CardEvent::WillRecycle(_, action) => Self::Recycle(action.clone()),
            CardEvent::DidRecycle(_, action) => Self::Recycle(action.clone()),
            CardEvent::WillDiscard(_, action) => Self::Discard(action.clone()),
            CardEvent::DidDiscard(_, action) => Self::Discard(action.clone()),
            CardEvent::WillPlay(_, action) => Self::Play(action.clone()),
            CardEvent::DidPlay(_, action) => Self::Play(action.clone()),
        }
    }
}

impl CardEvent {
    fn get_entity(&self) -> Entity {
        match self {
            CardEvent::WillDraw(entity, _) => *entity,
            CardEvent::DidDraw(entity, _) => *entity,
            CardEvent::WillRecycle(entity, _) => *entity,
            CardEvent::DidRecycle(entity, _) => *entity,
            CardEvent::WillDiscard(entity, _) => *entity,
            CardEvent::DidDiscard(entity, _) => *entity,
            CardEvent::WillPlay(entity, _) => *entity,
            CardEvent::DidPlay(entity, _) => *entity,
        }
    }
}

#[derive(Component)]
enum ActionState {
    Announced,
    Applied,
}

fn handle_card_events(
    frame: Res<bevy::core::FrameCount>,
    mut commands: Commands,
    mut events: EventReader<CardEvent>,
) {

    let mut cleanup_ent = None;
    for event in events.read() {
        println!("[{:?}] Event Read - {:?}", frame.0, event);
        match event {
            CardEvent::DidDiscard(entity, _) => {
                cleanup_ent = Some(*entity);
            },
            CardEvent::DidDraw(entity, _) => {
                cleanup_ent = Some(*entity);
            },
            CardEvent::DidRecycle(entity, _) => {
                cleanup_ent = Some(*entity);
            },
            CardEvent::DidPlay(entity, _) => {
                cleanup_ent = Some(*entity);
            },
            _ => {},
        }
    }
    if let Some(entity) = cleanup_ent {
        // TODO: Maybe despawn?
        commands.entity(entity)
            .remove::<ActionState>()
            .remove::<CardAction>();
    }
}

fn announce_card_actions (
    frame: Res<bevy::core::FrameCount>,
    mut commands: Commands,
    actions: Query<(Entity, &CardAction), Without<ActionState>>,
    mut events: EventWriter<CardEvent>
) {
    for (entity, action) in actions.iter() {
        println!("[{:?}] Announcing - {:?}", frame.0, action);
        events.send(action.to_will_event(entity));
        commands.entity(entity).insert(ActionState::Announced);
    }
}

fn apply_card_actions (
    frame: Res<bevy::core::FrameCount>,
    mut commands: Commands,
    actions: Query<(Entity, &CardAction, &ActionState)>,
    mut decks: Query<&mut Deck>,
    mut hands: Query<&mut Hand>,
    mut events: EventWriter<CardEvent>,
    cards: Query<&CardState>,
) {
    for (entity, action, state) in actions.iter() {
        if match (action, state) {
            (CardAction::Draw(action), ActionState::Announced) => {
                let mut deck = decks.get_mut(action.deck)
                    .expect("Failed to get the deck");
                let mut hand = hands.get_mut(action.hand)
                    .expect("Failed to get the hand");
                let card = deck.draw()
                    .expect("Failed to draw a card");
                hand.add(card);
                true
            },
            (CardAction::Recycle(action), ActionState::Announced) => {
                let state = cards.get(action.card)
                    .expect("Failed to get the card state");
                if let Some(hand) = state.hand {
                    let mut hand = hands.get_mut(hand)
                        .expect("Failed to get the hand");
                    hand.remove(action.card);
                }
                if let Some(deck) = state.deck {
                    let mut deck = decks.get_mut(deck)
                        .expect("Failed to get the deck");
                    deck.recycle(action.card);
                }
                true
            },
            (CardAction::Discard(action), ActionState::Announced) => {
                let mut hand = hands.get_mut(action.hand)
                    .expect("Failed to get the hand");
                let mut deck = decks.get_mut(action.deck)
                    .expect("Failed to get the deck");
                hand.remove(action.card);
                deck.discard(action.card);
                true
            },
            (CardAction::Play(action), ActionState::Announced) => {
                let mut hand = hands.get_mut(action.hand)
                    .expect("Failed to get the hand");
                let mut deck = decks.get_mut(action.deck)
                    .expect("Failed to get the deck");
                hand.remove(action.card);
                deck.recycle(action.card);
                true
            },
            _ => false,
        } {
            println!("[{:?}] Completed - {:?}", frame.0, action);
            commands.entity(entity).insert(ActionState::Applied);
            events.send(action.to_did_event(entity));
        }
    }
}


fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    player_state: Query<(Entity, &Position, &Energy, &Hand), With<Player>>,
    mut decks: Query<&mut Deck>,
) {
    if keyboard_input.get_just_released().last().is_none() {
        return;
    }
    let (entity, position, energy, hand) = player_state.get_single()
        .expect("Found more than one player position");

    let energy_change = ChangeAction {
        entity,
        updated_value: Energy {
            current: energy.current - 1,
            ..energy.clone()
        },
    };

    match keyboard_input.get_just_released().last() {
        Some(KeyCode::Return) => {
            decks.get_mut(entity)
                .expect("Failed to get the deck")
                .add(commands.spawn(Card).id());
        }
        Some(KeyCode::Space) => {
            commands.spawn(CardAction::Draw(DrawAction {
                deck: entity,
                hand: entity,
            }));
        }

        Some(x) if x < &KeyCode::Key6 => {
            let index = x.clone() as usize - KeyCode::Key1 as usize;
            if let Some(card) = hand.0[index] {
                commands.spawn(CardAction::Play(PlayAction {
                    card,
                    deck: entity,
                    hand: entity,
                }));
            }
        }
        Some(KeyCode::Up) => {
            commands.spawn((
                ChangeAction {
                    entity,
                    updated_value: Position {
                        y: position.y + 1,
                        d: Direction::Up,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        Some(KeyCode::Down) => {
            commands.spawn((
                ChangeAction {
                    entity,
                    updated_value: Position {
                        y: position.y - 1,
                        d: Direction::Down,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        Some(KeyCode::Left) => {
            commands.spawn((
                ChangeAction {
                    entity,
                    updated_value: Position {
                        x: position.x - 1,
                        d: Direction::Left,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        Some(KeyCode::Right) => {
            commands.spawn((
                ChangeAction {
                    entity,
                    updated_value: Position {
                        x: position.x + 1,
                        d: Direction::Right,
                        ..position.clone()
                    },
                },
                energy_change,
            ));
        },
        _ => {},
    }
}

#[derive(Component)]
struct Deck {
    cards: Vec<Entity>,
    recycled: Vec<Entity>,
    discarded: Vec<Entity>,
}

impl Deck {
    fn add(&mut self, card: Entity) {
        self.cards.push(card);
    }
    fn draw(&mut self) -> Option<Entity> {
        self.cards.pop()
    }
    fn recycle(&mut self, card: Entity) {
        self.cards.retain(|&c| c != card);
        self.recycled.push(card);
    }
    fn discard(&mut self, card: Entity) {
        self.cards.retain(|&c| c != card);
        self.recycled.retain(|&c| c != card);
        self.discarded.push(card);
    }
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UIPlugins)
        .init_resource::<PlayerSpriteSheet>()
        .add_event::<CardEvent>()
        .add_systems(Startup, |mut commands: Commands| commands.add(SpawnCamera::default()))
        .add_systems(Startup, |mut commands: Commands| commands.add(SpawnPlayer { max_energy: 100, ..default() }))
        .add_systems(Update, handle_input)
        .add_systems(Update, (apply_change_actions::<Position>, apply_change_actions::<Energy>))
        .add_systems(Update, (handle_card_events, apply_card_actions, announce_card_actions))
        .add_systems(Update, (sync_hand, sync_deck))
        .add_systems(PostUpdate, update_position_transforms.before(bevy::transform::TransformSystem::TransformPropagate))
        .run();
}
