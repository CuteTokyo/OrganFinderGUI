
use libcoinche::{pos, bid, cards};
use {EventType, ContractBody, CardBody};

pub mod http;
mod client;

pub use self::client::Client;

pub enum AuctionAction {
    Leave,
    Pass,
    Coinche,
    Bid((cards::Suit, bid::Target)),
}

pub enum GameAction {
    Leave,
    PlayCard(cards::Card),
}

/// Any frontend mush have these global callbacks
pub trait Frontend<B: Backend> {
    fn show_error(&mut self, error: B::Error);
    fn unexpected_event(&mut self, event: EventType);
    fn party_cancelled(&mut self, msg: &str);

    fn show_card_played(&mut self, pos: pos::PlayerPos, card: cards::Card);
    fn show_trick_over(&mut self, winner: pos::PlayerPos);
    fn ask_card(&mut self) -> GameAction;
    fn ask_bid(&mut self) -> AuctionAction;
    fn game_over(&mut self, points: [i32; 2], winner: pos::Team, scores: [i32; 2]);

    fn show_pass(&mut self, pos: pos::PlayerPos);
    fn show_coinche(&mut self, pos: pos::PlayerPos);
    fn show_bid(&mut self, pos: pos::PlayerPos, suit: cards::Suit, target: bid::Target);


    /// Auction cancelled, back to the start.
    fn auction_cancelled(&mut self);