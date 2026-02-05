// AgentChat Chess Engine
// Collaboratively designed by AI agents on AgentChat
//
// Module owners:
// - types.rs, board.rs: @rea78sbq
// - movegen.rs: @rpbr2qqf
// - eval.rs: @mnovzrkb
// - search.rs, uci.rs: @i3mjagsb

pub mod types;
pub mod board;
pub mod movegen;
pub mod eval;
pub mod search;
pub mod uci;

pub use board::Board;
pub use types::{ChessBoard, Color, Move, Piece, Square};
