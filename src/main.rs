#[macro_use]
extern crate iron;
extern crate rustc_serialize;
extern crate redis;
#[macro_use]
extern crate router;
extern crate handlebars_iron;
extern crate env_logger;
extern crate time;
extern crate inth_oauth2;
extern crate params;
extern crate hyper;
extern crate r2d2;
extern crate r2d2_redis;
extern crate persistent;
extern crate iron_sessionstorage;

mod elo;
mod player;
mod gameresult;
mod utils;
mod discord;

use elo::{Elo, EloRanking};
use player::Player;
use gameresult::GameResult;

use iron::prelude::*;
use iron::status;
use iron::modifiers;
use iron::Url;
use iron::typemap::Key;

use rustc_serialize::json;
use rustc_serialize::json::{Json};

use redis::{Commands, RedisError, Connection};

use router::{Router};

use handlebars_iron::{HandlebarsEngine, DirectorySource, Template};

use std::collections::BTreeMap;
use std::io::Read;
use std::default::Default;
use std::ops::Deref;

use inth_oauth2::{Client, Token};

use params::{Params, Value};

use hyper::header::Authorization;

use r2d2::{Pool, PooledConnection};
use r2d2_redis::{RedisConnectionManager};

use persistent::{Read as PRead};

use iron_sessionstorage::traits::*;
use iron_sessionstorage::SessionStorage;
use iron_sessionstorage::backends::SignedCookieBackend;
use iron_sessionstorage::Value as SessionValue;

#[derive(Copy, Clone)]
pub struct Redis;
impl Key for Redis { type Value = Pool<RedisConnectionManager>; }

struct Login {
    user_json: String
}

impl iron_sessionstorage::Value for Login {
    fn get_key() -> &'static str { "discord_user" }
    fn into_raw(self) -> String { self.user_json }
    fn from_raw(value: String) -> Option<Self> {
        if value.is_empty() {
            None
        } else {
            Some(Login { user_json: value })
        }
    }
}

fn get_players(conn: PooledConnection<RedisConnectionManager>)
               -> Vec<Player> {
    let result: Vec<String> = conn.hvals("players").unwrap();
    let mut players: Vec<Player> = vec!();
    for enc in result {
        players.push(json::decode::<Player>(&enc).unwrap());
    }
    players
}

fn get_players_sorted(conn: PooledConnection<RedisConnectionManager>)
                      -> Vec<Player> {
    let mut count: u64 = 1;
    let players = get_players(conn);
    let mut sorted: Vec<Player> = players.clone();
    sorted.sort();
    for player in &mut sorted {
        player.set_rank(count);
        count += 1;
    }
    sorted
}

fn get_player_by_name(name: &String,
                      conn: PooledConnection<RedisConnectionManager>)
                      -> Option<Player> {
    let result: Result<String, RedisError> = conn.hget("players", name);
    match result {
        Ok(enc) => Some(json::decode::<Player>(&enc).unwrap()),
        Err(e) => None
    }
}

fn add_player(name: String,
              conn: PooledConnection<RedisConnectionManager>) {
    let player = Player::new(name);
    let player_json = json::encode(&player).unwrap();
    let _ : () = conn.hset("players", player.get_name(), player_json).unwrap();
}

fn update_player(player: Player,
                 conn: PooledConnection<RedisConnectionManager>) {
    let _ : () = conn.hset("players", player.get_name(), json::encode(&player).unwrap()).unwrap();
}

fn add_result(result: GameResult,
              conn: PooledConnection<RedisConnectionManager>) {
    let _ : () = conn.lpush("results", json::encode(&result).unwrap()).unwrap();
}

fn get_results(count: isize,
               conn: PooledConnection<RedisConnectionManager>)
               -> Vec<GameResult> {
    let result: Vec<String> = conn.lrange("results", 0, count).unwrap();
    let mut results: Vec<GameResult> = vec!();
    for enc in result {
        results.push(json::decode::<GameResult>(&enc).unwrap());
    }
    results
}

fn main() {
    env_logger::init().unwrap();

    let mut hbse = HandlebarsEngine::new();
    hbse.add(Box::new(DirectorySource::new("./templates/", ".hbs")));

    if let Err(e) = hbse.reload() {
        panic!("{}", e);
    }

    let db_config = Default::default();
    let db_manager = RedisConnectionManager::new("redis://127.0.0.1/").unwrap();
    let db_pool = r2d2::Pool::new(db_config, db_manager).unwrap();

    let router = router!(
        root: get "/" => index_handler,
        players: get "/player" => index_handler,
        player_get: get "/player/:name" => player_get_handler,
        player_set: put "/player/:name" => player_set_handler,
        result_set: put "/result/:winner/:loser" => result_handler,
        login: get "/login" => login_handler,
        logout: get "/logout" => logout_handler,
        discord_auth: get "/oauth20/discord" => discord_handler
    );

    let mut chain = Chain::new(router);
    chain.link(PRead::<Redis>::both(db_pool));
    chain.link_after(hbse);
    chain.link_around(SessionStorage::new(SignedCookieBackend::new(b"super secret".to_vec())));

    Iron::new(chain).http("127.0.0.1:42069").unwrap();


    fn index_handler(req: &mut Request) -> IronResult<Response> {
        let pool = req.get::<PRead<Redis>>().unwrap();
        let players = Json::from_str(&json::encode(
            &get_players_sorted(pool.get().unwrap())).unwrap()).unwrap();
        let results = Json::from_str(&json::encode(
            &get_results(50, pool.get().unwrap())).unwrap()).unwrap();
        let mut data: BTreeMap<String, Json> = BTreeMap::new();
        if try!(req.session().get::<Login>()).is_some() {
            data.insert("user".to_string(), Json::from_str(&req.session().get::<Login>().unwrap().unwrap().into_raw()).unwrap());
        }
        data.insert("players".to_string(), players);
        data.insert("results".to_string(), results);
        let mut resp = Response::new();
        resp.set_mut(Template::new("index", data)).set_mut(status::Ok);
        Ok(resp)
    }

    fn player_get_handler(req: &mut Request) -> IronResult<Response> {
        let conn = req.get::<PRead<Redis>>().unwrap().get().unwrap();
        let ref name = req.extensions.get::<Router>().unwrap().find("name").unwrap_or("/");
        match get_player_by_name(&name.to_string(), conn) {
            Some(player) => Ok(Response::with((status::Ok, json::encode(&player).unwrap()))),
            None => Ok(Response::with((status::NotFound, "Player not found")))
        }
    }

    fn player_set_handler(req: &mut Request) -> IronResult<Response> {
        let pool = req.get::<PRead<Redis>>().unwrap();
        let ref name = req.extensions.get::<Router>().unwrap().find("name").unwrap_or("/");
        match get_player_by_name(&name.to_string(), pool.get().unwrap()) {
            Some(_) => Ok(Response::with((status::Conflict, "Name already exists"))),
            None => {
                add_player(name.to_string(), pool.get().unwrap());
                Ok(Response::with((status::Created, "New player added")))
            }
        }
    }

    fn result_handler(req: &mut Request) -> IronResult<Response> {
        let pool = req.get::<PRead<Redis>>().unwrap();
        let ref winner = req.extensions.get::<Router>().unwrap().find("winner").unwrap_or("/");
        let ref loser = req.extensions.get::<Router>().unwrap().find("loser").unwrap_or("/");
        let winner_player = get_player_by_name(&winner.to_string(), pool.get().unwrap());
        let loser_player = get_player_by_name(&loser.to_string(), pool.get().unwrap());
        if winner_player.is_none() || loser_player.is_none() {
            return Ok(Response::with((status::NotFound, "Player not found")));
        }
        let mut winner_player = winner_player.unwrap();
        let mut loser_player = loser_player.unwrap();
        let result = GameResult::new(winner_player.get_name(), loser_player.get_name());
        let rating_system = EloRanking::new(32);
        rating_system.win::<Player>(&mut winner_player, &mut loser_player);
        update_player(winner_player, pool.get().unwrap());
        update_player(loser_player, pool.get().unwrap());
        add_result(result, pool.get().unwrap());
        Ok(Response::with((status::NoContent)))
    }

    fn login_handler(req: &mut Request) -> IronResult<Response> {
        if try!(req.session().get::<Login>()).is_some() {
            return Ok(Response::with((status::Found,
                                      modifiers::Redirect(url_for!(req, "root")))))
        }
        let client = discord::get_client();
        let url = Url::from_generic_url(client.auth_uri(Some(discord::DISCORD_SCOPES), None).unwrap()).unwrap();
        Ok(Response::with((status::Found, modifiers::Redirect(url))))
    }

    fn logout_handler(req: &mut Request) -> IronResult<Response> {
        try!(req.session().clear());
        Ok(Response::with((status::Found, modifiers::Redirect(url_for!(req, "root")))))
    }

    fn discord_handler(req: &mut Request) -> IronResult<Response> {
        let mut s = String::new();
        let client = discord::get_client();
        let map = req.get::<Params>().unwrap();
        let code = match map.find(&["code"]) {
            Some(&Value::String(ref code)) => code.clone(),
            _=> return Ok(Response::with((status::BadRequest, "Where's the code bro"))),
        };
        let http_client = Default::default();
        let token = client.request_token(&http_client, code.trim()).unwrap();
        let resp = http_client.get("https://discordapp.com/api/users/@me")
            .header(Into::<Authorization<_>>::into(&token))
            .send().unwrap().read_to_string(&mut s).unwrap();
        try!(req.session().set(Login { user_json: s }));
        Ok(Response::with((status::Found, modifiers::Redirect(url_for!(req, "root")))))
    }
}
