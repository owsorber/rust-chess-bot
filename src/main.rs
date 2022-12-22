mod mdp;
use crate::mdp::{get_action, get_state};

use chess::{Board, ChessMove, MoveGen};
use rand::Rng;
use reqwest;
use serde_json::Value;
use std::env;
use std::fs;
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

    // get_state(&board, true);
    // get_action("h7h6", false);

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
