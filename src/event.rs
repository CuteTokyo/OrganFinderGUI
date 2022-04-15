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
                s.emit_struct("PlayerEvent", 3, |s| {
                    try!(encode_field!(s, "type", 0, "Bidded"));
                    try!(encode_field!(s, "suit", 1, suit));
                    try!(encode_field!(s, "target", 2, target));
                    Ok(())
                })
            }
            &PlayerEvent::Coinched => {
                s.emit_struct("PlayerEvent",
                              1,
                              |s| encode_field!(s, "type", 0, "Coinched"))
            }
            &PlayerEvent::Passed => {
                s.emit_struct("PlayerEvent", 1, |s| encode_field!(s, "type", 0, "Passed"))
            }
            &PlayerEvent::CardPlayed(card) => {
                s.emit_struct("PlayerEvent", 2, |s| {
                    try!(encode_field!(s, "type", 0, "CardPlayed"));
                    try!(encode_field!(s, "card", 1, card));
                    Ok(())
                })
            }
        }
    }
}

impl rustc_serialize::Decodable for PlayerEvent {
    fn decode<D: rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_struct("PlayerEvent", 0, |d| {
            match try!(d.read_struct_field("type", 0, |d| d.read_str())).as_ref() {
                "Bidded" => {
                    let suit = try!(d.read_struct_field("suit", 1, |d| cards::Suit::decode(d)));
                    let target = try!(d.read_struct_field("target", 2, |d| bid::Target::decode(d)));
                    Ok(PlayerEvent::Bidded(suit, target))
                }
                "CardPlayed" => {
                    let card = try!(d.read_struct_field("card", 1, |d| cards::Card::decode(d)));
                    Ok(PlayerEvent::CardPlayed(card))
                }
                "Passed" => Ok(PlayerEvent::Passed),
                "Coinched" => Ok(PlayerEvent::Coinched),
                _ => Err(d.error("unknown event type")),
            }
        })
    }
}

/// Represents an event that can happen during the game.
#[derive(Clone,Debug)]
pub enum EventType {
    /// Special event indicating the server expects the player to take an action.
    YourTurn,

    /// The party is cancelled. Contains an optional explanation.
    PartyCancelled(String),

    /// A player did something!
    FromPlayer(pos::PlayerPos, PlayerEvent),

    /// Bid over: contains the contract and the author
    BidOver(bid::Contract),
    /// The bid was cancelled, probably because no one bidded anything.
    /// A new game is probably on its way.
    BidCancelled,

    /// Trick over: contains the winner
    TrickOver {
        winner: pos::PlayerPos,
    },

    /// New game: contains the first player, and the player's hand.
    /// For internal use only, it is never sent on the network.
    NewGame {
        first: pos::PlayerPos,
        hands: [cards::Hand; 4],
    },
    /// New game event, translated for each player.
    NewGameRelative {
        first: pos::PlayerPos,
        hand: cards::Hand,
    },

    /// Game over: contains scores
    GameOver {
        points: [i32; 2],
        winner: pos::Team,
        scores: [i32; 2],
    },
}

impl EventType {
    /// Returns a version of the event from the point of view of a given player.
    /// It returns a direct clone of the event for most event types,
    /// except for a NewGame, where it only returns the player's hand.
    pub fn relativize(&self, from: pos::PlayerPos) -> Self {
        match self {
            &EventType::NewGame { first, hands } => {
                EventType::NewGameRelative {
                    first: first,
                    hand: hands[from as usize],
                }
            }
            _ => self.clone(),
        }
    }
}

// Ugly serialization...
impl rustc_serialize::Encodable for EventType {
    fn encode<S: rustc_serialize::Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match self {
            &EventType::PartyCancelled(ref msg) => {
                s.emit_struct("Event", 2, |s| {
                    try!(encode_field!(s, "type", 0, "PartyCancelled"));
                    try!(encode_field!(s, "msg", 1, msg));
                    Ok(())
                })
            }
            &EventType::YourTurn => {
                s.emit_struct("Event", 1, |s| encode_field!(s, "type", 0, "YourTurn"))
            }
            &EventType::BidCancelled => {
                s.emit_struct("Event", 1, |s| encode_field!(s, "type", 0, "BidCancelled"))
            }
            &EventType::BidOver(ref contract) => {
                s.emit_struct("Event", 2, |s| {
                    try!(encode_field!(s, "type", 0, "BidOver"));
                    try!(encode_field!(s, "contract", 1, contract));
                    Ok(())
                })
            }
            &EventType::TrickOver { winner } => {
                s.emit_struct("Event", 2, |s| {
                    try!(encode_field!(s, "type", 0, "TrickOver"));
                    try!(encode_field!(s, "winner", 1, winner));
                    Ok(())
                })
            }
            &EventType::FromPlayer(pos, ref event) => {
                s.emit_struct("Event", 3, |s| {
                