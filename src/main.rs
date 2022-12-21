use chess::{BitBoard, Board, ChessMove, Color, MoveGen, Piece};
use rand::Rng;
use reqwest;
use serde_json::Value;
use std::env;
use std::fs;
use std::ops::BitAnd;
use std::str::FromStr;

/**
 * Reads the Auth Token given by Lichess from the config.json file, which must
 * be included for the bot to work.
 */
fn read_auth_token() -> String {
    let config_str = &fs::read_to_string("config.json").expect("Unable to read config file");
    let json: serde_json::Value =
        serde_json::from_str(&config_str).expect("JSON was not well-formatted");

    let auth = match &json["auth_token"] {
        Value::String(s) => s,
        _ => panic!(),
    };

    return auth.to_string();
}

/**
 * [bitboard_color_piece(b, piece, color, player_white)] returns a BitBoard
 * containing all the instances of [piece] for a particular [color] in the board
 * [b]. The= BitBoard is flipped according to whether the player is white
 * (indicated by [player_white]), with the player's pieces always starting at
 * the bottom of the board.
 */
fn bitboard_color_piece(b: &Board, piece: Piece, color: Color, player_white: bool) -> BitBoard {
    let bitboard_piece = b.pieces(piece);
    let bitboard_color = b.color_combined(color);
    if player_white {
        return bitboard_piece.bitand(bitboard_color).reverse_colors();
    } else {
        return bitboard_piece.bitand(bitboard_color);
    }
}

/**
 * [bitboard_to_vec(bitboard)] converts [bitboard] to a 64-length hot vector
 * containing a 1 for each piece and a 0 for each empty square in the bitboard.
*/
fn bitboard_to_vec(bitboard: &BitBoard) -> Vec<i8> {
    let bitboard_str = bitboard.to_string().replace(" ", "").replace("\n", "");
    let mut vec = Vec::new();
    for i in (0..64) {
        let iter = &bitboard_str[i..i + 1];
        let dig = if iter == "X" { 1 } else { 0 };
        vec.push(dig);
    }

    return vec;
}

/**
 * [get_state(b, player_white)] converts the board [b] into a vector state based
 * on whether the player is white. The state is a concatenated vector of 12
 * bitboard representations, the first 6 of which represent the locations of the
 * 6 different pieces for the player and the last 6 of which represent the
 * locations of the 6 different pieces for the opponent.
 */
fn get_state(b: &Board, player_white: bool) -> Vec<i8> {
    let mut state = Vec::new();

    // White state
    let mut white_state = Vec::new();
    white_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Pawn,
        Color::White,
        player_white,
    )));
    white_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Bishop,
        Color::White,
        player_white,
    )));
    white_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Knight,
        Color::White,
        player_white,
    )));
    white_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Rook,
        Color::White,
        player_white,
    )));
    white_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Queen,
        Color::White,
        player_white,
    )));
    white_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::King,
        Color::White,
        player_white,
    )));

    // Black state
    let mut black_state = Vec::new();
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Pawn,
        Color::White,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Bishop,
        Color::White,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Knight,
        Color::White,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Rook,
        Color::White,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Queen,
        Color::White,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::King,
        Color::White,
        player_white,
    )));

    if player_white {
        state.append(&mut white_state);
        state.append(&mut black_state);
    } else {
        state.append(&mut black_state);
        state.append(&mut white_state);
    }

    println!("{:#?}", state);
    return state;
}

/**
 * [make_random_move(b)] applies a random legal move to board b. If there are no
 * legal moves, it returns (b, None) and does nothing to the board. If there is
 * at least one legal move, it returns (b', Some(m)) where b' is the new board
 * after move m was applied to it.
 */
fn make_random_move(b: Board) -> (Board, Option<ChessMove>) {
    // Generate legal moves
    let mut legal_moves = MoveGen::new_legal(&b);
    if legal_moves.len() == 0 {
        // If no legal moves, do nothing
        return (b, None);
    }

    // Pick a random move
    let next_move = legal_moves.nth(rand::thread_rng().gen_range(0..=legal_moves.len() - 1));

    // Update the board
    let mut result = Board::default();
    let m = match next_move {
        None => panic!(),
        Some(m) => {
            b.make_move(m, &mut result);
            m
        }
    };

    return (result, Some(m));
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Parse auth token from config file
    let auth_token = read_auth_token();

    // Parse game id from command line args
    let args: Vec<String> = env::args().collect();
    let game_id = &args[1];

    // Initialize board and most recent move
    let mut board = Board::default();
    let mut recent_move: Option<ChessMove> = None;

    get_state(&board, true);

    // Create new client to interact with lichess
    let client = reqwest::Client::new();

    Ok(())

    // The game loop
    /*loop {
        // Executes once each pair of moves
        loop {
            // Waiting for my turn

            // Poll json stream of in-game events
            let res = client
                .get("https://lichess.org/api/stream/event")
                .bearer_auth(&auth_token)
                .send()
                .await?
                .chunk()
                .await?;

            // Convert response output into bytes and then json
            let res_bytes = match res {
                None => panic!(),
                Some(b) => b,
            };
            let json: Value = match serde_json::from_slice(&res_bytes) {
                Ok(j) => j,
                Err(_) => panic!(),
            };

            // Check if my turn
            let my_turn = match &json["game"]["isMyTurn"] {
                Value::Bool(b) => *b,
                _ => panic!(),
            };
            if my_turn {
                // Update board from FEN string and exit wait loop
                board = match &json["game"]["fen"] {
                    Value::String(s) => match Board::from_str(s) {
                        Ok(b) => b,
                        Err(_) => panic!(),
                    },
                    _ => panic!(),
                };
                break;
            } else {
                println!("Waiting for my turn!");
            }
        }

        // Select a move
        println!("Making Move!");
        let x = make_random_move(board);
        match x {
            (b, m) => {
                board = b;
                recent_move = m
            }
        };
        let uci_str = match recent_move {
            None => panic!(),
            Some(m) => m.to_string(),
        };
        println!("Selected move {}", uci_str);

        // Post move
        client
            .post("https://lichess.org/api/bot/game/".to_owned() + game_id + "/move/" + &uci_str)
            .bearer_auth(&auth_token)
            .send()
            .await?;
    }*/
}
