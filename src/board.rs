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
    castling_rights: u8, // KQkq = bits 0-3
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
            castling_rights: 0b1111,
            en_passant_sq: None,
            halfmove_clock: 0,
            history: Vec::new(),
        };
        board.set_startpos();
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
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl ChessBoard for Board {
    fn piece_at(&self, sq: Square) -> Option<(Piece, Color)> {
        let mask = 1u64 << sq;
        for color in [Color::White, Color::Black] {
            if self.occupancy[color as usize] & mask != 0 {
                for piece in [
                    Piece::Pawn,
                    Piece::Knight,
                    Piece::Bishop,
                    Piece::Rook,
                    Piece::Queen,
                    Piece::King,
                ] {
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

    fn make_move(&mut self, m: Move) {
        let us = self.side_to_move as usize;
        let them = self.side_to_move.opposite() as usize;
        let from_mask = 1u64 << m.from;
        let to_mask = 1u64 << m.to;

        // Find the moving piece
        let mut moving_piece = None;
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            if self.pieces[us][piece as usize] & from_mask != 0 {
                moving_piece = Some(piece);
                break;
            }
        }
        let moving_piece = moving_piece.expect("no piece at from square");

        // Find captured piece (if any)
        let mut captured = None;
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
        ] {
            if self.pieces[them][piece as usize] & to_mask != 0 {
                captured = Some(piece);
                self.pieces[them][piece as usize] ^= to_mask;
                break;
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

        // Move the piece
        self.pieces[us][moving_piece as usize] ^= from_mask | to_mask;

        // Handle promotion
        if let Some(promo) = m.promotion {
            self.pieces[us][Piece::Pawn as usize] ^= to_mask;
            self.pieces[us][promo as usize] ^= to_mask;
        }

        // TODO: Handle castling, en passant

        self.update_occupancy();
        self.side_to_move = self.side_to_move.opposite();
        self.halfmove_clock += 1;
        if captured.is_some() || moving_piece == Piece::Pawn {
            self.halfmove_clock = 0;
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

        // Find the piece that moved (now at 'to')
        let mut moving_piece = None;
        for piece in [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ] {
            if self.pieces[us][piece as usize] & to_mask != 0 {
                moving_piece = Some(piece);
                break;
            }
        }

        // Handle promotion - the piece at 'to' is the promoted piece
        if let Some(promo) = m.promotion {
            self.pieces[us][promo as usize] ^= to_mask;
            self.pieces[us][Piece::Pawn as usize] ^= from_mask;
        } else if let Some(piece) = moving_piece {
            self.pieces[us][piece as usize] ^= from_mask | to_mask;
        }

        // Restore captured piece
        if let Some(captured) = undo.captured {
            self.pieces[them][captured as usize] ^= to_mask;
        }

        self.castling_rights = undo.castling_rights;
        self.en_passant_sq = undo.en_passant_sq;
        self.halfmove_clock = undo.halfmove_clock;
        self.update_occupancy();
    }

    fn is_capture(&self, m: Move) -> bool {
        let them = self.side_to_move.opposite();
        (self.occupancy[them as usize] & (1u64 << m.to)) != 0
    }

    fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    fn zobrist_hash(&self) -> u64 {
        // Phase 2: implement proper Zobrist hashing
        0
    }

    fn is_in_check(&self, color: Color) -> bool {
        // TODO: implement attack detection
        // For now, return false (unsafe but allows compilation)
        false
    }
}
