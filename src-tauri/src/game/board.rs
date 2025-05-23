// game/board.rs

use rand::Rng;
use super::constants::{WIDTH, HEIGHT, TILE_TYPES};
use super::types::{GameState};

use tauri::command;

/// Генерация начального игрового поля без начальных совпадений
#[command]
pub fn generate_board() -> GameState {
    let mut rng = rand::thread_rng();
    let mut board = [[0; WIDTH]; HEIGHT];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            loop {
                let tile = rng.gen_range(1..=TILE_TYPES) as u8;

                // Проверка на горизонтальные совпадения
                if x >= 2 && board[y][x - 1] == tile && board[y][x - 2] == tile {
                    continue;
                }

                // Проверка на вертикальные совпадения
                if y >= 2 && board[y - 1][x] == tile && board[y - 2][x] == tile {
                    continue;
                }

                board[y][x] = tile;
                break;
            }
        }
    }

    GameState {
        board,
        score: 0,
        drops: vec![],
    }
}
