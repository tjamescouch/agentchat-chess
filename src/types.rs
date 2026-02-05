// === Shared Types ===
// Designed by @rea78sbq with input from @i3mjagsb

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

pub type Bitboard = u64;
pub type Square = u8; // 0=a1, 7=h1, 56=a8, 63=h8

// Square constants
pub const A1: Square = 0;
pub const B1: Square = 1;
pub const C1: Square = 2;
pub const D1: Square = 3;
pub const E1: Square = 4;
pub const F1: Square = 5;
pub const G1: Square = 6;
pub const H1: Square = 7;
pub const A8: Square = 56;
pub const B8: Square = 57;
pub const C8: Square = 58;
pub const D8: Square = 59;
pub const E8: Square = 60;
pub const F8: Square = 61;
pub const G8: Square = 62;
pub const H8: Square = 63;

// Castling rights bits
pub const WHITE_KINGSIDE: u8 = 1;
pub const WHITE_QUEENSIDE: u8 = 2;
pub const BLACK_KINGSIDE: u8 = 4;
pub const BLACK_QUEENSIDE: u8 = 8;

#[derive(Copy, Clone, Debug, Default)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
    pub is_castle: bool,
    pub is_en_passant: bool,
}

/// Iterator over set bits in a Bitboard
pub struct BitIter(pub Bitboard);

impl Iterator for BitIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let sq = self.0.trailing_zeros() as Square;
            self.0 &= self.0 - 1; // clear lowest set bit
            Some(sq)
        }
    }
}

/// Core trait for chess board implementations
/// Clone bound added by @i3mjagsb for search tree exploration
pub trait ChessBoard: Clone {
    fn piece_at(&self, sq: Square) -> Option<(Piece, Color)>;
    fn pieces(&self, color: Color, piece: Piece) -> Bitboard;
    fn occupancy(&self, color: Color) -> Bitboard;
    fn side_to_move(&self) -> Color;
    fn make_move(&mut self, m: Move);
    fn unmake_move(&mut self);
    fn is_capture(&self, m: Move) -> bool;
    fn halfmove_clock(&self) -> u8;
    fn zobrist_hash(&self) -> u64;
    fn is_in_check(&self, color: Color) -> bool;
    fn castling_rights(&self) -> u8;
    fn en_passant_square(&self) -> Option<Square>;
    fn is_square_attacked(&self, sq: Square, by_color: Color) -> bool;
}
