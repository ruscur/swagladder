use elo::Elo;
use std::cmp::Ordering;

#[derive(RustcEncodable, RustcDecodable, PartialEq, PartialOrd, Default, Clone, Debug)]
pub struct Player {
    name: String,
    rating: f32,
    games: u64,
    wins: u64,
    losses: u64,
    rank: Option<u64>
}

impl Player {
    pub fn new(name: String) -> Player {
        Player {
            name: name,
            rating: 1000f32,
            games: 0,
            wins: 0,
            losses: 0,
            ..Default::default()
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

    pub fn set_rank(&mut self, rank: u64) {
        self.rank = Some(rank);
    }
}

impl Eq for Player {}

impl Ord for Player {
    fn cmp(&self, other: &Player) -> Ordering {
        // For some reason f32 doesn't implement cmp, because there's a
        // tiny chance it's wrong because floating point numbers are
        // broken in computing, which is true, but still dumb.
        println!("a {} b {}", self.rating, other.get_rating());
        if self.rating > other.get_rating() {
            Ordering::Less
        } else if self.rating == other.get_rating() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
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
