use super::*;

#[derive(Component)]
pub struct Card {
    pub energy_cost: u32,
}

#[derive(Component)]
pub struct CardState {
    pub hand: Option<Entity>,
    pub deck: Option<Entity>,
}

#[derive(Component)]
pub struct Hand(pub [Option<Entity>; 5]);

impl Hand {
    pub fn add(&mut self, card: Entity) {
        for slot in self.0.iter_mut() {
            if slot.is_none() {
                *slot = Some(card);
                return;
            }
        }
    }
    pub fn remove(&mut self, card: Entity) {
        for slot in self.0.iter_mut() {
            if *slot == Some(card) {
                *slot = None;
                return;
            }
        }
    }
}

pub fn sync_hand(
    mut commands: Commands,
    hands: Query<(Entity, &Hand), Changed<Hand>>,
    states: Query<&CardState>,
) {
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

#[derive(Component)]
pub struct Deck {
    pub cards: Vec<Entity>,
    pub recycled: Vec<Entity>,
    pub discarded: Vec<Entity>,
}

impl Deck {
    pub fn add(&mut self, card: Entity) {
        self.cards.push(card);
    }
    pub fn draw(&mut self) -> Option<Entity> {
        self.cards.pop()
    }
    pub fn recycle(&mut self, card: Entity) {
        self.cards.retain(|&c| c != card);
        self.recycled.push(card);
    }
    pub fn discard(&mut self, card: Entity) {
        self.cards.retain(|&c| c != card);
        self.recycled.retain(|&c| c != card);
        self.discarded.push(card);
    }
}

pub fn sync_deck(
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