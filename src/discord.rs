use inth_oauth2::provider::Provider;
use inth_oauth2::token::{Bearer, Refresh};
use inth_oauth2::Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Discord;

#[derive(RustcDecodable)]
pub struct DiscordUser {
    pub username: String,
    pub verified: bool,
    pub mfa_enabled: bool,
    pub id: String,
    pub avatar: String,
    pub discriminator: String,
    pub email: String
}

impl Provider for Discord {
    type Lifetime = Refresh;
    type Token = Bearer<Refresh>;
    fn auth_uri() -> &'static str { "https://discordapp.com/api/oauth2/authorize" }
    fn token_uri() -> &'static str { "https://discordapp.com/api/oauth2/token" }
}

pub const DISCORD_SCOPES: &'static str = "identify email guilds";

pub fn get_client() -> Client<Discord> {
    Client::<Discord>::new(
        // XXX don't commit these to git that would be very bad
//        String::from(""),
//        String::from(""),
//        Some(String::from(""))
    )
}
