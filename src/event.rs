//! Event module

use rustc_serialize;
use rustc_serialize::Decodable;
use libcoinche::{cards, bid, pos};

/// An event about a player.
#[derive(Clone,Debug)]
pub enum PlayerEvent {
    /// A player made a new bid in the auction.
    Bidded(cards::Suit, bid::Target),
    /// A player coinched the current bid in the auction.
    Coinched,
    /// A player passed in the auction.
    Passed,
    /// A player played a card.
    CardPlayed(cards::Card),
}

impl rustc_serialize::Encodable for PlayerEvent {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match self {
            &PlayerEvent::Bidded(suit, target) => {
                s.emit_struct("PlayerEven