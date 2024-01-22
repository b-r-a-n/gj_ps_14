use super::*;

#[derive(Component, Clone)]
pub struct Energy {
    pub current: i32,
    pub maxium: i32,
}

#[derive(Component, Clone)]
pub struct Water {
    pub current: i32,
    pub maxium: i32,
}

#[derive(Clone, Debug, Default)]
pub enum GameDirection {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Component, Debug, Default)]
pub struct GamePosition {
    pub x: i32,
    pub y: i32,
    pub d: GameDirection,
}

impl GamePosition {
    pub fn offset(&self, by: (i32, i32)) -> Self {
        match self.d {
            GameDirection::Up => Self {
                y: self.y + by.0,
                x: self.x + by.1,
                ..self.clone()
            },
            GameDirection::Down => Self {
                y: self.y - by.0,
                x: self.x - by.1,
                ..self.clone()
            },
            GameDirection::Left => Self {
                x: self.x - by.0,
                y: self.y + by.1,
                ..self.clone()
            },
            GameDirection::Right => Self {
                x: self.x + by.0,
                y: self.y - by.1,
                ..self.clone()
            },
        }
    }
    pub fn rotated(&self, rotation: &Rotation) -> Self {
        match (&self.d, rotation) {
            (GameDirection::Up, Rotation::Right) => Self {
                d: GameDirection::Right,
                ..self.clone()
            },
            (GameDirection::Down, Rotation::Right) => Self {
                d: GameDirection::Left,
                ..self.clone()
            },
            (GameDirection::Left, Rotation::Right) => Self {
                d: GameDirection::Up,
                ..self.clone()
            },
            (GameDirection::Right, Rotation::Right) => Self {
                d: GameDirection::Down,
                ..self.clone()
            },
            (GameDirection::Up, Rotation::Left) => Self {
                d: GameDirection::Left,
                ..self.clone()
            },
            (GameDirection::Down, Rotation::Left) => Self {
                d: GameDirection::Right,
                ..self.clone()
            },
            (GameDirection::Left, Rotation::Left) => Self {
                d: GameDirection::Down,
                ..self.clone()
            },
            (GameDirection::Right, Rotation::Left) => Self {
                d: GameDirection::Up,
                ..self.clone()
            },
            (_, Rotation::None) => self.clone(),
        }
    }
}
