use std::fmt;
use std::convert::From;

use libcoinche::bid;
use libcoinche::game;

/// A possible error.
pub enum Error {
    /// The given player ID is not associated with an actual game
    BadPlayerId,
  