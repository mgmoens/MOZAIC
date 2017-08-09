mod types;
mod higher_lower;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate rand;

use types::*;
use std::error::Error;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::collections::HashMap;
use std::process::Command;

use higher_lower::HigherLower;

fn main() {
    let game_config: GameConfig = match parse_config() {
        Ok(config) => config,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1)
        }
    };
    run::<HigherLower>(&game_config);
}

fn parse_config() -> Result<GameConfig, Box<Error>> {
    let args: Vec<_> = env::args().collect();
    if (args.len() < 2) || (args.len() > 2) {
        let msg = format!("Expected 1 argument (config file). {} given.", args.len() - 1).to_owned();
        return Err(From::from(msg))
    }
    
    println!("Opening config {}", &args[1]);
    let path = PathBuf::from(&args[1]);
    let mut file = File::open(path)?;

    println!("Reading contents");
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    println!("Parsing config");
    let gcf: GameConfigFormat = serde_json::from_str(&contents)?;
    let mut gc: GameConfig = GameConfig { players: HashMap::new() };
    
    // Move into easier to use struct
    // TODO, make oneliner
    for pc in gcf.players { 
        gc.players.insert(pc.name.clone(), pc);
    }
    println!("Config parsed succesfully");

    Ok(gc)
}

/* The algorithm for running a game goes as follows:
 *
 * ```
 * generate initial game
 * While (not finnished) do 
 *   for each player
 *     send gamestate to player
 *     receive new commands
 *   generate new gamestate
 * ```
 */
fn run<G: Game>(config: &GameConfig) {
    let mut game = G::init(config.players.keys().cloned().collect());
    let mut gamestate = game.start();
    let mut handles = create_bot_handles(config);
    loop {
        println!("\nNew step:\n==============");
        match gamestate {
            GameStatus::Running(pi) => {
                println!("Running with new player input:\n{:?}\n", pi);
                let po = fetch_player_outputs(&config, &pi, &handles);
                println!("Received new player output:\n{:?}\n", po);
                gamestate = game.step(&po);
            },
            GameStatus::Done(outcome) => {
                println!("Done with: {:?}", outcome);
                break;
            } 
        }
    }
}

fn create_bot_handles(config: &GameConfig) -> BotHandles {
    return BotHandles::new();
}

// TODO: This could be prettier i guess
fn fetch_player_outputs(config: &GameConfig, input: &PlayerInput, handles: &BotHandles) -> PlayerOutput {
    let mut po = PlayerOutput::new();
    for (player, info) in input.iter() {
        po.insert(player.clone(), fetch_player_output(player, info, handles));
    }
    po
}

fn fetch_player_output(player: &Player, info: &GameInfo, handles: &BotHandles) -> PlayerCommand {
    return r#"{"answer":"HIGHER"}"#.to_owned();
}