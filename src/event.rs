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
                    let target = try!(d.read_struct_field("target", 2, |d| bid::Target::decode(d)