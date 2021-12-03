use std::cmp;
use std::fmt;
const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
fn main() {
    let b = board_from_fen("6r1/6pp/7r/1B5K/1P3k2/N7/3R4/8 w - - 30 79");
    println!("{}", b);
    //println!("{:?}", b.board);
}

// piece identifiers. black pawn := BLACK + PAWN. piece_color = (piece/8) * 8; piece_type = piece % 8;
const EMPTY: u8 = 0;
const PAWN: u8 = 1;
const KNIGHT: u8 = 2;
const BISHOP: u8 = 3;
const ROOK: u8 = 4;
const QUEEN: u8 = 5;
const KING: u8 = 6;

const WHITE: u8 = 1;
const BLACK: u8 = 2;

fn board_from_fen(fen: &str) -> Board {
    let mut board_idx = 0;
    let mut fen_idx = 0;
    let mut board_arr = [Piece::new(EMPTY, WHITE); 64];

    // setup board
    for (i, c) in fen.chars().enumerate() {
        if c.is_whitespace() {
            fen_idx = i + 1;
            break;
        }
        if c.is_digit(10) {
            board_idx += c.to_digit(10).unwrap();
        } else if c.is_alphabetic() {
            let mut color = 0;
            let mut c_low = '0';
            if c.is_uppercase() {
                color = WHITE;
                c_low = c.to_ascii_lowercase();
            } else {
                color = BLACK;
                c_low = c;
            }
            board_arr[board_idx as usize] = match c_low {
                'p' => Piece::new(PAWN, color),
                'n' => Piece::new(KNIGHT, color),
                'b' => Piece::new(BISHOP, color),
                'r' => Piece::new(ROOK, color),
                'q' => Piece::new(QUEEN, color),
                'k' => Piece::new(KING, color),
                _ => Piece::new(EMPTY, WHITE),
            };
            board_idx += 1;
        } else if c == '\\' {
            board_idx -= 16;
        }
    }

    // get player turn
    let color_to_move = if fen.chars().nth(fen_idx).unwrap() == 'w' {
        WHITE
    } else {
        BLACK
    };
    fen_idx += 2;

    // rochade
    let mut castle_w_k = false;
    let mut castle_w_q = false;
    let mut castle_b_k = false;
    let mut castle_b_q = false;
    if fen.chars().nth(fen_idx).unwrap() == 'K' {
        castle_w_k = true;
        fen_idx += 1;
    }
    if fen.chars().nth(fen_idx).unwrap() == 'Q' {
        castle_w_q = true;
        fen_idx += 1;
    }
    if fen.chars().nth(fen_idx).unwrap() == 'k' {
        castle_b_k = true;
        fen_idx += 1;
    }
    if fen.chars().nth(fen_idx).unwrap() == 'q' {
        castle_b_q = true;
        fen_idx += 1;
    }
    if fen.chars().nth(fen_idx).unwrap() == '-' {
        fen_idx += 1;
    }
    fen_idx += 1;

    // en passant
    let mut en_passant_idx = 64;
    if fen.chars().nth(fen_idx).unwrap() == '-' {
        fen_idx += 2;
    } else {
        let file = fen.chars().nth(fen_idx).unwrap();
        let rank = fen.chars().nth(fen_idx + 1).unwrap();
        let mut field = String::from(file);
        field.push(rank);
        en_passant_idx = f2i(&field);
        fen_idx += 3
    }

    // plies / half-moves
    let mut plies = 0;
    while !fen.chars().nth(fen_idx).unwrap().is_whitespace() {
        plies *= 10;
        let d = fen.chars().nth(fen_idx).unwrap().to_digit(10).unwrap();
        plies += d;
        fen_idx += 1;
    }
    fen_idx += 1;
    // move nr.
    let mut move_nr = 0;
    while fen_idx < fen.chars().count() && !fen.chars().nth(fen_idx).unwrap().is_whitespace() {
        move_nr *= 10;
        let d = fen.chars().nth(fen_idx).unwrap().to_digit(10).unwrap();
        move_nr += d;
        fen_idx += 1;
    }

    Board {
        board: board_arr,
        color_to_move: color_to_move,
        castling_rights: [castle_w_k, castle_w_q, castle_b_k, castle_b_q],
        en_passant_possible: en_passant_idx != 64,
        en_passant_idx: en_passant_idx,
        plies: plies,
        move_nr: move_nr,
    }
}
// field to index
fn f2i(field: &str) -> usize {
    let file = field.chars().nth(0).unwrap();
    let rank = field.chars().nth(1).unwrap();
    let x = file as usize - 'a' as usize;
    let y = (rank.to_digit(10).unwrap() - 1) as usize;
    x + y * 8
}

//index to field
fn i2f(index: u8) -> String {
    let file = (97 + index % 8) as char;
    let rank = char::from_digit((index / 8).into(), 10).unwrap();
    let mut f = String::from(file);
    f.push(rank);
    f
}

fn is_opponent(piece1: &Piece, piece2: &Piece) -> bool {
    piece1.color + piece2.color == 3
}

#[derive(Copy, Clone, Debug)]
struct Piece {
    piece: u8,
    color: u8,
    weak_to_en_passant: bool,
    has_moved: bool,
}

impl Piece {
    fn new(piece: u8, color: u8) -> Piece {
        Piece {
            piece: piece,
            color: color,
            weak_to_en_passant: false,
            has_moved: false,
        }
    }

    fn print(&self) -> String {
        let out = match self.piece {
            PAWN => "P",
            KNIGHT => "N",
            BISHOP => "B",
            ROOK => "R",
            QUEEN => "Q",
            KING => "K",
            _ => " ",
        };
        let mut out = String::from(out);
        if self.color == BLACK && self.piece != EMPTY {
            out = out.to_lowercase();
        }
        out
    }
}

// top, bot, left, right, top_right, bot_right, bot_left, top_left
const DIRECTION_OFFSETS: [i8; 8] = [8, -8, -1, 1, 9, -7, -9, 7];
const DIR_TOP: usize = 0;
const DIR_BOT: usize = 1;
const DIR_LEFT: usize = 2;
const DIR_RIGHT: usize = 3;
const DIR_TOP_RIGHT: usize = 4;
const DIR_BOT_RIGHT: usize = 5;
const DIR_BOT_LEFT: usize = 6;
const DIR_TOP_LEFT: usize = 7;

// THE BOARD
// a list of 64 fields, index 0 is a1, 1 is a2, 8 is b1, etc. Bottom left to top right.
// field offsets: up = +8, down = -8, left = -1, right = +1, top_right = +9, top_left = +7, bottom_left = -9, bottom_right = -7

// get an array that contains for every field a tuple of distances to the different edges
fn get_num_squares_to_edge() -> [[u8; 8]; 64] {
    // init empty array
    let mut array = [[0u8; 8]; 64];

    // for every square on the chess board, get distances, pack into struct, and put into array
    for file in 0u8..8 {
        for rank in 0u8..8 {
            let top = 7 - rank;
            let bot = rank;
            let left = file;
            let right = 7 - file;
            let top_right = cmp::min(top, right);
            let top_left = cmp::min(top, left);
            let bot_right = cmp::min(bot, right);
            let bot_left = cmp::min(bot, left);

            let squareIndex = (rank * 8 + file) as usize;

            array[squareIndex] = [
                top, bot, left, right, top_right, bot_right, bot_left, top_left,
            ];
        }
    }
    // return array
    array
}

struct Board {
    board: [Piece; 64],
    color_to_move: u8,
    castling_rights: [bool; 4],
    en_passant_possible: bool,
    en_passant_idx: usize,
    plies: u32,
    move_nr: u32,
}

impl Board {
    fn to_fen(&self) -> String {
        String::new()
    }

    fn game_over(&self) -> bool {
        false
    }

    fn is_check(&self) {}

    fn is_mate(&self) {}

    fn is_draw(&self) {}

    fn attack_map(&self) {}

    fn can_promote(&self) {}

    fn promote(&self, piece_type: u8) {}

    fn make_move(&self, m: Move) -> Piece {
        Piece::new(EMPTY, WHITE)
    }

    fn unmake_move(&self, m: Move, captured_piece: Piece) {}

    // check whether the field is attacked by a piece of enemy_color
    fn is_attacked(&self, field: usize, enemy_color: u8) {}
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut repr = String::new();
        for row in 0..8 {
            let row_inv = row;
            repr.push_str("-----------------\n");
            for col in 0..8 {
                repr.push_str("|");
                repr.push_str(&self.board[row_inv * 8 + col].print());
            }
            repr.push_str("|\n");
        }
        repr.push_str("-----------------\n");
        let player = if self.color_to_move == WHITE {
            "White"
        } else {
            "Black"
        };

        let mut en_passant = String::new();

        write!(f, "{}Move #{}\n{} to move\nCastling rights [K,Q,k,q]: {:?}\nPlies since last pawn move or capture: {}", repr, self.move_nr, player, self.castling_rights,self.plies)
    }
}

// piece-centric board representation. for every piece type, contains one 64 bit number that encodes the locations of
// that piece type on the board (1 bit if field occupied by piece type, 0 bit if not)
struct bitboard {
    // white
    wp: u64,
    wn: u64,
    wb: u64,
    wr: u64,
    wq: u64,
    wk: u64,
    // black
    bp: u64,
    bn: u64,
    bb: u64,
    br: u64,
    bq: u64,
    bk: u64,
}

// MOVES
#[derive(Copy, Clone)]
struct Move {
    start: u8,
    target: u8,
}

fn play() {
    let mut board = board_from_fen(STARTING_FEN);
    while !board.game_over() {
        board.make_move(Move {
            start: 12,
            target: 20,
        });
    }
}

fn generate_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::new();
    for start_square in 0..64 {
        let piece = board.board[start_square];
        let mut new_moves = Vec::new();
        if piece.color == board.color_to_move {
            if piece.piece == PAWN {
                new_moves = generate_pawn_moves(board, start_square, &piece);
            } else if piece.piece == KNIGHT {
                new_moves = generate_knight_moves(board, start_square, &piece);
            } else {
                new_moves = generate_sliding_moves(board, start_square, &piece);
            }
        }
        // filter illegal moves
        moves.append(&mut new_moves);
    }
    moves
}

fn generate_pawn_moves(board: &Board, start_square: usize, piece: &Piece) -> Vec<Move> {
    let mut moves = Vec::new();
    let squaresToEdge = get_num_squares_to_edge();
    let mut move_offset: i8 = 0;
    let mut attack_offsets: [i8; 2] = [0, 0];
    if piece.color == WHITE {
        move_offset = DIRECTION_OFFSETS[DIR_TOP];
        attack_offsets = [
            DIRECTION_OFFSETS[DIR_TOP_LEFT],
            DIRECTION_OFFSETS[DIR_TOP_RIGHT],
        ];
    } else {
        move_offset = DIRECTION_OFFSETS[DIR_BOT];
        attack_offsets = [
            DIRECTION_OFFSETS[DIR_BOT_LEFT],
            DIRECTION_OFFSETS[DIR_BOT_RIGHT],
        ];
    }
    let target_square = ((start_square as i8) + move_offset) as usize;
    let start = start_square as u8;
    let target = target_square as u8;
    if board.board[target_square].piece == EMPTY {
        moves.push(Move { start, target });
        // if pawn hasn't moved yet and 2 squares in front of it are free, can also directly go 2 steps
        if !piece.has_moved {
            let target_square = ((target_square as i8) + move_offset) as usize;
            let target = target_square as u8;
            if board.board[target_square].piece == EMPTY {
                moves.push(Move { start, target });
            }
        }
    }
    // check if there is an opponent piece diagonally in front. if yes, can capture it
    for offset in attack_offsets {
        let target_square = ((start_square as i8) + offset) as usize;
        let target_piece = board.board[target_square];
        if is_opponent(piece, &target_piece) {
            let start = start_square as u8;
            let target = target_square as u8;
            moves.push(Move { start, target });
        }
    }
    // check for possible en passants
    let piece_right = board.board[((start_square as i8) + DIRECTION_OFFSETS[DIR_RIGHT]) as usize];
    let piece_left = board.board[((start_square as i8) + DIRECTION_OFFSETS[DIR_LEFT]) as usize];
    if is_opponent(piece, &piece_left) && piece_left.piece == PAWN && piece_left.weak_to_en_passant
    {
        let target_square = ((start_square as i8) + attack_offsets[0]) as usize;
        let start = start_square as u8;
        let target = target_square as u8;
        moves.push(Move { start, target });
    }
    if is_opponent(piece, &piece_right)
        && piece_right.piece == PAWN
        && piece_right.weak_to_en_passant
    {
        let target_square = ((start_square as i8) + attack_offsets[1]) as usize;
        let start = start_square as u8;
        let target = target_square as u8;
        moves.push(Move { start, target });
    }
    moves
}

fn generate_knight_moves(board: &Board, start_square: usize, piece: &Piece) -> Vec<Move> {
    let mut moves = Vec::new();
    let squaresToEdge = get_num_squares_to_edge();
    let coord_offsets: [(i8, i8); 8] = [
        (1, 2),
        (2, 1),
        (-1, 2),
        (2, -1),
        (-1, -2),
        (-2, -1),
        (1, -2),
        (-2, 1),
    ];

    for co in coord_offsets {
        if squaresToEdge[start_square][DIR_TOP] as i8 > co.0
            && squaresToEdge[start_square][DIR_BOT] as i8 > -co.0
            && squaresToEdge[start_square][DIR_RIGHT] as i8 > co.1
            && squaresToEdge[start_square][DIR_LEFT] as i8 > -co.1
        {
            let target_square = ((start_square as i8) + co.0 + co.1 * 8) as usize;
            let target_piece = board.board[target_square];
            if target_piece.piece != EMPTY && !is_opponent(piece, &target_piece) {
                continue;
            }
            let start = start_square as u8;
            let target = target_square as u8;
            moves.push(Move { start, target })
        }
    }

    moves
}

fn generate_sliding_moves(board: &Board, start_square: usize, piece: &Piece) -> Vec<Move> {
    // bishops only have access to the last 4 directions, rooks only to the first 4
    let start_direction_index = if piece.piece == BISHOP { 4 } else { 0 };
    let end_direction_index = if piece.piece == ROOK { 4 } else { 8 };

    let mut moves = Vec::new();
    let squaresToEdge = get_num_squares_to_edge();

    // go through directions clockwise
    for direction in start_direction_index..end_direction_index {
        let range = if piece.piece == KING {
            1
        } else {
            squaresToEdge[start_square][direction]
        };
        for n in 0..range {
            let target_square =
                ((start_square as i8) + DIRECTION_OFFSETS[direction] * ((n as i8) + 1)) as usize;
            let piece_on_target_square = board.board[target_square];
            let start = start_square as u8;
            let target = target_square as u8;
            // if same colored piece is on field, can't go further in this direction
            if piece_on_target_square.color == piece.color {
                break;
            }
            // if field is empty, can move there and further
            // if opponent piece is on the field, capture it but cannot go further

            moves.push(Move { start, target });
            if is_opponent(piece, &piece_on_target_square) {
                break;
            }
        }
    }

    moves
}
