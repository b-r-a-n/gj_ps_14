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
    pub fn offset(&self, by: i32) -> Self {
        let mut pos = match self.d {
            GameDirection::Up => Self { y: self.y + by, ..self.clone() },
            GameDirection::Down => Self { y: self.y - by, ..self.clone() },
            GameDirection::Left => Self { x: self.x - by, ..self.clone() },
            GameDirection::Right => Self { x: self.x + by, ..self.clone() },
        };
        if pos.y < 0 { pos.y = 0; }
        if pos.x < 0 { pos.x = 0; }
        pos
    }
    pub fn adjacent(&self, radius: i32) -> Vec<Self> {
        let mut positions = vec![];
        for i in -radius..=radius {
            for j in -radius..=radius {
                if i.abs() + j.abs() <= radius {
                    if self.x + i > 0 && self.y + j > 0 {
                        positions.push(Self { x: self.x + i, y: self.y + j, ..self.clone() });
                    }
                }
            }
        }
        positions
    }
    pub fn rotated(&self, rotation: &Rotation) -> Self {
        match (&self.d, rotation) {
                (GameDirection::Up, Rotation::Right) => Self { d: GameDirection::Right, ..self.clone() },
                (GameDirection::Down, Rotation::Right) => Self { d: GameDirection::Left, ..self.clone() },
                (GameDirection::Left, Rotation::Right) => Self { d: GameDirection::Up, ..self.clone() },
                (GameDirection::Right, Rotation::Right) => Self { d: GameDirection::Down, ..self.clone() },
                (GameDirection::Up, Rotation::Left) => Self { d: GameDirection::Left, ..self.clone() },
                (GameDirection::Down, Rotation::Left) => Self { d: GameDirection::Right, ..self.clone() },
                (GameDirection::Left, Rotation::Left) => Self { d: GameDirection::Down, ..self.clone() },
                (GameDirection::Right, Rotation::Left) => Self { d: GameDirection::Up, ..self.clone() },
                (_, Rotation::None) => self.clone(),
        }
    }
}