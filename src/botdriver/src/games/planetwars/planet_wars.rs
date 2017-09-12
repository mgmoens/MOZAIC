extern crate serde_json;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell, Ref};

use game::*;
use games::planetwars::protocol;
use games::planetwars::planet_gen::{gen_map};

const START_SHIPS: u64 = 15;



pub struct Fleet {
    owner: Rc<RefCell<Player>>,
    ship_count: u64,
}

pub struct Planet {
    pub name: String,
    pub fleets: Vec<Fleet>,
    pub x: u64,
    pub y: u64,
}

impl Planet {
    fn owner<'a>(&'a self) -> Option<Ref<'a, Player>> {
        self.fleets.first().map(|f| f.owner.borrow())
    }
    
    fn ship_count(&self) -> u64 {
        self.fleets.first().map_or(0, |f| f.ship_count)
    }

    fn orbit(&mut self, fleet: Fleet) {
        // TODO: deduplication (merge fleets from same player)
        self.fleets.push(fleet);
    }

    fn resolve_combat(&mut self) {
        // destroy some ships
        self.fleets.sort_by(|a, b| a.ship_count.cmp(&b.ship_count).reverse());
        while self.fleets.len() > 1 {
            let fleet = self.fleets.pop().unwrap();
            for other in self.fleets.iter_mut() {
                other.ship_count -= fleet.ship_count;
            }

            // remove dead fleets
            while self.fleets[self.fleets.len()-1].ship_count == 0 {
                self.fleets.pop();
            }
        }
    }

    fn distance(&self, other: &Planet) -> u64 {
        (((self.x - other.x).pow(2) - (self.y - other.y).pow(2)) as f64).sqrt() as u64
    }
}

struct Expedition {
    origin: Rc<RefCell<Planet>>,
    target: Rc<RefCell<Planet>>,
    fleet: Fleet,
    turns_remaining: u64,
}

impl Expedition {
    fn into_orbit(self) {
        self.target.borrow_mut().orbit(self.fleet);
    }

    fn repr(&self) -> protocol::Expedition {
        protocol::Expedition {
            ship_count: self.fleet.ship_count,
            origin: self.origin.borrow().name.clone(),
            destination: self.target.borrow().name.clone(),
            owner: self.fleet.owner.borrow().name.clone(),
            turns_remaining: self.turns_remaining,
        }
    }


}

struct Player {
    id: usize,
    name: String,
}

impl PartialEq for Player {
    fn eq(&self, other: &Player) -> bool {
        self.name == other.name
    }
}


pub struct PlanetWars {
    players: HashMap<PlayerId, Rc<RefCell<Player>>>,
    eliminated: Vec<Rc<RefCell<Player>>>,
    planets: HashMap<String, Rc<RefCell<Planet>>>,
    expeditions: Vec<Expedition>,
}

impl Game for PlanetWars {
    type Outcome = Vec<Rc<RefCell<Player>>>;
    type Config = ();

    fn init(params: MatchParams<Self>) -> (Self, GameStatus<Self>) {
        // Transform to HashMap<PlayerId, Rc<RefCell<Player>>>
        let mut players = HashMap::new();
        for (&id, info) in params.players.iter() {
            players.insert(id, Rc::new(RefCell::new(
                Player {
                    id: id,
                    name: info.name.clone(),
                }))
            );
        }
        let mut state = PlanetWars {
            planets: gen_map(players.len()),
            players: players,
            eliminated: Vec::new(),
            expeditions: Vec::new(),

        };

        state.place_players();

        let prompts = state.generate_prompts();
        return (state, GameStatus::Prompting(prompts));
    }

    fn step(&mut self, player_output: &PlayerMap<String>) -> GameStatus<Self> {
        let mut commands : Vec<(PlayerId, protocol::Command)> = Vec::new();

        // Serialize commands
        for (player, command) in player_output {
            let c: protocol::Command = match serde_json::from_str(command) {
                Ok(command) => command,
                Err(_) => protocol::Command { value: None }
            };
            commands.push((player.clone(), c))
        }

        // Filter empty moves
        let moves = commands.into_iter().filter_map(|(player, command)| {
            command.value.map(|moof| (player, moof))
        });

        // Filter invalid moves
        let moves :Vec<(PlayerId, protocol::Move)> = moves.filter_map(|(player, moof)| {
            self.validate_move(player.clone(), moof).map(|moof| (player, moof))
        }).collect();

        // Add moves as expeditionss
        for (player, moof) in moves {
            let exp = self.exp_from_move(player, moof);
            self.expeditions.push(exp)
        }

        // Play one step of the game, given the new expeditions
        self.step_expeditions();
        self.resolve_combats();

        if self.is_finished() {
            //TODO Actually make the outcome
            // -> Vector with winner first, then losers
            // -> last item is first player to be eliminated
            GameStatus::Finished(Vec::new())
        } else {
            GameStatus::Prompting(self.generate_prompts())
        }
    }
}

impl PlanetWars {

    fn validate_move(&mut self, player: PlayerId, m: protocol::Move) -> Option<protocol::Move> {
        // Check whether origin is a valid planet
        let origin = match self.planets.get(&m.origin) {
            Some(planet) => planet.borrow(),
            None => return None
        };

        // Check whether destination is a valid planet
        let destination = match self.planets.get(&m.destination){
            Some(planet) => planet,
            None => return None
        };

        // Check whether the sender is the owner of origin
        if origin.owner().map(|p| p.id) != Some(player) {
            return None
        }

        // Check whether the owner has enough ships to send
        if origin.ship_count() < m.ship_count {
            return None
        }

        Some(m)
    }

    fn generate_prompts(&self) -> PlayerMap<String> {
        let mut prompts = HashMap::new();
        let state = self.to_state();

        for (&id, _) in &self.players {
            if self.is_alive(id) {
                let serialized = serde_json::to_string(&state)
                    .expect("[PLANET_WARS] Serializing game state failed.");
                prompts.insert(id, serialized);
            }
        }
        return prompts;
    }

    fn is_finished(&self) -> bool {
        return self.eliminated.len() < self.players.len() - 1;
    }

    fn is_alive(&self, id: PlayerId) -> bool {
        self.expeditions.iter().any(|e| {
            e.fleet.owner.borrow().id == id
        })
    }

    fn owner_name(&self, id: PlayerId) -> String {
        self.players[&id].borrow().name.clone()
    }

    fn to_state(&self) -> protocol::State {
        let mut players = Vec::new();
        let mut planets = Vec::new();

        //Fill players vector
        for id in self.players.keys() {
            players.push(id);
        }

        //Fill planets vector
        for planet in self.planets.values() {
            let planet_value = planet.borrow();

            let planet_clone = protocol::Planet {
                ship_count: planet_value.ship_count(),
                x: planet_value.x as f64,
                y: planet_value.y as f64,
                owner: planet_value.owner().map(|p| p.name.clone()),
                name: planet_value.name.clone(),
            };
            planets.push(planet_clone);
        }

        //Fill expeditions vector
        let expeditions = self.expeditions.iter().map(|e| e.repr()).collect();

        protocol::State {
            planets: planets,
            expeditions: expeditions,
            players: self.players.values()
                                 .map(|p| p.borrow().name.clone())
                                 .collect(),
        }

    }

    fn exp_from_move(&mut self, player_name: PlayerId, m: protocol::Move) -> Expedition {
        let owner = self.players.get(&player_name).unwrap(); // Add error message
        let fleet = Fleet {
            owner: owner.clone(),
            ship_count: m.ship_count
        };

        let origin = self.planets.get(&m.origin).unwrap();
        let target = self.planets.get(&m.destination).unwrap(); // Add error message
        Expedition {
            origin: origin.clone(),
            target: target.clone(),
            fleet: fleet,
            turns_remaining: origin.borrow().distance(&target.borrow())
        }
    }

    fn step_expeditions(&mut self) {
        let mut i = 0;
        let exps = &mut self.expeditions;
        while i < exps.len() {
            exps[i].turns_remaining -= 1;
            if exps[i].turns_remaining == 0 {
                exps.swap_remove(i).into_orbit();
            } else {
                i += 1;
            }
        }
    }

    fn resolve_combats(&mut self) {
        for planet in self.planets.values() {
            planet.borrow_mut().resolve_combat();
        }
    }

    fn place_players(&mut self) {
        let mut values = self.planets.values_mut().take(self.players.len());
        for (_, player) in &self.players {
            let value : &mut Rc<RefCell<Planet>> = values.next()
                .expect("Not enough planets generated for players.");
            let mut value = Rc::get_mut(value).unwrap().borrow_mut();
            value.fleets.push( Fleet {
                owner: player.clone(),
                ship_count: START_SHIPS
            });
        }
    }
}