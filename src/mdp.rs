/**
 * Utility module for handling conversion of Chess into an MDP (Markov Decision
 * Process)
 */
use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square};
use neuroflow::FeedForward;
use std::ops::BitAnd;
use std::str::FromStr;

// Struct to represent the experience of the bot at one time-step (i.e. move)
#[derive(Clone, Debug)]
pub struct Experience {
    pub state: Vec<f64>,
    pub action: Vec<f64>,
    pub reward: f64,
    pub next_state: Vec<f64>,
    pub next_board: Board,
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
fn bitboard_to_vec(bitboard: &BitBoard) -> Vec<f64> {
    let bitboard_str = bitboard.to_string().replace(" ", "").replace("\n", "");
    let mut vec = Vec::new();
    for i in 0..64 {
        let iter = &bitboard_str[i..i + 1];
        let dig = if iter == "X" { 1. } else { 0. };
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
pub fn get_state(b: &Board, player_white: bool) -> Vec<f64> {
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

    return state;
}

/**
* [vec_from_board_square(square_str, player_white)] converts the bitboard with
* only the square represented by [square_str] into a vector based on whether
* the player is white. This is used in action representation for representing
* bitboards of initial and final positions of a piece.
*/
fn vec_from_board_square(square_str: &str, player_white: bool) -> Vec<f64> {
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
pub fn get_action(uci_str: &str, player_white: bool) -> Vec<f64> {
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
        promotion = vec![0., 0., 0., 0.];
    } else if promote_str.eq("b") {
        promotion = vec![1., 0., 0., 0.];
    } else if promote_str.eq("n") {
        promotion = vec![0., 1., 0., 0.];
    } else if promote_str.eq("r") {
        promotion = vec![0., 0., 1., 0.];
    } else {
        // queen
        promotion = vec![0., 0., 0., 1.];
    }
    action.append(&mut promotion);

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
pub fn get_reward(b: &Board, player_white: bool) -> f64 {
    match b.status() {
        BoardStatus::Ongoing | BoardStatus::Stalemate => 0.,
        BoardStatus::Checkmate => {
            if player_white {
                if b.side_to_move() == Color::Black {
                    100.
                } else {
                    -100.
                }
            } else {
                if b.side_to_move() == Color::White {
                    100.
                } else {
                    -100.
                }
            }
        }
    }
}

/**
 * [compute_q_max(b, state, q_network, player_white)] computes the predicted max
 * value obtained by the Q function for any coming out of board [b] (with
 * [state] as a vector representation of [b]) depending on whether the player is
 * white. It uses [q_network] to approximate the output.
 */
fn compute_q_max(
    b: &Board,
    state: Vec<f64>,
    q_network: &mut FeedForward,
    player_white: bool,
) -> f64 {
    // Generate legal moves
    let legal_moves = MoveGen::new_legal(&b);

    // No more moves, means we are at end state
    if legal_moves.len() == 0 {
        return 0.;
    }

    // For each legal move compute the score
    let mut high_score = f64::NEG_INFINITY;
    for m in legal_moves {
        let mut a = get_action(&m.to_string(), player_white);
        let mut sa = state.clone();
        sa.append(&mut a);

        let nn_calc = q_network.calc(&sa[..]);
        let score = nn_calc[0];
        if score >= high_score {
            high_score = score;
        }
    }
    return high_score;
}

/**
 * [learn_from_experience(policy_network, q_network, replay_memory, gamma, player_white)]
 * trains the policy network on all experiences in [replay_memory] based on
 * whether the player is white, with [q_network] as the network that
 * approximates the Q-function and [gamma] being the discounting factor used in
 * the Bellman equation.
 */
pub fn learn_from_experience(
    policy_network: &mut FeedForward,
    mut q_network: FeedForward,
    replay_memory: Vec<Experience>,
    gamma: f64,
    player_white: bool,
) {
    for e in replay_memory {
        // Extract action
        let mut action = e.action;

        // Build state-action pair
        let mut sa = e.state.clone();
        sa.append(&mut action);

        // Calculate label from q network on next state using Bellman equation
        let bellman_label = e.reward
            + gamma * compute_q_max(&e.next_board, e.next_state, &mut q_network, player_white);

        // Learn from training example
        policy_network.fit(&sa[..], &[bellman_label]);

        println!(
            "Experience: reward is {}, bellman label is {}",
            e.reward, bellman_label
        );
    }
}

/**
 * [move_by_policy(nn, b, player_white)] utilizes the policy represented by
 * policy network [nn] to return a chess move in board [b] depending on whether
 * the player is white. Alternatively if there are no legal moves it returns
 * None.
 */
pub fn move_by_policy(nn: &mut FeedForward, b: &Board, player_white: bool) -> Option<ChessMove> {
    // Generate legal moves
    let legal_moves = MoveGen::new_legal(&b);
    if legal_moves.len() == 0 {
        // If no legal moves, do nothing
        return None;
    }

    let state = get_state(b, player_white);

    let mut high_score: f64 = f64::NEG_INFINITY;
    let mut best_move: Option<ChessMove> = None;
    for possible_move in legal_moves {
        let mut action = get_action(&possible_move.to_string(), player_white);
        // Grab sa pair
        let mut sa = state.clone();
        sa.append(&mut action);

        // Compute Q-Value from policy
        let nn_calc = nn.calc(&sa[..]);
        let score = nn_calc[0];
        println!("{}", score);
        if score >= high_score {
            high_score = score;
            best_move = Some(possible_move);
        }
    }

    // Pick the best move
    return best_move;
}
