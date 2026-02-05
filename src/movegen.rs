// === Move Generation ===
// Module owner: @rpbr2qqf

use crate::types::*;

/// Precomputed knight attack bitboards
const fn precompute_knight_attacks() -> [Bitboard; 64] {
    let mut attacks = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        let bb = 1u64 << sq;
        let file = sq % 8;
        let rank = sq / 8;
        let mut attack = 0u64;

        if file > 0 && rank < 6 { attack |= bb << 15; }
        if file < 7 && rank < 6 { attack |= bb << 17; }
        if file > 1 && rank < 7 { attack |= bb << 6; }
        if file < 6 && rank < 7 { attack |= bb << 10; }
        if file > 0 && rank > 1 { attack |= bb >> 17; }
        if file < 7 && rank > 1 { attack |= bb >> 15; }
        if file > 1 && rank > 0 { attack |= bb >> 10; }
        if file < 6 && rank > 0 { attack |= bb >> 6; }

        attacks[sq] = attack;
        sq += 1;
    }
    attacks
}

/// Precomputed king attack bitboards
const fn precompute_king_attacks() -> [Bitboard; 64] {
    let mut attacks = [0u64; 64];
    let mut sq = 0;
    while sq < 64 {
        let bb = 1u64 << sq;
        let file = sq % 8;
        let rank = sq / 8;
        let mut attack = 0u64;

        if rank < 7 { attack |= bb << 8; }
        if rank > 0 { attack |= bb >> 8; }
        if file > 0 { attack |= bb >> 1; }
        if file < 7 { attack |= bb << 1; }
        if file > 0 && rank < 7 { attack |= bb << 7; }
        if file < 7 && rank < 7 { attack |= bb << 9; }
        if file > 0 && rank > 0 { attack |= bb >> 9; }
        if file < 7 && rank > 0 { attack |= bb >> 7; }

        attacks[sq] = attack;
        sq += 1;
    }
    attacks
}

static KNIGHT_ATTACKS: [Bitboard; 64] = precompute_knight_attacks();
static KING_ATTACKS: [Bitboard; 64] = precompute_king_attacks();

pub fn generate_moves(board: &impl ChessBoard) -> Vec<Move> {
    let mut moves = Vec::with_capacity(256);
    let us = board.side_to_move();

    generate_pawn_moves(board, us, &mut moves);
    generate_knight_moves(board, us, &mut moves);
    generate_bishop_moves(board, us, &mut moves);
    generate_rook_moves(board, us, &mut moves);
    generate_queen_moves(board, us, &mut moves);
    generate_king_moves(board, us, &mut moves);
    generate_castling_moves(board, us, &mut moves);

    // Filter to legal moves only
    moves.retain(|m| is_legal(board, *m));
    moves
}

fn generate_pawn_moves(board: &impl ChessBoard, us: Color, moves: &mut Vec<Move>) {
    let pawns = board.pieces(us, Piece::Pawn);
    let empty = !(board.occupancy(Color::White) | board.occupancy(Color::Black));
    let enemies = board.occupancy(us.opposite());

    let (push_dir, start_rank_mask, promo_rank_mask): (i8, Bitboard, Bitboard) = match us {
        Color::White => (8, 0x000000000000FF00, 0xFF00000000000000),
        Color::Black => (-8, 0x00FF000000000000, 0x00000000000000FF),
    };

    for from in BitIter(pawns) {
        let to = (from as i8 + push_dir) as Square;
        if to >= 64 { continue; }
        let to_mask = 1u64 << to;

        // Single push
        if empty & to_mask != 0 {
            if promo_rank_mask & to_mask != 0 {
                for promo in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                    moves.push(Move {
                        from,
                        to,
                        promotion: Some(promo),
                        is_castle: false,
                        is_en_passant: false,
                    });
                }
            } else {
                moves.push(Move { from, to, promotion: None, is_castle: false, is_en_passant: false });

                // Double push from starting rank
                let from_mask = 1u64 << from;
                if start_rank_mask & from_mask != 0 {
                    let double_to = (to as i8 + push_dir) as Square;
                    if double_to < 64 && empty & (1u64 << double_to) != 0 {
                        moves.push(Move { from, to: double_to, promotion: None, is_castle: false, is_en_passant: false });
                    }
                }
            }
        }

        // Captures (including en passant)
        let capture_dirs: &[i8] = match us {
            Color::White => &[7, 9],
            Color::Black => &[-7, -9],
        };

        for &dir in capture_dirs {
            let cap_to = from as i8 + dir;
            if cap_to < 0 || cap_to >= 64 { continue; }
            let cap_to = cap_to as Square;

            // Check for file wrap
            let from_file = from % 8;
            let to_file = cap_to % 8;
            if (from_file as i8 - to_file as i8).abs() != 1 { continue; }

            let cap_mask = 1u64 << cap_to;

            // Normal capture
            if enemies & cap_mask != 0 {
                if promo_rank_mask & cap_mask != 0 {
                    for promo in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                        moves.push(Move {
                            from,
                            to: cap_to,
                            promotion: Some(promo),
                            is_castle: false,
                            is_en_passant: false,
                        });
                    }
                } else {
                    moves.push(Move { from, to: cap_to, promotion: None, is_castle: false, is_en_passant: false });
                }
            }
            // En passant capture
            else if let Some(ep_sq) = board.en_passant_square() {
                if cap_to == ep_sq {
                    moves.push(Move {
                        from,
                        to: cap_to,
                        promotion: None,
                        is_castle: false,
                        is_en_passant: true,
                    });
                }
            }
        }
    }
}

fn generate_knight_moves(board: &impl ChessBoard, us: Color, moves: &mut Vec<Move>) {
    let knights = board.pieces(us, Piece::Knight);
    let valid_targets = !board.occupancy(us);

    for from in BitIter(knights) {
        let attacks = KNIGHT_ATTACKS[from as usize] & valid_targets;
        for to in BitIter(attacks) {
            moves.push(Move { from, to, promotion: None, is_castle: false, is_en_passant: false });
        }
    }
}

fn generate_bishop_moves(board: &impl ChessBoard, us: Color, moves: &mut Vec<Move>) {
    let bishops = board.pieces(us, Piece::Bishop);
    let valid_targets = !board.occupancy(us);
    let all_pieces = board.occupancy(Color::White) | board.occupancy(Color::Black);

    for from in BitIter(bishops) {
        let attacks = sliding_attacks(from, all_pieces, true) & valid_targets;
        for to in BitIter(attacks) {
            moves.push(Move { from, to, promotion: None, is_castle: false, is_en_passant: false });
        }
    }
}

fn generate_rook_moves(board: &impl ChessBoard, us: Color, moves: &mut Vec<Move>) {
    let rooks = board.pieces(us, Piece::Rook);
    let valid_targets = !board.occupancy(us);
    let all_pieces = board.occupancy(Color::White) | board.occupancy(Color::Black);

    for from in BitIter(rooks) {
        let attacks = sliding_attacks(from, all_pieces, false) & valid_targets;
        for to in BitIter(attacks) {
            moves.push(Move { from, to, promotion: None, is_castle: false, is_en_passant: false });
        }
    }
}

fn generate_queen_moves(board: &impl ChessBoard, us: Color, moves: &mut Vec<Move>) {
    let queens = board.pieces(us, Piece::Queen);
    let valid_targets = !board.occupancy(us);
    let all_pieces = board.occupancy(Color::White) | board.occupancy(Color::Black);

    for from in BitIter(queens) {
        let attacks = (sliding_attacks(from, all_pieces, true)
            | sliding_attacks(from, all_pieces, false))
            & valid_targets;
        for to in BitIter(attacks) {
            moves.push(Move { from, to, promotion: None, is_castle: false, is_en_passant: false });
        }
    }
}

fn generate_king_moves(board: &impl ChessBoard, us: Color, moves: &mut Vec<Move>) {
    let king = board.pieces(us, Piece::King);
    let valid_targets = !board.occupancy(us);

    for from in BitIter(king) {
        let attacks = KING_ATTACKS[from as usize] & valid_targets;
        for to in BitIter(attacks) {
            moves.push(Move { from, to, promotion: None, is_castle: false, is_en_passant: false });
        }
    }
}

fn generate_castling_moves(board: &impl ChessBoard, us: Color, moves: &mut Vec<Move>) {
    let rights = board.castling_rights();
    let all_pieces = board.occupancy(Color::White) | board.occupancy(Color::Black);
    let enemy = us.opposite();

    // Can't castle while in check
    if board.is_in_check(us) {
        return;
    }

    match us {
        Color::White => {
            // White kingside (e1-g1)
            if rights & WHITE_KINGSIDE != 0 {
                // Check squares between king and rook are empty
                if all_pieces & 0x60 == 0 {  // f1, g1
                    // Check king doesn't pass through or end up in check
                    if !board.is_square_attacked(F1, enemy) && !board.is_square_attacked(G1, enemy) {
                        moves.push(Move { from: E1, to: G1, promotion: None, is_castle: true, is_en_passant: false });
                    }
                }
            }
            // White queenside (e1-c1)
            if rights & WHITE_QUEENSIDE != 0 {
                // Check squares between king and rook are empty
                if all_pieces & 0x0E == 0 {  // b1, c1, d1
                    // Check king doesn't pass through or end up in check
                    if !board.is_square_attacked(D1, enemy) && !board.is_square_attacked(C1, enemy) {
                        moves.push(Move { from: E1, to: C1, promotion: None, is_castle: true, is_en_passant: false });
                    }
                }
            }
        }
        Color::Black => {
            // Black kingside (e8-g8)
            if rights & BLACK_KINGSIDE != 0 {
                if all_pieces & 0x6000000000000000 == 0 {  // f8, g8
                    if !board.is_square_attacked(F8, enemy) && !board.is_square_attacked(G8, enemy) {
                        moves.push(Move { from: E8, to: G8, promotion: None, is_castle: true, is_en_passant: false });
                    }
                }
            }
            // Black queenside (e8-c8)
            if rights & BLACK_QUEENSIDE != 0 {
                if all_pieces & 0x0E00000000000000 == 0 {  // b8, c8, d8
                    if !board.is_square_attacked(D8, enemy) && !board.is_square_attacked(C8, enemy) {
                        moves.push(Move { from: E8, to: C8, promotion: None, is_castle: true, is_en_passant: false });
                    }
                }
            }
        }
    }
}

/// Simple ray-based sliding piece attacks
fn sliding_attacks(sq: Square, blockers: Bitboard, diagonal: bool) -> Bitboard {
    let mut attacks = 0u64;
    let directions: &[(i8, i8)] = if diagonal {
        &[(1, 1), (1, -1), (-1, 1), (-1, -1)]
    } else {
        &[(0, 1), (0, -1), (1, 0), (-1, 0)]
    };

    for &(dr, df) in directions {
        let mut r = (sq / 8) as i8;
        let mut f = (sq % 8) as i8;
        loop {
            r += dr;
            f += df;
            if r < 0 || r > 7 || f < 0 || f > 7 {
                break;
            }
            let target = (r * 8 + f) as Square;
            attacks |= 1u64 << target;
            if blockers & (1u64 << target) != 0 {
                break;
            }
        }
    }
    attacks
}

fn is_legal(board: &impl ChessBoard, m: Move) -> bool {
    let mut test_board = board.clone();
    test_board.make_move(m);
    !test_board.is_in_check(board.side_to_move())
}

/// Perft: count leaf nodes at given depth (for testing)
pub fn perft(board: &mut impl ChessBoard, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = generate_moves(board);
    if depth == 1 {
        return moves.len() as u64;
    }

    moves
        .iter()
        .map(|m| {
            board.make_move(*m);
            let count = perft(board, depth - 1);
            board.unmake_move();
            count
        })
        .sum()
}
