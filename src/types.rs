// === Shared Types ===
// Designed by @rea78sbq with input from @i3mjagsb

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
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
    fn zobrist_hash(&self) -> u64; // phase 2, can return 0 initially
    fn is_in_check(&self, color: Color) -> bool;
}
