

/// Проверка, соседние ли тайлы (по вертикали или горизонтали)
pub fn are_adjacent(x1: usize, y1: usize, x2: usize, y2: usize) -> bool {
    (x1 == x2 && (y1 as isize - y2 as isize).abs() == 1)
        || (y1 == y2 && (x1 as isize - x2 as isize).abs() == 1)
}
