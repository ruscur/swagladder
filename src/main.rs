extern crate iron;
extern crate rustc_serialize;
extern crate redis;
#[macro_use]
extern crate router;
extern crate handlebars_iron;
extern crate env_logger;

mod elo;
mod player;

use elo::{Elo, EloRanking};
use player::Player;

use iron::prelude::*;
use iron::status;

use rustc_serialize::json;
use rustc_serialize::json::{Json};

use redis::{Commands, RedisError};

use router::{Router};

use handlebars_iron::{HandlebarsEngine, DirectorySource, Template};

use std::collections::BTreeMap;

fn get_players() -> Vec<Player> {
    // Connect to Redis and get a list of players
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();
    let result: Vec<String> = con.hvals("players").unwrap();
    let mut players: Vec<Player> = vec!();
    for enc in result {
        players.push(json::decode::<Player>(&enc).unwrap());
    }
    players
}

fn get_player_by_name(name: &String) -> Option<Player> {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();
    let result: Result<String, RedisError> = con.hget("players", name);
    match result {
        Ok(enc) => Some(json::decode::<Player>(&enc).unwrap()),
        Err(e) => None
    }
}

fn add_player(name: String) {
    let player = Player::new(name);
    let player_json = json::encode(&player).unwrap();
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();
    let _ : () = con.hset("players", player.get_name(), player_json).unwrap();
}

fn update_player(player: Player) {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();
    let _ : () = con.hset("players", player.get_name(), json::encode(&player).unwrap()).unwrap();
}

fn main() {
    env_logger::init().unwrap();

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));

    if let Err(e) = hbse.reload() {
        panic!("{}", e);
    }

    let router = router!(
        root: get "/" => index_handler,
        players: get "/player" => index_handler,
        player_get: get "/player/:name" => player_get_handler,
        player_set: put "/player/:name" => player_set_handler,
        result_set: put "/result/:winner/:loser" => result_handler
    );

    let mut chain = Chain::new(router);
    chain.link_after(hbse);

    Iron::new(chain).http("127.0.0.1:42069").unwrap();

    fn index_handler(_: &mut Request) -> IronResult<Response> {
        let players = Json::from_str(&json::encode(&get_players()).unwrap()).unwrap();
        let mut data: BTreeMap<String, Json> = BTreeMap::new();
        data.insert("players".to_string(), players);
        let mut resp = Response::new();
        resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
        Ok(resp)
    }

    fn player_get_handler(req: &mut Request) -> IronResult<Response> {
        let ref name = req.extensions.get::<Router>().unwrap().find("name").unwrap_or("/");
        match get_player_by_name(&name.to_string()) {
            Some(player) => Ok(Response::with((status::Ok, json::encode(&player).unwrap()))),
            None => Ok(Response::with((status::NotFound, "Player not found")))
        }
    }

    fn player_set_handler(req: &mut Request) -> IronResult<Response> {
        let ref name = req.extensions.get::<Router>().unwrap().find("name").unwrap_or("/");
        match get_player_by_name(&name.to_string()) {
            Some(_) => Ok(Response::with((status::Conflict, "Name already exists"))),
            None => {
                add_player(name.to_string());
                Ok(Response::with((status::Created, "New player added")))
            }
        }
    }

    fn result_handler(req: &mut Request) -> IronResult<Response> {
        let ref winner = req.extensions.get::<Router>().unwrap().find("winner").unwrap_or("/");
        let ref loser = req.extensions.get::<Router>().unwrap().find("loser").unwrap_or("/");
        let winner_player = get_player_by_name(&winner.to_string());
        let loser_player = get_player_by_name(&loser.to_string());
        if winner_player.is_none() || loser_player.is_none() {
            return Ok(Response::with((status::NotFound, "Player not found")));
        }
        let mut winner_player = winner_player.unwrap();
        let mut loser_player = loser_player.unwrap();
        let rating_system = EloRanking::new(32);
        rating_system.win::<Player>(&mut winner_player, &mut loser_player);
        update_player(winner_player);
        update_player(loser_player);
        Ok(Response::with((status::NoContent)))
    }
}
