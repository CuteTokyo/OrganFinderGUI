//! A game manager, mostly for the server.

use rand::{thread_rng, Rng};
use time;

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};

use eventual::{Future, Complete, Async};

use libcoinche::{bid, cards, pos, game, trick};
use {Event, EventType, PlayerEvent};
use {NewPartyInfo, ContractBody, CardBody};

use super::error::Error;

use self::FutureResult::{Ready, Waiting};

enum FutureResult<T: Send + 'static> {
    Ready(T),
    Waiting(Future<T, ()>),
}

type WaitResult = FutureResult<Event>;
type JoinResult = FutureResult<NewPartyInfo>;

pub type ManagerResult<T> = Result<T, Error>;


/// Base class for managing matchmaking.
///
/// It is the main entry point for the server API.
/// It offers a thread-safe access to various actions.
pub struct GameManager {
    party_list: RwLock<PlayerList>,

    waiting_list: Mutex<Vec<Complete<NewPartyInfo, ()>>>,
}

/// Describe a single game.
pub enum Game {
    /// The game is still in the auction phase
    Bidding(bid::Auction),
    /// The game is in the main playing phase
    Playing(game::GameState),
}

impl Game {
    fn next_player(&self) -> pos::PlayerPos {
        match self {
            &Game::Bidding(ref auction) => auction.next_player(),
            &Game::Playing(ref game) => game.next_player()