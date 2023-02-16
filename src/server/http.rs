
use super::game_manager::GameManager;
use {ContractBody, CardBody, Error};

use std::sync::Arc;
use std::str::FromStr;

use rustc_serialize::json;
use iron::prelude::*;
use iron;
use bodyparser;

struct Router {
    manager: Arc<GameManager>,
}

#[derive(RustcEncodable)]