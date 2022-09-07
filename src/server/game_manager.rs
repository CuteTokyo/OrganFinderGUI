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
            &Game::Playing(ref game) => game.next_player(),
        }
    }
}

// Creates a new game, starting with an auction.
// Also returns a NewGame Event with the players cards.
fn make_game(first: pos::PlayerPos) -> (bid::Auction, EventType) {
    let auction = bid::Auction::new(first);
    let hands = auction.hands();

    let event = EventType::NewGame {
        first: first,
        hands: hands,
    };

    (auction, event)
}

/// Represents a party
struct Party {
    game: Game,
    first: pos::PlayerPos,

    scores: [i32; 2],

    events: Vec<EventType>,
    observers: Mutex<Vec<Complete<Event, ()>>>,
}

impl Party {
    fn new(first: pos::PlayerPos) -> Self {
        let (auction, event) = make_game(first);
        Party {
            first: first,
            game: Game::Bidding(auction),
            scores: [0; 2],
            events: vec![event],
            observers: Mutex::new(Vec::new()),
        }
    }

    fn add_event(&mut self, event: EventType) -> Event {
        trace!("Adding event: {:?}", event);
        let ev = Event {
            event: event.clone(),
            id: self.events.len(),
        };
        let mut observers = self.observers.lock().unwrap();
        for promise in observers.drain(..) {
            // TODO: handle cancelled wait?
            promise.complete(ev.clone());
        }
        self.events.push(event);

        ev
    }

    fn get_auction_mut(&mut self) -> ManagerResult<&mut bid::Auction> {
        match self.game {
            Game::Bidding(ref mut auction) => Ok(auction),
            Game::Playing(_) => Err(Error::BidInGame),
        }
    }

    fn get_game(&self) -> ManagerResult<&game::GameState> {
        match self.game {
            Game::Bidding(_) => Err(Error::PlayInAuction),
            Game::Playing(ref game) => Ok(game),
        }
    }

    fn get_game_mut(&mut self) -> ManagerResult<&mut game::GameState> {
        match self.game {
            Game::Bidding(_) => Err(Error::PlayInAuction),
            Game::Playing(ref mut game) => Ok(game),
        }
    }

    fn next_game(&mut self) {
        // TODO: Maybe keep the current game in the history?

        let (auction, event) = make_game(self.first);

        self.first = self.first.next();
        self.game = Game::Bidding(auction);
        self.add_event(event);
    }

    fn cancel(&mut self, msg: String) {
        self.add_event(EventType::PartyCancelled(msg));
    }

    fn bid(&mut self,
           pos: pos::PlayerPos,
           trump: cards::Suit,
           target: bid::Target)
           -> ManagerResult<Event> {
        trace!("Bid from {:?}: {:?} on {:?}", pos, target, trump);
        let state = {
            let auction = try!(self.get_auction_mut());
            try!(auction.bid(pos, trump, target))
        };
        trace!("Current state: {:?}", state);

        let event = EventType::FromPlayer(pos, PlayerEvent::Bidded(trump, target));
        let main_event = self.add_event(event);
        match state {
            bid::AuctionState::Over => self.complete_auction(),
            _ => (),
        }

        Ok(main_event)
    }

    fn pass(&mut self, pos: pos::PlayerPos) -> Result<Event, Error> {
        let state = {
            let auction = try!(self.get_auction_mut());
            try!(auction.pass(pos))
        };

        let main_event = self.add_event(EventType::FromPlayer(pos, PlayerEvent::Passed));
        match state {
            bid::AuctionState::Over => self.complete_auction(),
            bid::AuctionState::Cancelled => {
                self.add_event(EventType::BidCancelled);
                self.next_game();
            }
            _ => (),
        }

        Ok(main_event)
    }

    fn coinche(&mut self, pos: pos::PlayerPos) -> Result<Event, Error> {
        let state = {
            let auction = try!(self.get_auction_mut());
            try!(auction.coinche(pos))
        };

        let main_event = self.add_event(EventType::FromPlayer(pos, PlayerEvent::Coinched));
        match state {
            bid::AuctionState::Over => self.complete_auction(),
            _ => (),
        }

        Ok(main_event)
    }

    fn complete_auction(&mut self) {
        let game = match &mut self.game {
            &mut Game::Playing(_) => unreachable!(),
            &mut Game::Bidding(ref mut auction) => {
                match auction.complete() {
                    Ok(game) => game,
                    Err(err) => panic!(err),
                }
            }
        };

        self.add_event(EventType::BidOver(game.contract().clone()));

        self.game = Game::Playing(game);
    }

    fn play_card(&mut self, pos: pos::PlayerPos, card: cards::Card) -> Result<Event, Error> {
        let result = {
            let game = try!(self.get_game_mut());
            try!(game.play_card(pos, card))
        };

        // This is the main event we want to send.
        // TODO: Batch event dispatch, and send all those together.
        let main_event = self.add_event(EventType::FromPlayer(pos, PlayerEvent::CardPlayed(card)));
        match result {
            game::TrickResult::Nothing => (),
            game::TrickResult::TrickOver(winner, game_result) => {
                self.add_event(EventType::TrickOver { winner: winner });
                match game_result {
                    game::GameResult::Nothing => (),
                    game::GameResult::GameOver{points, winners, scores} => {
                        for i in 0..2 {
                            self.scores[i] += scores[i];
                        }
                        self.add_event(EventType::GameOver {
                            points: points,
                            winner: winners,
                            scores: scores,
                        });
                        self.next_game();
                    }
                }
            }
        }

        Ok(main_event)
    }
}

// Information for a current player
struct PlayerInfo {
    // The party he's playing in
    pub party: Arc<RwLock<Party>>,
    // His position in the table
    pub pos: pos::PlayerPos,
    // Last time we received something from him
    // (to detect inactivity, and disconnect him)
    pub last_time: Mutex<time::Tm>,
}

// Maps player IDs to PlayerInfo
struct PlayerList {
    pub player_map: HashMap<u32, PlayerInfo>,
}

impl PlayerList {
    fn new() -> PlayerList {
        PlayerList { player_map: HashMap::new() }
    }

    fn get_player_info(&self, player_id: u32) -> Result<&PlayerInfo, Error> {
        match self.player_map.get(&player_id) {
            None => Err(Error::BadPlayerId),
            Some(info) => {
                // Update the last active time
                *info.last_time.lock().unwrap() = time::now();
                Ok(info)
            }
        }
    }

    // Creates 4 random IDs, avoiding clashes with the ones currently in use.
    // TODO: if it becomes performance critical, we could skip the conflict check
    //       and hope that it won't happen.
    fn make_ids(&self) -> [u32; 4] {
        // Expect self.player_map to be locked
  