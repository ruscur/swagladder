use elo::Elo;

#[derive(RustcEncodable)]
pub struct Player {
    name: String,
    rating: f32,
    games: u64,
    wins: u64,
    losses: u64
}

impl Player {
    pub fn new(name: String) -> Player {
        Player {
            name: name,
            rating: 1000f32,
            games: 0,
            wins: 0,
            losses: 0
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_games(&self) -> u64 {
        self.games
    }

    pub fn get_wins(&self) -> u64 {
        self.wins
    }

    pub fn get_losses(&self) -> u64 {
        self.losses
    }

}

impl Elo for Player {
    fn get_rating(&self) -> f32 {
        self.rating
    }

    fn change_rating(&mut self, rating: f32) {
        self.games += 1;
        if rating > 0f32 {
            self.wins += 1;
        } else {
            self.losses += 1;
        }
        self.rating += rating;
    }
}
