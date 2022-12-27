mod mdp;
use crate::mdp::learn_from_experience;
use crate::mdp::{get_action, get_reward, get_state, move_by_policy, Experience};

use chess::{Board, ChessMove, MoveGen};
use mdp::play_against_self;
use neuroflow::{io, FeedForward};
use rand::Rng;
use reqwest;
use serde_json::Value;
use std::env;
use std::fs;
use std::str::FromStr;

const INPUT_DIM: i32 = 12 * 64 + 2 * 64 + 4;
const GAMMA: f64 = 0.8;

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
 * [board_from_moves(move_str)] generates a chess board from a string of moves
 * [move_str], with each move being in uci format separated by a space. This is
 * used because the Lichess game state request reliably gives this move string.
 * Could improve to not have to redo every single move each time, but currently
 * used to keep the board updated consistently.
 */
fn board_from_moves(move_str: &str) -> Board {
    let mut board = Board::default();
    let moves = move_str.split(" ");
    for ms in moves {
        if ms.len() == 0 {
            // No moves yet
            break;
        }
        match ChessMove::from_str(ms) {
            Ok(m) => board = board.make_move_new(m),
            Err(_) => panic!(),
        };
    }

    return board;
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
    let mut color_white = true;

    // Game state booleans
    let mut first_move = true;
    let mut game_over = false;

    // Initialize experience replay memory logic
    let mut curr_experience = Experience {
        state: Vec::new(),
        action: Vec::new(),
        reward: 0.,
        next_state: Vec::new(),
        next_board: board.clone(),
    };
    let mut experience_memory: Vec<Experience> = Vec::new();

    // Initialize policy network and Q network (sync up to start game)
    let mut policy_network: FeedForward = io::load("policy.flow").unwrap(); // FeedForward::new(&[INPUT_DIM, 64, 1]);
    let mut q_network: FeedForward = io::load("policy.flow").unwrap();

    // Create new client to interact with lichess
    let client = reqwest::Client::new();

    // The game loop
    /*loop {
        // Executes once each pair of moves
        loop {
            // Waiting for my turn

            // Poll general events json stream
            let res_events = client
                .get("https://lichess.org/api/stream/event")
                .bearer_auth(&auth_token)
                .send()
                .await?
                .chunk()
                .await?;

            // Convert event response output into bytes and then json
            let res_events_bytes = match res_events {
                None => panic!(),
                Some(b) => b,
            };
            let event_json: Value = match serde_json::from_slice(&res_events_bytes) {
                Ok(j) => j,
                Err(_) => {
                    game_over = true;
                    break; // break inner loop so final board state still gets updated
                }
            };

            // Set color if first move
            if first_move {
                let color_str = match &event_json["game"]["color"] {
                    Value::String(s) => s,
                    _ => panic!(),
                };
                color_white = color_str.eq("white");
            }

            // Check if my turn
            let my_turn = match &event_json["game"]["isMyTurn"] {
                Value::Bool(b) => *b,
                _ => panic!(),
            };
            if my_turn {
                // Exit, no longer waiting for turn
                break;
            } else {
                println!("Waiting for my turn!");
            }
        }

        // Poll game-specific json stream to acquire move list
        let res_game = client
            .get("https://lichess.org/api/bot/game/stream/".to_owned() + game_id)
            .bearer_auth(&auth_token)
            .send()
            .await?
            .chunk()
            .await?;

        // Convert event response output into bytes and then json
        let res_game_bytes = match res_game {
            None => panic!(),
            Some(b) => b,
        };
        let game_json: Value = match serde_json::from_slice(&res_game_bytes) {
            Ok(j) => j,
            Err(_) => panic!(),
        };

        // Update board from moves string
        board = match &game_json["state"]["moves"] {
            Value::String(s) => board_from_moves(s),
            _ => panic!(),
        };

        // Grab board state and reward
        let board_state = get_state(&board, color_white);
        let board_reward = get_reward(&board, color_white);

        // Update previous experience and push to replay memory if not first move
        if first_move {
            first_move = false;
        } else {
            curr_experience.reward = board_reward;
            curr_experience.next_state = board_state.clone();
            curr_experience.next_board = board.clone();
            if rand::thread_rng().gen_range(0. ..=1.) < 0.2 || game_over {
                experience_memory.push(curr_experience.clone());
            }
            println!("Reward Recorded: {:#?}", curr_experience.reward);
        }

        // Last experience has been recorded, we can now end game loop
        if game_over {
            break;
        }

        // Update current experience state
        curr_experience.state = board_state.clone();

        // Select a move
        println!("Making Move!");
        let selected_move = move_by_policy(&mut policy_network, &board, color_white);
        let uci_str = match selected_move {
            None => panic!(),
            Some(m) => m.to_string(),
        };
        curr_experience.action = get_action(&uci_str, color_white);
        println!("Selected move {}", uci_str);

        // Post move
        client
            .post("https://lichess.org/api/bot/game/".to_owned() + game_id + "/move/" + &uci_str)
            .bearer_auth(&auth_token)
            .send()
            .await?;
    }

    println!("Game is over!");
    println!("Collected {} experiences", experience_memory.len());

    // Learn from experience gained in the game
    learn_from_experience(
        &mut policy_network,
        &mut q_network,
        experience_memory,
        GAMMA,
        color_white,
    );*/

    io::save(&policy_network, "policy.flow").unwrap();
    println!("Learned from game and saved policy network to file.");

    for i in 1..187 {
        println!("*** GAME {} ***", i);
        let experience_memory = play_against_self(&mut policy_network);

        learn_from_experience(
            &mut policy_network,
            &mut q_network,
            experience_memory,
            GAMMA,
            color_white,
        );

        // Save neural network to file
        io::save(&policy_network, "policy.flow").unwrap();
        q_network = io::load("policy.flow").unwrap();
        println!("Learned from game and saved policy network to file.");
    }

    Ok(())
}
