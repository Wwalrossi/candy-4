// game/logic.rs

use std::collections:: HashSet;
use rand::Rng;

use super::constants::*;
use super::types::*;
use super::utils::are_adjacent;

use tauri::command;

/// Обработка перемещения двух соседних тайлов
#[command]
pub fn move_tile(x1: usize, y1: usize, x2: usize, y2: usize, mut board: Board, score: u32) -> GameState {
    // Если тайлы не соседние — ход недопустим
    if !are_adjacent(x1, y1, x2, y2) {
        return GameState { board, score, drops: vec![] };
    }

    // Обмениваем тайлы местами
    board[y1][x1] ^= board[y2][x2];
    board[y2][x2] ^= board[y1][x1];
    board[y1][x1] ^= board[y2][x2];

    let mut total_score = score;
    let mut had_matches = false;
    let mut total_drops = vec![];
    let mut bonus_placed = false;

    // Основной цикл: обрабатываем совпадения, падения и бонусы до тех пор, пока они есть
    loop {
        let groups = find_matches(&board);
        if groups.is_empty() {
            break;
        }

        had_matches = true;

        // Собираем все совпавшие клетки в одно множество
        let all_matches: HashSet<_> = groups.iter().flatten().copied().collect();

        // Начисляем очки: 3 очка за тройку + по 1 за каждый дополнительный тайл
        if all_matches.len() >= 3 {
            total_score += 3 + (all_matches.len() as u32 - 3);
        }

        // Удаляем все совпавшие тайлы (всегда сначала, чтобы избежать конфликтов с бонусом)
        for &(x, y) in &all_matches {
            board[y][x] = 0;
        }

        // Один бонус за длинную линию (5+), ставим бонус ПОСЛЕ удаления
        if !bonus_placed {
            if let Some(group) = groups.iter().find(|g| g.len() >= 5) {
                let mid = group[group.len() / 2]; // середина группы
                board[mid.1][mid.0] = BONUS_TILE; // ставим бонус
                bonus_placed = true;
            }
        }

        // Обрабатываем падение тайлов после очистки
        let drops = drop_tiles_with_info(&mut board);
        total_drops.extend(drops);
    }

    // Если не было совпадений — откатываем ход
    if !had_matches {
        board[y1][x1] ^= board[y2][x2];
        board[y2][x2] ^= board[y1][x1];
        board[y1][x1] ^= board[y2][x2];
        return GameState { board, score, drops: vec![] };
    }

    // Возвращаем финальное состояние игры
    GameState {
        board,
        score: total_score,
        drops: total_drops,
    }
}


/// Активация бонусного тайла — очищает строку и столбец
#[command]
pub fn activate_bonus_tile(x: usize, y: usize, mut board: Board, score: u32) -> GameState {
    if board[y][x] != BONUS_TILE {
        return GameState { board, score, drops: vec![] };
    }

    board[y][x] = 0;

    let mut total_score = score;
    let mut positions_to_clear = HashSet::new();

    for i in 0..WIDTH {
        positions_to_clear.insert((i, y));
    }
    for j in 0..HEIGHT {
        positions_to_clear.insert((x, j));
    }

    for &(cx, cy) in &positions_to_clear {
        board[cy][cx] = 0;
    }

    total_score += positions_to_clear.len() as u32;
    let mut drops = drop_tiles_with_info(&mut board);

    let mut bonus_placed = false;

    // Повторная обработка совпадений
    loop {
        let groups = find_matches(&board);
        if groups.is_empty() {
            break;
        }

        let all_matches: HashSet<_> = groups.iter().flatten().copied().collect();
        total_score += 3 + (all_matches.len() as u32 - 3);

        // Один бонус за длинную линию
        if !bonus_placed {
            if let Some(group) = groups.iter().find(|g| g.len() >= 5) {
                let mid = group[group.len() / 2];
                board[mid.1][mid.0] = BONUS_TILE;
                bonus_placed = true;
            }
        }

        for &(x, y) in &all_matches {
            if board[y][x] != BONUS_TILE {
                board[y][x] = 0;
            }
        }

        let new_drops = drop_tiles_with_info(&mut board);
        drops.extend(new_drops);
    }

    GameState { board, score: total_score, drops }
}

/// Поиск всех совпадений (группами) по 3+ одинаковых тайла
pub fn find_matches(board: &Board) -> Vec<Vec<(usize, usize)>> {
    let mut matches = vec![];

    // Горизонтальные группы
    for y in 0..HEIGHT {
        let mut group: Vec<(usize, usize)> = vec![];

        for x in 0..WIDTH {
            if !group.is_empty() && board[y][x] != board[y][group[0].0] {
                if group.len() >= 3 {
                    matches.push(group.clone());
                }
                group.clear();
            }
            if board[y][x] != 0 {
                group.push((x, y));
            } else {
                if group.len() >= 3 {
                    matches.push(group.clone());
                }
                group.clear();
            }
        }
        if group.len() >= 3 {
            matches.push(group);
        }
    }

    // Вертикальные группы
    for x in 0..WIDTH {
let mut group: Vec<(usize, usize)> = vec![];
        for y in 0..HEIGHT {
            if !group.is_empty() && board[y][x] != board[group[0].1][x] {
                if group.len() >= 3 {
                    matches.push(group.clone());
                }
                group.clear();
            }
            if board[y][x] != 0 {
                group.push((x, y));
            } else {
                if group.len() >= 3 {
                    matches.push(group.clone());
                }
                group.clear();
            }
        }
        if group.len() >= 3 {
            matches.push(group);
        }
    }

    matches
}

/// Опускает тайлы вниз и генерирует новые
pub fn drop_tiles_with_info(board: &mut Board) -> Vec<DropInfo> {
    let mut rng = rand::thread_rng();
    let mut drops = vec![];

    for x in 0..WIDTH {
        let mut write_y = HEIGHT as isize - 1;

        for y in (0..HEIGHT).rev() {
            if board[y][x] != 0 {
                if y as isize != write_y {
                    board[write_y as usize][x] = board[y][x];
                    board[y][x] = 0;

                    drops.push(DropInfo {
                        from_x: x,
                        from_y: y,
                        to_x: x,
                        to_y: write_y as usize,
                        value: board[write_y as usize][x],
                    });
                }
                write_y -= 1;
            }
        }

        for y in (0..=write_y).rev() {
            let value = rng.gen_range(1..=TILE_TYPES) as u8;
            board[y as usize][x] = value;

            drops.push(DropInfo {
                from_x: x,
                from_y: 0,
                to_x: x,
                to_y: y as usize,
                value,
            });
        }
    }

    drops
}
