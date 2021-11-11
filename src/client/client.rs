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