use super::*;

#[derive(Clone)]
pub struct Level {
    pub flames: Vec<(i32, i32)>,
    pub items: Vec<(i32, i32, Item)>,
    pub map_size: (i32, i32),
    pub deck_list: Vec<usize>,
}

impl Level {
    fn new() -> Self {
        Self {
            flames: Vec::new(),
            items: Vec::new(),
            map_size: (1, 1),
            deck_list: Vec::new(),
        }
    }
    fn with_flames(&self, flames: Vec<(i32, i32)>) -> Self {
        let mut new_flames = self.flames.clone();
        new_flames.extend(flames);
        Self {
            flames: new_flames,
            ..self.clone()
        }
    }
    fn with_items(&self, items: Vec<(i32, i32, Item)>) -> Self {
        let mut new_items = self.items.clone();
        new_items.extend(items);
        Self {
            items: new_items,
            ..self.clone()
        }
    }
    fn with_size(&self , size: (i32, i32)) -> Self {
        Self {
            map_size: size,
            ..self.clone()
        }
    }
    fn with_deck(&self, deck: Vec<usize>) -> Self {
        Self {
            deck_list: deck,
            ..self.clone()
        }
    }
}

pub const NUM_PUZZLES: usize = 5;

pub fn get_puzzle(index: usize) -> Level {
    match index {
        0 => {
            Level::new()
                .with_size((1, 3))
                .with_flames(vec![(1, 3)])
                .with_deck(vec![1,])
        },
        1 => {
            Level::new()
                .with_size((2, 2))
                .with_flames(vec![(2, 2)])
                .with_deck(vec![2, 4])
        },
        2 => {
            Level::new()
                .with_size((3, 3))
                .with_flames(vec![(3, 3)])
                .with_deck(vec![1, 1, 3])
        },
        3 => {
            Level::new()
                .with_size((1, 3))
                .with_flames(vec![(1, 3)])
                .with_items(vec![(1, 2, Item::Water)])
                .with_deck(vec![1, 5])
        },
        4 => {
            Level::new()
                .with_size((4, 4))
                .with_flames(vec![(4, 4)])
                .with_items(vec![(2, 2, Item::Card(ContentID(13)))])
                .with_deck(vec![1, 1, 3])
        },

        _ => Level::new()
    }
}

pub const SPAWN_POINTS: [[Option<(i32, i32)>; 5]; 5] = [
    [Some((1, 3)), None, None, None, None],
    [Some((2, 2)), None, None, None, None],
    [Some((3, 3)), None, None, None, None],
    [Some((4, 1)), Some((3, 3)), None, None, None],
    [Some((2, 2)), Some((3, 2)), Some((3, 1)), None, None],
];
pub const MAP_SIZES: [(i32, i32); 5] = [(1, 3), (3, 3), (3, 3), (5, 5), (6, 6)];
pub const DECK_LISTS: [[Option<usize>; 16]; 5] = [
    [
        Some(1),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(1),
        Some(5),
        Some(3),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(1),
        Some(5),
        Some(3),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(1),
        Some(5),
        Some(3),
        Some(4),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
    [
        Some(1),
        Some(5),
        Some(3),
        Some(4),
        Some(8),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
];