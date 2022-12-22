mod mdp;
use crate::mdp::{get_action, get_reward, get_state, Experience};

use chess::{Board, ChessMove, MoveGen};
use rand::Rng;
use reqwest;
use serde_json::Value;
use std::env;
use std::fs;
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
 * [make_random_move(b)] selects a random legal move for board b. If there are
 * no legal moves, it returns None. If there is at least one legal move, it
 * returns Some(m) where m is the legal move selected.
 */
fn make_random_move(b: Board) -> Option<ChessMove> {
    // Generate legal moves
    let mut legal_moves = MoveGen::new_legal(&b);
    if legal_moves.len() == 0 {
        // If no legal moves, do nothing
        return None;
    }

    // Pick a random move
    let next_move = legal_moves.nth(rand::thread_rng().gen_range(0..=legal_moves.len() - 1));

    return next_move;
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // Parse auth token from config file
    let auth_token = read_auth_token();

    // Parse game id from command line args
    let args: Vec<String> = env::args().collect();
    let game_id = &args[1];

    // Initialize board
    let mut board = Board::default();

    // Initialize experience replay memory logic
    let mut curr_experience = Experience {
        state: Vec::new(),
        action: Vec::new(),
        reward: 0,
        next_state: Vec::new(),
    };
    let mut experience_memory: Vec<Experience> = Vec::new();
    let mut first_move = true;

    // Create new client to interact with lichess
    let client = reqwest::Client::new();

    // The game loop
    'game_loop: loop {
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
                Err(_) => break 'game_loop,
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

        // Grab board state and reward now that opponent has moved
        let board_state = get_state(&board, true);
        let board_reward = get_reward(&board, true);

        // Update previous experience and push to replay memory
        if first_move {
            first_move = false;
        } else {
            curr_experience.reward = board_reward;
            curr_experience.next_state = board_state.clone();
            experience_memory.push(curr_experience.clone());
            println!("Experience Recorded: {:#?}", curr_experience);
        }

        // Update current experience state
        curr_experience.state = board_state.clone();

        // Select a move
        println!("Making Move!");
        let selected_move = make_random_move(board);
        let uci_str = match selected_move {
            None => panic!(),
            Some(m) => m.to_string(),
        };
        curr_experience.action = get_action(&uci_str, true);
        println!("Selected move {}", uci_str);

        // Post move
        client
            .post("https://lichess.org/api/bot/game/".to_owned() + game_id + "/move/" + &uci_str)
            .bearer_auth(&auth_token)
            .send()
            .await?;
    }

    println!("Game is over!");
    Ok(())
}
