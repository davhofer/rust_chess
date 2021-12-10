use chess::{
    self, BitBoard, Board, BoardStatus, ChessMove, File, Game, MoveGen, Piece, Rank, Square,
};
use std::cmp;
use std::str::FromStr;
use std::usize;

const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/////////////////////////////////////////////////////////////////////////////////
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event;
use ggez::filesystem;
use ggez::graphics::{self, Color};
use ggez::input::{keyboard, mouse};
use ggez::timer;
use ggez::{Context, GameResult};
use std::env;
use std::io::{stdin, stdout, Write};
use std::path;

const WINDWOW_SIZE: f32 = 800.;

fn canvas_square_to_board_square(square: (i16, i16), pov: u8) -> Square {
    let (x, y) = (square.0 as usize, square.1 as usize);
    let (x, y) = if pov == 1 { (x, 7 - y) } else { (7 - x, y) };
    Square::make_square(Rank::from_index(y), File::from_index(x))
}

fn board_square_to_canvas_square(square: &Square, pov: u8) -> (f32, f32) {
    let idx = Square::to_int(square);
    if pov == 1 {
        ((idx % 8) as f32, 7. - (idx / 8) as f32)
    } else {
        (7. - (idx % 8) as f32, (idx / 8) as f32)
    }
}

struct MainState {
    pov: u8,
    flip_timeout: u16,
    field_selected: bool,
    field: (i16, i16),
    game: Game,
    current_legal_moves: Vec<ChessMove>,
    player_color: u8,
    bot: Bot,
}
fn canvas_coord_to_canvas_square(x: i16, y: i16, pov: u8) -> (i16, i16) {
    let file = x / (WINDWOW_SIZE as i16 / 8);
    let rank = y / (WINDWOW_SIZE as i16 / 8);

    (file, rank)
}

fn movegen_empty() -> Vec<ChessMove> {
    let game: Game = Game::from_str(STARTING_FEN).expect("Valid FEN");
    let mut empty = MoveGen::new_legal(&game.current_position());
    empty.remove_mask(BitBoard::new(u64::MAX));
    empty.collect()
}

fn movegen(board: &Board, start_square: Square, color_to_move: chess::Color) -> Vec<ChessMove> {
    match board.color_on(start_square) {
        None => movegen_empty(),
        Some(color) => MoveGen::new_legal(board)
            .filter(|m| m.get_source() == start_square)
            .collect(),
    }
}

impl MainState {
    fn new(ctx: &mut Context, player_color: u8) -> GameResult<MainState> {
        //filesystem::print_all(ctx);

        let rng = oorandom::Rand32::new(271828);
        let game: Game = Game::from_str(STARTING_FEN).expect("Valid FEN");
        let bot_color = if player_color == 1 {
            chess::Color::Black
        } else {
            chess::Color::White
        };

        let s = MainState {
            pov: player_color,
            flip_timeout: 0,
            field_selected: false,
            field: (-1, -1),
            game,
            current_legal_moves: movegen_empty(),
            player_color,
            bot: Bot::new(bot_color),
        };

        Ok(s)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if self.flip_timeout == 0 && keyboard::is_key_pressed(ctx, event::KeyCode::F) {
            self.pov = (self.pov % 2) + 1;
            self.flip_timeout = 10;
            self.current_legal_moves = movegen_empty();
            self.field_selected = false;
        }
        if mouse::button_pressed(ctx, mouse::MouseButton::Left) {
            if self.game.side_to_move() != self.bot.color {
                let canvas_square_clicked = canvas_coord_to_canvas_square(
                    mouse::position(ctx).x as i16,
                    mouse::position(ctx).y as i16,
                    self.pov,
                );
                if self.field != canvas_square_clicked {
                    if !self.field_selected {
                        let square = canvas_square_to_board_square(canvas_square_clicked, self.pov);
                        self.current_legal_moves = movegen(
                            &self.game.current_position(),
                            square,
                            self.game.side_to_move(),
                        );
                        self.field = canvas_square_clicked;
                        self.field_selected = true;
                    } else if self.field_selected {
                        let start_square = canvas_square_to_board_square(self.field, self.pov);
                        let target_square =
                            canvas_square_to_board_square(canvas_square_clicked, self.pov);
                        let (is_piece_1, piece1) =
                            match self.game.current_position().piece_on(start_square) {
                                Some(x) => (true, x),
                                None => (false, Piece::Pawn),
                            };
                        let (is_piece_2, _) =
                            match self.game.current_position().piece_on(target_square) {
                                Some(x) => (true, x),
                                None => (false, Piece::Pawn),
                            };
                        let board = self.game.current_position();
                        if is_piece_1
                            && (board.color_on(start_square) != board.color_on(target_square)
                                || !is_piece_2)
                            && Some(self.game.side_to_move()) == board.color_on(start_square)
                            && self
                                .current_legal_moves
                                .iter()
                                .filter(|m| m.get_dest() == target_square)
                                .count()
                                > 0
                        {
                            // TODO: special case for promotions
                            let prom = if piece1 == Piece::Pawn
                                && (target_square.get_rank() == Rank::First
                                    || target_square.get_rank() == Rank::Eighth)
                            {
                                Some(Piece::Queen)
                            } else {
                                None
                            };
                            self.game
                                .make_move(ChessMove::new(start_square, target_square, prom));
                            self.field_selected = false;
                            self.current_legal_moves = movegen_empty();
                        } else {
                            self.field = canvas_square_clicked;
                            self.current_legal_moves =
                                movegen(&board, target_square, self.game.side_to_move());
                        }
                    }
                }
            } else {
                self.game
                    .make_move(self.bot.get_move(self.game.current_position()));
            }
            if mouse::button_pressed(ctx, mouse::MouseButton::Right) {
                self.field_selected = false;
                self.field = (-1, -1);
                self.current_legal_moves = movegen_empty();
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // since draw gets constantly called, use this to decrease the timeout
        if self.flip_timeout > 0 {
            self.flip_timeout -= 1;
        }

        let tile_size = (WINDWOW_SIZE as u32 / 8) as f32;
        graphics::clear(ctx, [1., 1., 1., 1.0].into());
        let color_to_move = self.game.side_to_move();
        let king_square = board_square_to_canvas_square(
            &self.game.current_position().king_square(color_to_move),
            self.pov,
        );
        for x in 0..8 {
            for y in 0..8 {
                let color = if self.field == (x, y) && self.field_selected {
                    Color::from((240, 60, 140, 255))
                } else if self
                    .current_legal_moves
                    .iter()
                    .filter(|m| {
                        m.get_dest()
                            == canvas_square_to_board_square((x as i16, y as i16), self.pov)
                    })
                    .count()
                    > 0
                {
                    Color::from((200, 80, 80, 255))
                } else if self.game.current_position().checkers().popcnt() > 0
                    && (x as f32, y as f32) == king_square
                {
                    Color::from((100, 6, 5, 255))
                } else if (x + y) % 2 == 0 {
                    Color::from((200, 200, 200, 255))
                } else {
                    Color::from((50, 50, 50, 255))
                };

                let square = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    graphics::Rect::new(
                        x as f32 * tile_size,
                        y as f32 * tile_size,
                        tile_size,
                        tile_size,
                    ),
                    color,
                )?;
                graphics::draw(ctx, &square, (glam::Vec2::new(0.0, 0.0),))?;
            }
        }

        // const EMPTY: u8 = 0;
        // const PAWN: u8 = 1;
        // const KNIGHT: u8 = 2;
        // const BISHOP: u8 = 3;
        // const ROOK: u8 = 4;
        // const QUEEN: u8 = 5;
        // const KING: u8 = 6;

        // const WHITE: u8 = 1;
        // const BLACK: u8 = 2;
        let mut piece_imgs: [[graphics::Image; 6]; 2] = [
            [
                graphics::Image::new(ctx, "/Chess_plt60.png")?,
                graphics::Image::new(ctx, "/Chess_nlt60.png")?,
                graphics::Image::new(ctx, "/Chess_blt60.png")?,
                graphics::Image::new(ctx, "/Chess_rlt60.png")?,
                graphics::Image::new(ctx, "/Chess_qlt60.png")?,
                graphics::Image::new(ctx, "/Chess_klt60.png")?,
            ],
            [
                graphics::Image::new(ctx, "/Chess_pdt60.png")?,
                graphics::Image::new(ctx, "/Chess_ndt60.png")?,
                graphics::Image::new(ctx, "/Chess_bdt60.png")?,
                graphics::Image::new(ctx, "/Chess_rdt60.png")?,
                graphics::Image::new(ctx, "/Chess_qdt60.png")?,
                graphics::Image::new(ctx, "/Chess_kdt60.png")?,
            ],
        ];
        let img_size = piece_imgs[0][0].width() as f32;
        let offset = (tile_size - img_size) / 2.0;
        let board = self.game.current_position();
        for i in 0..8 {
            for j in 0..8 {
                let square = canvas_square_to_board_square((i as i16, j as i16), self.pov);
                if let Some(piece) = board.piece_on(square) {
                    let color = if board.color_on(square) == Some(chess::Color::White) {
                        0
                    } else {
                        1
                    };
                    let dest_point = glam::Vec2::new(
                        i as f32 * tile_size + offset,
                        j as f32 * tile_size + offset,
                    );
                    //let scale = glam::Vec2::new(1.3, 1.3);
                    let param = graphics::Rect::new(
                        i as f32 * tile_size + tile_size * 0.1,
                        j as f32 * tile_size + tile_size * 0.1,
                        tile_size - (tile_size * 0.2),
                        tile_size - (tile_size * 0.2),
                    );
                    graphics::draw(
                        ctx,
                        &piece_imgs[color as usize][piece.to_index() as usize],
                        (dest_point,),
                    )?;
                }
            }
        }

        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("assets");
        path
    } else {
        path::PathBuf::from("./assets")
    };

    let cb = ggez::ContextBuilder::new("Chess", "davhofer")
        .add_resource_path(resource_dir)
        .window_setup(WindowSetup {
            title: "Chess".to_string(),
            icon: "/Chess_rlt60.png".to_string(),
            ..WindowSetup::default()
        })
        .window_mode(WindowMode {
            width: WINDWOW_SIZE,
            height: WINDWOW_SIZE,
            resizable: false,
            ..WindowMode::default()
        });
    let (mut ctx, event_loop) = cb.build()?;

    let player_color = 2;

    let state = MainState::new(&mut ctx, player_color)?;
    event::run(ctx, event_loop, state)
}

struct Bot {
    color: chess::Color,
    objective: i32,
}

impl Bot {
    fn new(color: chess::Color) -> Bot {
        Bot {
            color,
            objective: if color == chess::Color::White { 1 } else { -1 },
        }
    }

    fn eval(&self, board: &Board) -> i32 {
        evaluate(board)
    }

    fn get_move(&self, board: Board) -> ChessMove {
        let depth = 3;
        println!("Searching for move...");
        let (pos_score, best_move) =
            self.negamax_ab(board, depth, i32::MIN + 2, i32::MAX - 2, self.objective); //self.negamax(board, depth, self.objective);
        println!("Score for current position: {}", pos_score);
        if let Some(m) = best_move {
            println!("Move chosen: {:?}", m);
            m
        } else {
            println!("No move possible!");
            ChessMove::new(Square::A1, Square::A2, None)
        }
    }

    fn negamax(&self, board: Board, depth: u8, player_obj: i32) -> (i32, Option<ChessMove>) {
        if depth == 0 || board.status() != BoardStatus::Ongoing {
            return (player_obj * self.eval(&board), None);
        }
        let mut score = i32::MIN;
        let mut best_move = None;
        for m in MoveGen::new_legal(&board) {
            let new_board = board.make_move_new(m);
            let (child_score, child_move) = self.negamax(new_board, depth - 1, -player_obj);
            let child_score = -child_score;
            if child_score > score {
                score = child_score;
                best_move = Some(m);
            }
        }
        (score, best_move)
    }

    fn negamax_ab(
        &self,
        board: Board,
        depth: u8,
        alpha: i32,
        beta: i32,
        player_obj: i32,
    ) -> (i32, Option<ChessMove>) {
        if depth == 0 || board.status() != BoardStatus::Ongoing {
            return (player_obj * self.eval(&board), None);
        }

        let child_nodes = MoveGen::new_legal(&board);
        // order the moves
        let mut score = i32::MIN;
        let mut best_move = None;
        for m in child_nodes {
            let new_board = board.make_move_new(m);
            let (child_score, child_move) =
                self.negamax_ab(new_board, depth - 1, -beta, -alpha, -player_obj);
            let child_score = -child_score;
            if child_score > score {
                score = child_score;
                best_move = Some(m);
            }

            let alpha = cmp::max(alpha, child_score);
            if alpha > beta {
                break;
            }
        }
        (score, best_move)
    }
}

fn evaluate(board: &Board) -> i32 {
    // can learn the parameters with a python neural network?

    // material value + balance & other material considerations, adjusting piece values based on game state
    // piece square tables
    // pawn structure
    // mobility
    // center control
    // king safety
    // special piece patterns (fianchetto, outposts, etc)
    // connectivity
    // protectivity of pieces
    // trapped pieces
    // space
    // tempo
    // danger levels
    // attacking
    // stuff like forks
    // hanging pieces

    // material value
    let pawn = 10;
    let bishop = 30;
    let knight = 30;
    let rook = 50;
    let queen = 90;

    let no_pawns_penalty = -5;
    let bishoppair = 5;
    let knightpair = -1;
    let rookpair = -1;
    let white_pieces = board.color_combined(chess::Color::White);
    let black_pieces = board.color_combined(chess::Color::Black);
    let mut mat_white = 0;
    let mut mat_black = 0;

    //     Piece::Pawn
    let pawns = board.pieces(Piece::Pawn);
    let pawns_w = (pawns & white_pieces).popcnt() as i32;
    let pawns_b = (pawns & black_pieces).popcnt() as i32;
    mat_white += pawns_w * pawn + if pawns_w == 0 { no_pawns_penalty } else { 0 };
    mat_black += pawns_b * pawn + if pawns_b == 0 { no_pawns_penalty } else { 0 };
    //     Piece::Knight
    let knights = board.pieces(Piece::Knight);
    let knight_w = (knights & white_pieces).popcnt() as i32;
    let knight_b = (knights & black_pieces).popcnt() as i32;
    mat_white += knight_w * knight + if knight_w == 2 { knightpair } else { 0 };
    mat_black += knight_b * knight + if knight_b == 2 { knightpair } else { 0 };
    //     Piece::Bishop
    let bishops = board.pieces(Piece::Bishop);
    let bishops_w = (bishops & white_pieces).popcnt() as i32;
    let bishops_b = (bishops & black_pieces).popcnt() as i32;
    // bonus for bishoppair
    mat_white += bishops_w * bishop + if bishops_w == 2 { bishoppair } else { 0 };
    mat_black += bishops_b * bishop + if bishops_b == 2 { bishoppair } else { 0 };
    //     Piece::Rook
    let rooks = board.pieces(Piece::Rook);
    let rooks_w = (rooks & white_pieces).popcnt() as i32;
    let rooks_b = (rooks & black_pieces).popcnt() as i32;
    // malus for rook pair (redundancy)
    mat_white += rooks_w * rook + if rooks_w == 2 { rookpair } else { 0 };
    mat_black += rooks_b * rook + if rooks_b == 2 { rookpair } else { 0 };
    //
    //     Piece::Queen
    let queens = board.pieces(Piece::Queen);
    mat_white += (queens & white_pieces).popcnt() as i32 * queen;
    mat_black += (queens & black_pieces).popcnt() as i32 * queen;
    let mat_score = mat_white - mat_black;

    // possible moves
    let current_player_moves = MoveGen::new_legal(board).len();

    mat_white - mat_black
}
