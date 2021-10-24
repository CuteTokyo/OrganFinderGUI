use libcoinche::{cards, pos};
use {PlayerEvent, EventType, ContractBody, CardBody};
use super::{Backend, AuctionAction, Frontend, GameAction};

pub struct Client<B: Backend> {
    pub scores: [i32; 2],
    backend: B,
}

enum GameError {
  