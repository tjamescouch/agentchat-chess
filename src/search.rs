// === Search ===
// Module owner: @i3mjagsb

use crate::eval::evaluate;
use crate::movegen::generate_moves;
use crate::types::*;

const INF: i32 = 100_000;

/// Find best move at given depth
pub fn search(board: &mut impl ChessBoard, depth: u8) -> (Move, i32) {
    let mut best_move = None;
    let mut best_score = -INF;

    for m in generate_moves(board) {
        board.make_move(m);
        let score = -negamax(board, depth - 1, -INF, INF);
        board.unmake_move();

        if score > best_score {
            best_score = score;
            best_move = Some(m);
        }
    }
    (best_move.expect("no legal moves"), best_score)
}

/// Negamax with alpha-beta pruning
fn negamax(board: &mut impl ChessBoard, depth: u8, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let moves = generate_moves(board);
    if moves.is_empty() {
        // No legal moves: checkmate or stalemate
        return if board.is_in_check(board.side_to_move()) {
            -INF + 1 // Checkmate (add 1 to prefer shorter mates)
        } else {
            0 // Stalemate
        };
    }

    for m in moves {
        board.make_move(m);
        let score = -negamax(board, depth - 1, -beta, -alpha);
        board.unmake_move();

        if score >= beta {
            return beta; // Beta cutoff
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

// Phase 2 improvements:
// - Iterative deepening
// - Move ordering (captures first, killer moves, history heuristic)
// - Transposition table
// - Quiescence search
// - Check extensions
