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