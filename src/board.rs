// === Board Representation ===
// Module owner: @rea78sbq

use crate::types::*;

/// Undo information for unmake_move
#[derive(Clone)]
struct MoveUndo {
    m: Move,
    captured: Option<Piece>,
    castling_rights: u8,
    en_passant_sq: Option<Square>,
    halfmove_clock: u8,
}

#[derive(Clone)]
pub struct Board {
    pieces: [[Bitboard; 6]; 2], // [color][piece_type]
    occupancy: [Bitboard; 2],   // per color
    side_to_move: Color,
    castling_rights: u8,
    en_passant_sq: Option<Square>,
    halfmove_clock: u8,
    history: Vec<MoveUndo>,
}

impl Board {
    /// Create starting position
    pub fn new() -> Self {
        let mut board = Self {
            pieces: [[0; 6]; 2],
            occupancy: [0; 2],
            side_to_move: Color::White,
            castling_rights: WHITE_KINGSIDE | WHITE_QUEENSIDE | BLACK_KINGSIDE | BLACK_QUEENSIDE,
            en_passant_sq: None,
            halfmove_clock: 0,
            history: Vec::new(),
        };
        board.set_startpos();
        board
    }

    /// Create board from FEN parts
    pub fn from_fen(parts: &[&str]) -> Self {
        let mut board = Self {
            pieces: [[0; 6]; 2],
            occupancy: [0; 2],
            side_to_move: Color::White,
            castling_rights: 0,
            en_passant_sq: None,
            halfmove_clock: 0,
            history: Vec::new(),
        };

        // Parse piece placement (part 0)
        if !parts.is_empty() {
            let mut sq: i8 = 56; // Start at a8
            for c in parts[0].chars() {
                match c {
                    '/' => sq -= 16, // Next rank
                    '1'..='8' => sq += (c as i8 - '0' as i8),
                    _ => {
                        let (piece, color) = match c {
                            'P' => (Piece::Pawn, Color::White),
                            'N' => (Piece::Knight, Color::White),
                            'B' => (Piece::Bishop, Color::White),
                            'R' => (Piece::Rook, Color::White),
                            'Q' => (Piece::Queen, Color::White),
                            'K' => (Piece::King, Color::White),
                            'p' => (Piece::Pawn, Color::Black),
                            'n' => (Piece::Knight, Color::Black),
                            'b' => (Piece::Bishop, Color::Black),
                            'r' => (Piece::Rook, Color::Black),
                            'q' => (Piece::Queen, Color::Black),
                            'k' => (Piece::King, Color::Black),
                            _ => continue,
                        };
                        if sq >= 0 && sq < 64 {
                            board.pieces[color as usize][piece as usize] |= 1u64 << sq;
                        }
                        sq += 1;
                    }
                }
            }
        }

        // Parse side to move (part 1)
        if parts.len() > 1 {
            board.side_to_move = if parts[1] == "b" { Color::Black } else { Color::White };
        }

        // Parse castling rights (part 2)
        if parts.len() > 2 {
            for c in parts[2].chars() {
                match c {
                    'K' => board.castling_rights |= WHITE_KINGSIDE,
                    'Q' => board.castling_rights |= WHITE_QUEENSIDE,
                    'k' => board.castling_rights |= BLACK_KINGSIDE,
                    'q' => board.castling_rights |= BLACK_QUEENSIDE,
                    _ => {}
                }
            }
        }

        // Parse en passant square (part 3)
        if parts.len() > 3 && parts[3] != "-" {
            let bytes = parts[3].as_bytes();
            if bytes.len() >= 2 {
                let file = bytes[0].wrapping_sub(b'a');
                let rank = bytes[1].wrapping_sub(b'1');
                if file < 8 && rank < 8 {
                    board.en_passant_sq = Some(rank * 8 + file);
                }
            }
        }

        // Parse halfmove clock (part 4)
        if parts.len() > 4 {
            board.halfmove_clock = parts[4].parse().unwrap_or(0);
        }

        board.update_occupancy();
        board
    }

    fn set_startpos(&mut self) {
        // White pieces
        self.pieces[0][Piece::Pawn as usize] = 0x000000000000FF00;
        self.pieces[0][Piece::Knight as usize] = 0x0000000000000042;
        self.pieces[0][Piece::Bishop as usize] = 0x0000000000000024;
        self.pieces[0][Piece::Rook as usize] = 0x0000000000000081;
        self.pieces[0][Piece::Queen as usize] = 0x0000000000000008;
        self.pieces[0][Piece::King as usize] = 0x0000000000000010;

        // Black pieces
        self.pieces[1][Piece::Pawn as usize] = 0x00FF000000000000;
        self.pieces[1][Piece::Knight as usize] = 0x4200000000000000;
        self.pieces[1][Piece::Bishop as usize] = 0x2400000000000000;
        self.pieces[1][Piece::Rook as usize] = 0x8100000000000000;
        self.pieces[1][Piece::Queen as usize] = 0x0800000000000000;
        self.pieces[1][Piece::King as usize] = 0x1000000000000000;

        self.update_occupancy();
    }

    fn update_occupancy(&mut self) {
        for color in 0..2 {
            self.occupancy[color] = self.pieces[color].iter().fold(0, |acc, &bb| acc | bb);
        }
    }

    fn find_piece_at(&self, sq: Square, color: usize) -> Option<Piece> {
        let mask = 1u64 << sq;
        for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King] {
            if self.pieces[color][piece as usize] & mask != 0 {
                return Some(piece);
            }
        }
        None
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

// Attack generation helpers (used by is_square_attacked)
fn knight_attacks(sq: Square) -> Bitboard {
    let bb = 1u64 << sq;
    let mut attacks = 0u64;
    let file = sq % 8;
    let rank = sq / 8;

    if file > 0 && rank < 6 { attacks |= bb << 15; }
    if file < 7 && rank < 6 { attacks |= bb << 17; }
    if file > 1 && rank < 7 { attacks |= bb << 6; }
    if file < 6 && rank < 7 { attacks |= bb << 10; }
    if file > 0 && rank > 1 { attacks |= bb >> 17; }
    if file < 7 && rank > 1 { attacks |= bb >> 15; }
    if file > 1 && rank > 0 { attacks |= bb >> 10; }
    if file < 6 && rank > 0 { attacks |= bb >> 6; }
    attacks
}

fn king_attacks(sq: Square) -> Bitboard {
    let bb = 1u64 << sq;
    let mut attacks = 0u64;
    let file = sq % 8;
    let rank = sq / 8;

    if rank < 7 { attacks |= bb << 8; }
    if rank > 0 { attacks |= bb >> 8; }
    if file > 0 { attacks |= bb >> 1; }
    if file < 7 { attacks |= bb << 1; }
    if file > 0 && rank < 7 { attacks |= bb << 7; }
    if file < 7 && rank < 7 { attacks |= bb << 9; }
    if file > 0 && rank > 0 { attacks |= bb >> 9; }
    if file < 7 && rank > 0 { attacks |= bb >> 7; }
    attacks
}

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

impl ChessBoard for Board {
    fn piece_at(&self, sq: Square) -> Option<(Piece, Color)> {
        let mask = 1u64 << sq;
        for color in [Color::White, Color::Black] {
            if self.occupancy[color as usize] & mask != 0 {
                for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King] {
                    if self.pieces[color as usize][piece as usize] & mask != 0 {
                        return Some((piece, color));
                    }
                }
            }
        }
        None
    }

    fn pieces(&self, color: Color, piece: Piece) -> Bitboard {
        self.pieces[color as usize][piece as usize]
    }

    fn occupancy(&self, color: Color) -> Bitboard {
        self.occupancy[color as usize]
    }

    fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    fn castling_rights(&self) -> u8 {
        self.castling_rights
    }

    fn en_passant_square(&self) -> Option<Square> {
        self.en_passant_sq
    }

    fn make_move(&mut self, m: Move) {
        let us = self.side_to_move as usize;
        let them = self.side_to_move.opposite() as usize;
        let from_mask = 1u64 << m.from;
        let to_mask = 1u64 << m.to;

        // Find the moving piece
        let moving_piece = self.find_piece_at(m.from, us).expect("no piece at from square");

        // Find captured piece (if any) - but not for en passant (handled separately)
        let mut captured = None;
        if !m.is_en_passant {
            for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
                if self.pieces[them][piece as usize] & to_mask != 0 {
                    captured = Some(piece);
                    self.pieces[them][piece as usize] ^= to_mask;
                    break;
                }
            }
        }

        // Save undo info
        self.history.push(MoveUndo {
            m,
            captured,
            castling_rights: self.castling_rights,
            en_passant_sq: self.en_passant_sq,
            halfmove_clock: self.halfmove_clock,
        });

        // Clear en passant (will be set if double pawn push)
        self.en_passant_sq = None;

        // Handle castling
        if m.is_castle {
            // Move king
            self.pieces[us][Piece::King as usize] ^= from_mask | to_mask;

            // Move rook
            let (rook_from, rook_to) = if m.to > m.from {
                // Kingside
                if us == 0 { (H1, F1) } else { (H8, F8) }
            } else {
                // Queenside
                if us == 0 { (A1, D1) } else { (A8, D8) }
            };
            self.pieces[us][Piece::Rook as usize] ^= (1u64 << rook_from) | (1u64 << rook_to);
        }
        // Handle en passant capture
        else if m.is_en_passant {
            // Move pawn
            self.pieces[us][Piece::Pawn as usize] ^= from_mask | to_mask;

            // Remove captured pawn (one rank behind the destination)
            let captured_sq = if self.side_to_move == Color::White {
                m.to - 8
            } else {
                m.to + 8
            };
            self.pieces[them][Piece::Pawn as usize] ^= 1u64 << captured_sq;
        }
        // Normal move
        else {
            self.pieces[us][moving_piece as usize] ^= from_mask | to_mask;

            // Handle promotion
            if let Some(promo) = m.promotion {
                self.pieces[us][Piece::Pawn as usize] ^= to_mask;
                self.pieces[us][promo as usize] ^= to_mask;
            }

            // Set en passant square for double pawn push
            if moving_piece == Piece::Pawn {
                let diff = (m.to as i8 - m.from as i8).abs();
                if diff == 16 {
                    self.en_passant_sq = Some((m.from as i8 + (m.to as i8 - m.from as i8) / 2) as Square);
                }
            }
        }

        // Update castling rights
        // King moves
        if moving_piece == Piece::King {
            if us == 0 {
                self.castling_rights &= !(WHITE_KINGSIDE | WHITE_QUEENSIDE);
            } else {
                self.castling_rights &= !(BLACK_KINGSIDE | BLACK_QUEENSIDE);
            }
        }
        // Rook moves or is captured
        if m.from == A1 || m.to == A1 { self.castling_rights &= !WHITE_QUEENSIDE; }
        if m.from == H1 || m.to == H1 { self.castling_rights &= !WHITE_KINGSIDE; }
        if m.from == A8 || m.to == A8 { self.castling_rights &= !BLACK_QUEENSIDE; }
        if m.from == H8 || m.to == H8 { self.castling_rights &= !BLACK_KINGSIDE; }

        self.update_occupancy();
        self.side_to_move = self.side_to_move.opposite();

        // Update halfmove clock
        if captured.is_some() || m.is_en_passant || moving_piece == Piece::Pawn {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
    }

    fn unmake_move(&mut self) {
        let undo = self.history.pop().expect("no move to unmake");
        let m = undo.m;

        self.side_to_move = self.side_to_move.opposite();
        let us = self.side_to_move as usize;
        let them = self.side_to_move.opposite() as usize;
        let from_mask = 1u64 << m.from;
        let to_mask = 1u64 << m.to;

        // Handle castling
        if m.is_castle {
            // Move king back
            self.pieces[us][Piece::King as usize] ^= from_mask | to_mask;

            // Move rook back
            let (rook_from, rook_to) = if m.to > m.from {
                if us == 0 { (H1, F1) } else { (H8, F8) }
            } else {
                if us == 0 { (A1, D1) } else { (A8, D8) }
            };
            self.pieces[us][Piece::Rook as usize] ^= (1u64 << rook_from) | (1u64 << rook_to);
        }
        // Handle en passant
        else if m.is_en_passant {
            // Move pawn back
            self.pieces[us][Piece::Pawn as usize] ^= from_mask | to_mask;

            // Restore captured pawn
            let captured_sq = if self.side_to_move == Color::White {
                m.to - 8
            } else {
                m.to + 8
            };
            self.pieces[them][Piece::Pawn as usize] ^= 1u64 << captured_sq;
        }
        // Normal move
        else {
            // Handle promotion
            if let Some(promo) = m.promotion {
                self.pieces[us][promo as usize] ^= to_mask;
                self.pieces[us][Piece::Pawn as usize] ^= from_mask;
            } else {
                // Find piece at destination
                let moving_piece = self.find_piece_at(m.to, us).expect("no piece at to square");
                self.pieces[us][moving_piece as usize] ^= from_mask | to_mask;
            }

            // Restore captured piece
            if let Some(captured) = undo.captured {
                self.pieces[them][captured as usize] ^= to_mask;
            }
        }

        self.castling_rights = undo.castling_rights;
        self.en_passant_sq = undo.en_passant_sq;
        self.halfmove_clock = undo.halfmove_clock;
        self.update_occupancy();
    }

    fn is_capture(&self, m: Move) -> bool {
        if m.is_en_passant {
            return true;
        }
        let them = self.side_to_move.opposite();
        (self.occupancy[them as usize] & (1u64 << m.to)) != 0
    }

    fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    fn zobrist_hash(&self) -> u64 {
        0 // Phase 2
    }

    fn is_square_attacked(&self, sq: Square, by_color: Color) -> bool {
        let attacker = by_color as usize;
        let all_pieces = self.occupancy[0] | self.occupancy[1];

        // Pawn attacks
        let pawn_attacks = if by_color == Color::White {
            // White pawns attack diagonally upward
            let file = sq % 8;
            let mut attacks = 0u64;
            if sq >= 9 && file > 0 { attacks |= 1u64 << (sq - 9); }
            if sq >= 7 && file < 7 { attacks |= 1u64 << (sq - 7); }
            attacks
        } else {
            // Black pawns attack diagonally downward
            let file = sq % 8;
            let mut attacks = 0u64;
            if sq < 55 && file < 7 { attacks |= 1u64 << (sq + 9); }
            if sq < 57 && file > 0 { attacks |= 1u64 << (sq + 7); }
            attacks
        };
        if pawn_attacks & self.pieces[attacker][Piece::Pawn as usize] != 0 {
            return true;
        }

        // Knight attacks
        if knight_attacks(sq) & self.pieces[attacker][Piece::Knight as usize] != 0 {
            return true;
        }

        // King attacks
        if king_attacks(sq) & self.pieces[attacker][Piece::King as usize] != 0 {
            return true;
        }

        // Bishop/Queen (diagonal)
        let diagonal_attackers = self.pieces[attacker][Piece::Bishop as usize]
                                | self.pieces[attacker][Piece::Queen as usize];
        if sliding_attacks(sq, all_pieces, true) & diagonal_attackers != 0 {
            return true;
        }

        // Rook/Queen (straight)
        let straight_attackers = self.pieces[attacker][Piece::Rook as usize]
                                | self.pieces[attacker][Piece::Queen as usize];
        if sliding_attacks(sq, all_pieces, false) & straight_attackers != 0 {
            return true;
        }

        false
    }

    fn is_in_check(&self, color: Color) -> bool {
        let king_bb = self.pieces[color as usize][Piece::King as usize];
        if king_bb == 0 {
            return false;
        }
        let king_sq = king_bb.trailing_zeros() as Square;
        self.is_square_attacked(king_sq, color.opposite())
    }
}
