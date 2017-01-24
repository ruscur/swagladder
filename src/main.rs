extern crate iron;
extern crate rustc_serialize;
extern crate redis;

mod elo;
mod player;

use elo::{Elo, EloRanking};
use player::Player;

use iron::prelude::*;
use iron::status;

use rustc_serialize::json;


fn main() {
    let rating_system = EloRanking::new(32);
    let mut ruscur = Player::new("ruscur".to_string());
    let mut njd = Player::new("Nick".to_string());

    // test redis
    let client = try!(redis::Client::open("redis://127.0.0.1/"));
    let con = try!(client.get_connection());
    let _ : () = try!(con.set("test", 420));

    println!("{}", try!(con.get("my_key")));


    rating_system.win::<Player>(&mut ruscur, &mut njd);
    let ladder = vec!(ruscur, njd);

    // TODO move should be unnecessary, clone before displaying
    Iron::new(move |_: &mut Request| {
        Ok(Response::with((status::Ok, json::encode(&ladder).unwrap())))
    }).http("localhost:8080").unwrap();
}
