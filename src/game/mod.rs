use std::future::pending;

use super::*;
pub use actions::*;
pub use cards::*;
pub use game::*;
pub use player::*;
pub use stats::*;

mod actions;
mod cards;
mod game;
mod player;
mod stats;

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

pub fn check_for_turn_ready(
    mut commands: Commands,
    pending_actions: Query<(Entity, &CardActionType)>,
    hand: Query<&Hand, With<Player>>,
    deck: Query<&Deck, With<Player>>,
    mut pending_action_count: Local<usize>,
    mut next_turn_state: ResMut<NextState<TurnState>>,
) {
    if pending_actions.iter().len() != *pending_action_count {
        info!("Checking for turn ready. {} actions remain", pending_actions.iter().len());
        *pending_action_count = pending_actions.iter().len();
        for (e, t) in pending_actions.iter() {
            info!("{:?} | {:?}", t, e);
        }
    }

    if pending_actions.iter().len() == 0 {
        info!("Turn is ready");
        next_turn_state.set(TurnState::Started);
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PlayerSpriteSheet>()
            .add_state::<GameState>()
            .add_state::<TurnState>()
            .add_systems(Startup, (
                load_card_infos, 
                make_grid
            ))
            .add_systems(OnTransition { from: GameState::Menu, to: GameState::Loading }, (
                spawn_player, 
                schedule_transition::<NextGameState>
            ))
            .add_systems(OnTransition { from: GameState::Loading, to: GameState::Playing }, (
                add_cards_to_deck, 
                schedule_transition::<NextTurnState>
            ))
            .add_systems(OnEnter(TurnState::Starting), (
                fill_hand_with_cards, 
                restore_resources
                )
                .run_if(in_state(GameState::Playing)
            ))
            .add_systems(Update, (
                check_for_turn_ready
                )
                .run_if(in_state(GameState::Playing))
                .run_if(in_state(TurnState::Starting)))
            .add_systems(OnEnter(TurnState::Started), (
                update_playable, 
                schedule_transition::<NextTurnState>
                ).run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(TurnState::WaitingForInput), (
                log_playable,
            ))
            .add_systems(OnEnter(TurnState::Animating), (
                animate_cards,
            ))
            .add_systems(Update, (
                check_for_turn_over
                )
                .run_if(in_state(TurnState::WaitingForInput))
                .run_if(in_state(GameState::Playing))
            )
            .add_systems(Update, (
                transition::<GameState, NextGameState>,
                transition::<TurnState, NextTurnState>
            ))
            .add_systems(Update, (
                apply_change::<GamePosition>, 
                apply_change::<Energy>, 
                apply_change::<Water>, 
                apply_card,
                apply_card_actions, 
                sync_deck, 
                sync_hand, 
                log_playability_changes,
                realize_card_instances
                ).run_if(in_state(GameState::Playing)))
            ;
    }
}
