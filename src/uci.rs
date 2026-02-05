// === UCI Protocol ===
// Module owner: @i3mjagsb

use crate::board::Board;
use crate::search::search;
use crate::types::*;
use std::io::{self, BufRead, Write};

pub fn uci_loop() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut board = Board::new();

    for line in stdin.lock().lines() {
        let input = match line {
            Ok(s) => s,
            Err(_) => break,
        };

        let tokens: Vec<&str> = input.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "uci" => {
                println!("id name AgentChat-Chess");
                println!("id author AgentChat Team (@rea78sbq @rpbr2qqf @mnovzrkb @i3mjagsb)");
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "ucinewgame" => board = Board::new(),
            "position" => parse_position(&mut board, &tokens),
            "go" => {
                let depth = parse_depth(&tokens);
                let (m, _score) = search(&mut board, depth);
                println!("bestmove {}", move_to_uci(m));
            }
            "quit" => break,
            "d" => debug_print(&board),
            _ => {}
        }

        stdout.flush().ok();
    }
}

fn parse_position(board: &mut Board, tokens: &[&str]) {
    let mut i = 1;
    if i >= tokens.len() {
        return;
    }

    if tokens[i] == "startpos" {
        *board = Board::new();
        i += 1;
    } else if tokens[i] == "fen" {
        // TODO: parse FEN
        i += 7; // skip fen parts
    }

    if i < tokens.len() && tokens[i] == "moves" {
        i += 1;
        while i < tokens.len() {
            if let Some(m) = uci_to_move(tokens[i]) {
                board.make_move(m);
            }
            i += 1;
        }
    }
}

fn parse_depth(tokens: &[&str]) -> u8 {
    for (i, &token) in tokens.iter().enumerate() {
        if token == "depth" && i + 1 < tokens.len() {
            return tokens[i + 1].parse().unwrap_or(6);
        }
    }
    6 // default depth
}

fn move_to_uci(m: Move) -> String {
    let from_file = (b'a' + m.from % 8) as char;
    let from_rank = (b'1' + m.from / 8) as char;
    let to_file = (b'a' + m.to % 8) as char;
    let to_rank = (b'1' + m.to / 8) as char;

    let mut s = format!("{}{}{}{}", from_file, from_rank, to_file, to_rank);
    if let Some(promo) = m.promotion {
        s.push(match promo {
            Piece::Queen => 'q',
            Piece::Rook => 'r',
            Piece::Bishop => 'b',
            Piece::Knight => 'n',
            _ => 'q',
        });
    }
    s
}

fn uci_to_move(s: &str) -> Option<Move> {
    let bytes = s.as_bytes();
    if bytes.len() < 4 {
        return None;
    }

    let from_file = bytes[0].wrapping_sub(b'a');
    let from_rank = bytes[1].wrapping_sub(b'1');
    let to_file = bytes[2].wrapping_sub(b'a');
    let to_rank = bytes[3].wrapping_sub(b'1');

    if from_file > 7 || from_rank > 7 || to_file > 7 || to_rank > 7 {
        return None;
    }

    let from = from_rank * 8 + from_file;
    let to = to_rank * 8 + to_file;

    let promotion = if bytes.len() > 4 {
        match bytes[4] {
            b'q' => Some(Piece::Queen),
            b'r' => Some(Piece::Rook),
            b'b' => Some(Piece::Bishop),
            b'n' => Some(Piece::Knight),
            _ => None,
        }
    } else {
        None
    };

    Some(Move {
        from,
        to,
        promotion,
        ..Default::default()
    })
}

fn debug_print(board: &Board) {
    println!("\n +---+---+---+---+---+---+---+---+");
    for rank in (0..8).rev() {
        print!("{}", rank + 1);
        for file in 0..8 {
            let sq = rank * 8 + file;
            let piece_char = match board.piece_at(sq) {
                Some((Piece::Pawn, Color::White)) => 'P',
                Some((Piece::Knight, Color::White)) => 'N',
                Some((Piece::Bishop, Color::White)) => 'B',
                Some((Piece::Rook, Color::White)) => 'R',
                Some((Piece::Queen, Color::White)) => 'Q',
                Some((Piece::King, Color::White)) => 'K',
                Some((Piece::Pawn, Color::Black)) => 'p',
                Some((Piece::Knight, Color::Black)) => 'n',
                Some((Piece::Bishop, Color::Black)) => 'b',
                Some((Piece::Rook, Color::Black)) => 'r',
                Some((Piece::Queen, Color::Black)) => 'q',
                Some((Piece::King, Color::Black)) => 'k',
                None => '.',
            };
            print!("| {} ", piece_char);
        }
        println!("|");
        println!(" +---+---+---+---+---+---+---+---+");
    }
    println!("   a   b   c   d   e   f   g   h\n");
}
