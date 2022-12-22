use chess::{BitBoard, Board, BoardStatus, Color, Piece, Square};
use std::ops::BitAnd;
use std::str::FromStr;

/*
Struct to represent the experience of the RL agent at one time-step (i.e. move)
*/
struct Experience {
    state: Vec<i8>,
    action: Vec<i8>,
    reward: i8,
    next_state: Vec<i8>,
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
        return bitboard_piece.bitand(bitboard_color);
    } else {
        return bitboard_piece.bitand(bitboard_color).reverse_colors();
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
pub fn get_state(b: &Board, player_white: bool) -> Vec<i8> {
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
        Color::Black,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Bishop,
        Color::Black,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Knight,
        Color::Black,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Rook,
        Color::Black,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::Queen,
        Color::Black,
        player_white,
    )));
    black_state.append(&mut bitboard_to_vec(&bitboard_color_piece(
        b,
        Piece::King,
        Color::Black,
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
* [vec_from_board_square(square_str, player_white)] converts the bitboard with
* only the square represented by [square_str] into a vector based on whether
* the player is white. This is used in action representation for representing
* bitboards of initial and final positions of a piece.
*/
fn vec_from_board_square(square_str: &str, player_white: bool) -> Vec<i8> {
    let square = match Square::from_str(square_str) {
        Ok(sq) => sq,
        Err(_) => panic!(),
    };
    let square_bitboard = if player_white {
        BitBoard::from_square(square)
    } else {
        BitBoard::from_square(square).reverse_colors()
    };

    return bitboard_to_vec(&square_bitboard);
}

/**
* [get_action(uci_str, player_white)] converts the move represented by the
* [uci_str] into an action vector based on whether the player is white. The
* action is a concatenated vector of two bitboard representations, the first of
* which being the initial position of the moved piece and the second of which
* being the final position of the moved piece, along with a final 4 dimensional
* hot vector representing the promoted-to piece if a promotion occured.
*/
pub fn get_action(uci_str: &str, player_white: bool) -> Vec<i8> {
    // Parse uci string
    let init_str = &uci_str[0..2];
    let final_str = &uci_str[2..4];
    let promote_str = if uci_str.len() > 4 {
        &uci_str[4..5]
    } else {
        ""
    };

    // Initialize empty action vector
    let mut action = Vec::new();

    // Convert initial and final position into vectors
    let mut init_pos = vec_from_board_square(init_str, player_white);
    action.append(&mut init_pos);
    let mut final_pos = vec_from_board_square(final_str, player_white);
    action.append(&mut final_pos);

    // Handle promotion vector possibilities
    let mut promotion = Vec::new();
    if promote_str.eq("") {
        promotion = vec![0, 0, 0, 0];
    } else if promote_str.eq("b") {
        promotion = vec![1, 0, 0, 0];
    } else if promote_str.eq("n") {
        promotion = vec![0, 1, 0, 0];
    } else if promote_str.eq("r") {
        promotion = vec![0, 0, 1, 0];
    } else {
        // queen
        promotion = vec![0, 0, 0, 1];
    }
    action.append(&mut promotion);

    println!("{:#?}", action);

    return action;
}

/**
* [get_reward(b, player_white)] returns the reward of a certain board state
* depending on whether the player is white. Ongoing games and stalemates give
* 0 reward, whereas winning/losing via checkmate provides 1 or -1 reward
* respectively.
* Note: this function will eventually probably base itself on response from the
* Lichess API to handle situations like draw or win via resign.
*/
fn get_reward(b: &Board, player_white: bool) -> i8 {
    match b.status() {
        BoardStatus::Ongoing | BoardStatus::Stalemate => 0,
        BoardStatus::Checkmate => {
            if player_white {
                if b.side_to_move() == Color::Black {
                    1
                } else {
                    -1
                }
            } else {
                if b.side_to_move() == Color::White {
                    1
                } else {
                    -1
                }
            }
        }
    }
}
