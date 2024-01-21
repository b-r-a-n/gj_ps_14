use super::*;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameState {
    #[default]
    None,
    Loading,
    Loaded,
    Playing,
}

#[derive(Component, Default)]
pub struct NextGameState;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum TurnState {
    #[default]
    None,
    Starting,
    Started,
    WaitingForInput,
    Animating,
    Ended,
}

#[derive(Component, Default)]
pub struct NextTurnState;

pub fn check_for_turn_over(
    current_turn_state: Res<State<TurnState>>,
    mut next_turn_state: ResMut<NextState<TurnState>>,
    playable_cards: Query<&CardStatus, With<InHand>>,
) {
    if playable_cards.iter().filter(|card| card.is_playable()).count() < 1 {
        info!("Turn is over. Press enter");
        next_turn_state.set(TurnState::WaitingForInput);
    } else {
        next_turn_state.set(current_turn_state.next())
    }
}

pub trait StateTransition {
    fn next(&self) -> Self;
}

impl StateTransition for GameState {
    fn next(&self) -> Self {
        match self {
            GameState::None => GameState::Loading,
            GameState::Loading => GameState::Loaded,
            GameState::Loaded => GameState::Playing,
            GameState::Playing => GameState::Loaded,
        }
    }
}

impl StateTransition for TurnState {
    fn next(&self) -> Self {
        match self {
            TurnState::None => TurnState::Starting,
            TurnState::Starting => TurnState::Started,
            TurnState::Started => TurnState::WaitingForInput,
            TurnState::WaitingForInput => TurnState::Ended,
            TurnState::Animating => TurnState::Started,
            TurnState::Ended => TurnState::Starting,
        }
    }
}

pub fn schedule_transition<U: Component + Default>(
    mut commands: Commands,
) {
    commands.spawn(U::default());
}

pub fn transition<T: States + StateTransition, U: Component>(
    mut commands: Commands,
    query: Query<Entity, With<U>>,
    current: Res<State<T>>,
    mut next: ResMut<NextState<T>>,
) {
    if query.is_empty() {
        return;
    }
    next.set(current.get().next());
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}