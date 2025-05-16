#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rand::Rng;
use serde::Serialize;
use tauri::command;
use std::collections::HashSet;

const WIDTH: usize = 8;
const HEIGHT: usize = 8;
const TILE_TYPES: usize = 5;

type Board = [[u8; WIDTH]; HEIGHT];

#[derive(Serialize, Clone)]
struct DropInfo {
    from_x: usize,
    from_y: usize,
    to_x: usize,
    to_y: usize,
    value: u8,
}

#[derive(Serialize, Clone)]
struct GameState {
    board: Board,
    score: u32,
    drops: Vec<DropInfo>,
}

#[command]
fn generate_board() -> GameState {
    let mut rng = rand::thread_rng();
    let mut board = [[0; WIDTH]; HEIGHT];

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            loop {
                let tile = rng.gen_range(1..=TILE_TYPES) as u8;
                if x >= 2 && board[y][x - 1] == tile && board[y][x - 2] == tile {
                    continue;
                }
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

#[command]
fn move_tile(
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    mut board: Board,
    score: u32,
) -> GameState {
    if !are_adjacent(x1, y1, x2, y2) {
        return GameState { board, score, drops: vec![] };
    }

    board[y1][x1] ^= board[y2][x2];
    board[y2][x2] ^= board[y1][x1];
    board[y1][x1] ^= board[y2][x2];

    let mut total_score = score;
    let mut had_matches = false;
    let mut total_drops = vec![];

    loop {
        let matches = find_matches(&board);
        if matches.is_empty() {
            break;
        }

        had_matches = true;

        let matched_count = matches.len();
        if matched_count >= 3 {
            total_score += 3 + (matched_count as u32 - 3);
        }

        for &(x, y) in &matches {
            board[y][x] = 0;
        }

        let drops = drop_tiles_with_info(&mut board);
        total_drops.extend(drops);
    }

    if !had_matches {
        board[y1][x1] ^= board[y2][x2];
        board[y2][x2] ^= board[y1][x1];
        board[y1][x1] ^= board[y2][x2];
        return GameState { board, score, drops: vec![] };
    }

    GameState {
        board,
        score: total_score,
        drops: total_drops,
    }
}

fn are_adjacent(x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
    (x1 == x2 && (y1 as isize - y2 as isize).abs() == 1)
        || (y1 == y2 && (x1 as isize - x2 as isize).abs() == 1)
}

fn find_matches(board: &Board) -> Vec<(usize, usize)> {
    let mut matches = HashSet::new();

    for y in 0..HEIGHT {
        let mut count = 1;
        for x in 1..WIDTH {
            if board[y][x] != 0 && board[y][x] == board[y][x - 1] {
                count += 1;
            } else {
                if count >= 3 {
                    for k in 0..count {
                        matches.insert((x - 1 - k, y));
                    }
                }
                count = 1;
            }
        }
        if count >= 3 {
            for k in 0..count {
                matches.insert((WIDTH - 1 - k, y));
            }
        }
    }

    for x in 0..WIDTH {
        let mut count = 1;
        for y in 1..HEIGHT {
            if board[y][x] != 0 && board[y][x] == board[y - 1][x] {
                count += 1;
            } else {
                if count >= 3 {
                    for k in 0..count {
                        matches.insert((x, y - 1 - k));
                    }
                }
                count = 1;
            }
        }
        if count >= 3 {
            for k in 0..count {
                matches.insert((x, HEIGHT - 1 - k));
            }
        }
    }

    matches.into_iter().collect()
}

fn drop_tiles_with_info(board: &mut Board) -> Vec<DropInfo> {
    let mut drops = vec![];

    for x in 0..WIDTH {
        let mut column: Vec<(usize, u8)> = vec![];

        for y in (0..HEIGHT).rev() {
            if board[y][x] != 0 {
                column.push((y, board[y][x]));
            }
        }

        let mut rng = rand::thread_rng();
        while column.len() < HEIGHT {
            column.push((999, rng.gen_range(1..=TILE_TYPES) as u8));
        }

        for (offset, &(from_y, value)) in column.iter().enumerate() {
            let to_y = HEIGHT - 1 - offset;
            board[to_y][x] = value;

            if from_y != to_y {
                drops.push(DropInfo {
                    from_x: x,
                    from_y,
                    to_x: x,
                    to_y,
                    value,
                });
            }
        }
    }

    drops
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![generate_board, move_tile])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
