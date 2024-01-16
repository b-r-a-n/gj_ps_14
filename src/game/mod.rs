use super::*;
pub use cards::*;
pub use actions::*;
pub use player::*;
pub use stats::*;

mod actions;
mod cards;
mod player;
mod stats;


#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Paused,
    Playing,
}

pub fn next_game_state(
    current: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {

    match current.get() {
        GameState::Loading => next.set(GameState::Playing),
        GameState::Paused => next.set(GameState::Playing),
        GameState::Playing => next.set(GameState::Paused),
    }
}

pub fn add_cards_to_deck(
    mut commands: Commands,
    mut deck: Query<&mut Deck, With<Player>>,
    card_infos: Query<Entity, With<CardInfo>>,
) {
    let mut deck = deck.get_single_mut().expect("Should be exactly 1 deck");
    let card_info_id = card_infos.iter().next().expect("Should be at least 1 card info");
    (0..40).for_each(|_| {
        deck.add(commands.spawn(BaseCardInfo(card_info_id)).id());
    });
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerSpriteSheet>()
            .add_state::<GameState>()
            .add_event::<CardEvent>()
            .add_systems(Startup, (spawn_player, load_card_infos, make_grid, next_game_state))
            .add_systems(OnTransition { from: GameState::Loading, to: GameState::Playing }, add_cards_to_deck)
            .add_systems(Update, (
                apply_change::<GamePosition>, 
                apply_change::<Energy>, 
                apply_change::<Water>, 
                apply_card))
            .add_systems(Update, (handle_card_events, apply_card_actions, announce_card_actions))
            .add_systems(Update, (sync_deck, sync_hand, mark_playable, realize_card_instances));
    }
}
