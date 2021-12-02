use std::cmp;

fn main() {
    let board = bitboard {
        wp: 0,
        wn: 0,
        wb: 0,
        wr: 0,
        wq: 0,
        wk: 0,
        bp: 0,
        bn: 0,
        bb: 0,
        br: 0,
        bq: 0,
        bk: 0,
    };
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

fn is_opponent(piece1: &Piece, piece2: &Piece) -> bool {
    piece1.color + piece2.color == 3
}

#[derive(Copy, Clone)]
struct Piece {
    piece: u8,
    color: u8,
    weak_to_en_passant: bool,
    has_moved: bool,
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

fn generate_moves(board: &Board) -> Vec<Move> {
    let moves = Vec::new();
    for start_square in 0..64 {
        let piece = board.board[start_square];
        if piece.color == board.color_to_move {
            generate_sliding_moves(board, start_square, &piece);
        }
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
    // check for possible en_passants
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
        {}
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
        for n in 0..squaresToEdge[start_square][direction] {
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
