import './App.css';
import { invoke } from "@tauri-apps/api";

let currentBoard = [];
let currentScore = 0;
let selected = null;

const COLORS = {
  1: "#FF6B6B",
  2: "#4ECDC4",
  3: "#FFD93D",
  4: "#A29BFE",
  5: "#00B894",
};

function startGame() {
  invoke("generate_board").then((result) => {
    currentBoard = result.board;
    currentScore = result.score;
    selected = null;
    renderBoard(currentBoard);
    updateScore();
  });
}

function updateScore() {
  document.getElementById("score").textContent = "Очки: " + currentScore;
}

// Отрисовка доски — используем абсолютные тайлы
function renderBoard(board, animate = false) {
  const boardElement = document.getElementById("board");
  boardElement.innerHTML = "";

  const tileSize = 40 + 4;

  boardElement.style.position = "relative";
  boardElement.style.width = `${board[0].length * tileSize}px`;
  boardElement.style.height = `${board.length * tileSize}px`;

  board.forEach((row, y) => {
    row.forEach((tile, x) => {
      if (tile === 0) return;

      const tileDiv = document.createElement("div");
      tileDiv.className = "tile";
      tileDiv.dataset.x = x;
      tileDiv.dataset.y = y;
      tileDiv.textContent = tile;
      tileDiv.style.backgroundColor = COLORS[tile] || "#ccc";

      // Позиционируем тайл
      tileDiv.style.position = "absolute";
      tileDiv.style.transform = `translate(${x * tileSize}px, ${y * tileSize}px)`;

      if (selected && selected.x === x && selected.y === y) {
        tileDiv.classList.add("selected");
      }

      tileDiv.onclick = () => handleTileClick(x, y);

      // Анимация появления новых тайлов
      if (animate && tile !== 0) {
        tileDiv.classList.add("appear");
        void tileDiv.offsetWidth;
        tileDiv.classList.add("appear");
        tileDiv.addEventListener("animationend", () => {
          tileDiv.classList.remove("appear");
        }, { once: true });
      }

      boardElement.appendChild(tileDiv);
    });
  });
}

// Анимация исчезающих тайлов
function animateRemovingTiles(removingTiles, onComplete) {
  const boardElement = document.getElementById("board");
  let completed = 0;

  removingTiles.forEach(({ x, y }) => {
    const tile = boardElement.querySelector(`.tile[data-x="${x}"][data-y="${y}"]`);
    if (tile) {
      tile.classList.add("removing");
      tile.addEventListener("animationend", () => {
        completed++;
        if (completed === removingTiles.length) {
          onComplete();
        }
      }, { once: true });
    }
  });

  if (removingTiles.length === 0) {
    onComplete();
  }
}

// Анимация падения тайлов после удаления
function animateFallingTiles(oldBoard, newBoard, onComplete) {
  const boardElement = document.getElementById("board");
  const tileSize = 40 + 4;
  const moves = [];

  for (let y = 0; y < oldBoard.length; y++) {
    for (let x = 0; x < oldBoard[0].length; x++) {
      const oldVal = oldBoard[y][x];
      if (oldVal === 0) continue;

      for (let ny = y + 1; ny < newBoard.length; ny++) {
        if (newBoard[ny][x] === oldVal && oldBoard[ny][x] === 0) {
          moves.push({ x, y, toY: ny, value: oldVal });
          break;
        }
      }
    }
  }

  let completed = 0;

  moves.forEach(({ x, y, toY }) => {
    const tile = boardElement.querySelector(`.tile[data-x="${x}"][data-y="${y}"]`);
    if (!tile) return;

    // Анимация transform
    tile.style.transition = 'transform 0.3s ease';
    tile.style.transform = `translate(${x * tileSize}px, ${toY * tileSize}px)`;

    tile.addEventListener("transitionend", () => {
      completed++;
      if (completed === moves.length) {
        onComplete();
      }
    }, { once: true });
  });

  if (moves.length === 0) {
    onComplete();
  }
}

// Анимация перемещения двух соседних тайлов при клике
function animateTileSwap(x1, y1, x2, y2, callback) {
  const boardElement = document.getElementById("board");

  const tile1 = boardElement.querySelector(`.tile[data-x="${x1}"][data-y="${y1}"]`);
  const tile2 = boardElement.querySelector(`.tile[data-x="${x2}"][data-y="${y2}"]`);

  if (!tile1 || !tile2) {
    callback();
    return;
  }

  const rect1 = tile1.getBoundingClientRect();
  const rect2 = tile2.getBoundingClientRect();

  const dx = rect2.left - rect1.left;
  const dy = rect2.top - rect1.top;

  tile1.style.transition = 'transform 0.25s ease';
  tile2.style.transition = 'transform 0.25s ease';
  tile1.style.transform += ` translate(${dx}px, ${dy}px)`;
  tile2.style.transform += ` translate(${-dx}px, ${-dy}px)`;

  tile1.classList.add("moving");
  tile2.classList.add("moving");

  setTimeout(() => {
    tile1.style.transition = '';
    tile2.style.transition = '';
    tile1.style.transform = '';
    tile2.style.transform = '';
    tile1.classList.remove("moving");
    tile2.classList.remove("moving");

    callback();
  }, 250);
}

// Обработка клика по тайлу
function handleTileClick(x, y) {
  if (!selected) {
    selected = { x, y };
    renderBoard(currentBoard);
  } else {
    const x1 = selected.x, y1 = selected.y;
    const x2 = x, y2 = y;
    selected = null;

    const isAdjacent = (Math.abs(x1 - x2) + Math.abs(y1 - y2)) === 1;
    if (!isAdjacent) {
      renderBoard(currentBoard);
      return;
    }

    animateTileSwap(x1, y1, x2, y2, () => {
      invoke("move_tile", {
        x1, y1, x2, y2,
        board: currentBoard,
        score: currentScore,
      }).then((result) => {
        if (JSON.stringify(result.board) !== JSON.stringify(currentBoard)) {
          const removingTiles = [];
          for (let y = 0; y < currentBoard.length; y++) {
            for (let x = 0; x < currentBoard[y].length; x++) {
              if (currentBoard[y][x] !== 0 && result.board[y][x] === 0) {
                removingTiles.push({ x, y });
              }
            }
          }

          animateRemovingTiles(removingTiles, () => {
            const oldBoard = currentBoard.map(row => [...row]);
            currentBoard = result.board;
            currentScore = result.score;

            renderBoard(oldBoard); // визуально покажем старые тайлы
            requestAnimationFrame(() => {
              animateFallingTiles(oldBoard, currentBoard, () => {
                renderBoard(currentBoard, true); // финальный рендер
                updateScore();
              });
            });
          });

        } else {
          renderBoard(currentBoard); // неудачный ход
        }
      }).catch((err) => {
        console.error("Move error:", err);
        renderBoard(currentBoard);
      });
    });
  }
}

window.startGame = startGame;
