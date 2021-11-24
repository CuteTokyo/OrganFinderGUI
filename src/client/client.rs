use libcoinche::{cards, pos};
use {PlayerEvent, EventType, ContractBody, CardBody};
use super::{Backend, AuctionAction, Frontend, GameAction};

pub struct Client<B: Backend> {
    pub scores: [i32; 2],
    backend: B,
}

enum GameError {
    NoContract,
    PlayerLeft,
}


impl<B: Backend> Client<B> {
    pub fn new(backend: B) -> Self {
        Client {
            scores: [0, 0],
            backend: backend,
        }
    }

    pub fn run<F: Frontend<B>>(mut self, frontend: &mut F) -> [i32; 2] {
        loop {
            match self.backend.wait() {
                Ok(EventType::NewGameRelative {first, hand}) => {
                    match self.run_game(frontend, first, hand) {
                        Err(GameError::PlayerLeft) => return self.scores,
                        _ => (),
                    }
                }
                Ok(event) => frontend.unexpected_event(event),
                Err(err) => frontend.show_error(err),
            }
        }
    }

    fn run_game<F: Frontend<B>>(&mut self, frontend: &mut F,
                                    first: pos::PlayerPos,
                                    hand: cards::Hand) -> Result<(), GameError> {
        frontend.start_game(first, hand);
        try!(self.run_auction(frontend));
        try!(self.run_cardgame(frontend));
        Ok(())
    }

    // God that's an ugly type. Really, I want `F::Auction::Game`.
    fn run_auction<F: Frontend<B>>(&mut self, frontend: &mut F) -> Result<(), GameError> {
        loop {
            let mut event = self.backend.wait();
            match event {
                Ok(EventType::YourTurn) => {
                    event = match frontend.ask_bid() {
                        AuctionAction::Leave => {
                            frontend.party_cancelled("you left");
                            return Err(GameError::PlayerLeft);
                        }
                        AuctionActio