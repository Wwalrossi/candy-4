// game/types.rs

use serde::Serialize;
use super::constants::{WIDTH, HEIGHT};

/// Тип для игрового поля — двумерный массив из тайлов
pub type Board = [[u8; WIDTH]; HEIGHT];

/// Информация о перемещении тайла (анимация падения)
#[derive(Serialize, Clone)]
pub struct DropInfo {
    pub from_x: usize,
    pub from_y: usize,
    pub to_x: usize,
    pub to_y: usize,
    pub value: u8,
}

/// Состояние игры, отправляемое в JS
#[derive(Serialize, Clone)]
pub struct GameState {
    pub board: Board,
    pub score: u32,
    pub drops: Vec<DropInfo>,
}
