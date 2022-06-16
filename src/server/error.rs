use std::fmt;
use std::convert::From;

use libcoinche::bid;
use libcoinche::game;

/// A possible error.
pub enum Error {
    /// The given player ID is not associated with an actual game
    BadPlayerId,
    /// The given event ID is not associated with an actual event
    BadEventId,

    /// Player tried to play a card during auction.
    PlayInAuction,
    /// Player tried to bid during card play.
    BidInGame,

    /// An error occured during bidding.
    Bid