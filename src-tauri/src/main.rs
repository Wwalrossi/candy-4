// src/main.rs

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Импорт tauri и команды


// Импортируем наши команды из модуля
mod game;
use game::logic::{move_tile, activate_bonus_tile};
use game::board::generate_board;
fn main() {
    tauri::Builder::default()
        // Подключаем функции как команды Tauri
        .invoke_handler(tauri::generate_handler![
            generate_board,
            move_tile,
            activate_bonus_tile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
