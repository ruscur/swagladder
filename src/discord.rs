use inth_oauth2::provider::Provider;
use inth_oauth2::token::{Bearer, Expiring};
use inth_oauth2::Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Discord;

impl Provider for Discord {
    type Lifetime = Expiring;
    type Token = Bearer<Expiring>;
    fn auth_uri() -> &'static str { "https://discordapp.com/api/oauth2/authorize" }
    fn token_uri() -> &'static str { "https://discordapp.com/api/oauth2/token" }
}

pub const DISCORD_SCOPES: &'static str = "email guilds";

pub fn get_client() -> Client<Discord> {
    Client::<Discord>::new(
        // XXX don't commit these to git that would be very bad
//        String::from(""),
//        String::from(""),
//        Some(String::from(""))
    )
}
