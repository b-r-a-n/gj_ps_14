use super::*;

#[derive(Component, Clone)]
pub struct Energy {
    pub current: i32,
    pub maxium: i32,
}

#[derive(Clone, Default)]
pub enum GameDirection {
    #[default]
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Component, Default)]
pub struct GamePosition {
    pub x: i32,
    pub y: i32,
    pub d: GameDirection,
}

impl GamePosition {
    pub fn offset(&self, by: i32) -> Self {
        match self.d {
            GameDirection::Up => Self { y: self.y + by, ..self.clone() },
            GameDirection::Down => Self { y: self.y - by, ..self.clone() },
            GameDirection::Left => Self { x: self.x - by, ..self.clone() },
            GameDirection::Right => Self { x: self.x + by, ..self.clone() },
        }
    }
}