
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
struct HelpAction {
    href: &'static str,
    method: &'static str,
    help: &'static str,
}

#[derive(RustcEncodable)]
struct HelpMessage {
    title: &'static str,
    actions: Vec<HelpAction>,
}

pub struct Server {
    port: u16,
    manager: Arc<GameManager>,
}

fn help_message() -> String {

    json::encode(&HelpMessage {
        title: "Help Page",
        actions: vec![
            HelpAction {
                href: "/join",
                method: "POST",
                help: "Join a new game.",
            },
            HelpAction {
                href: "/leave/[PLAYER_ID]",
                method: "POST",
                help: "Leave the current game.",
            },
            HelpAction {
                href: "/pass/[PLAYER_ID]",
                method: "POST",
                help: "Pass during auction.",
            },
            HelpAction {
                href: "/coinche/[PLAYER_ID]",
                method: "POST",
                help: "Coinche the opponent's bid during auction.",
            },
            HelpAction {
                href: "/bid/[PLAYER_ID]",
                method: "POST",
                help: "Bid a contract during auction.",
            },
            HelpAction {
                href: "/play/[PLAYER_ID]",
                method: "POST",
                help: "Play a card.",
            },
            HelpAction {
                href: "/hand/[PLAYER_ID]",
                method: "GET",
                help: "Checks the current hand.",
            },
            HelpAction {
                href: "/trick/[PLAYER_ID]",
                method: "GET",
                help: "Checks the current trick.",
            },
            HelpAction {
                href: "/last_trick/[PLAYER_ID]",
                method: "GET",
                help: "Checks the last complete trick.",
            },
            HelpAction {
                href: "/scores/[PLAYER_ID]",
                method: "GET",
                help: "Get the current scores.",
            },
            HelpAction {
                href: "/pos/[PLAYER_ID]",
                method: "GET",
                help: "Get the player's position on the table.",
            },
            HelpAction {
                href: "/wait/[PLAYER_ID]/[EVENT_ID]",
                method: "GET",
                help: "Wait until the next event, or return it if it already happened.",
            },
        ],
    })
        .unwrap()
}


fn help_resp() -> IronResult<Response> {
    let content_type: iron::mime::Mime = "application/json".parse::<iron::mime::Mime>().unwrap();
    return Ok(Response::with((content_type, iron::status::NotFound, help_message())));
}

fn err_resp<S: ToString>(msg: S) -> IronResult<Response> {
    let content_type: iron::mime::Mime = "application/json".parse::<iron::mime::Mime>().unwrap();

    return Ok(Response::with((content_type,
                              iron::status::Ok,
                              json::encode(&Error { error: msg.to_string() }).unwrap())));
}

macro_rules! parse_id {
    ( $name:expr, $value:expr ) => {
        {
            match u32::from_str($value) {
                Ok(id) => id,
                Err(e) => return err_resp(format!("invalid {} ID: `{}` ({})", $name, $value, e)),
            }
        }
    };
}

macro_rules! check_len {
    ( $path:expr, 1 ) => {
        {
            if $path.len() != 1 {
                return err_resp(format!("incorrect parameters (Usage: /{})", $path[0]));
            }
        }
    };
    ( $path:expr, 2 ) => {
        {
            if $path.len() != 2 {
                return err_resp(format!("incorrect parameters (Usage: /{}/[PID])", $path[0]));
            }
        }
    };
    ( $path:expr, 3 ) => {
        {
            if $path.len() != 3 {
                return err_resp(format!(
                        "incorrect parameters (Usage: /{}/[PID]/[EID])",
                        $path[0]));
            }
        }
    };
}

macro_rules! my_try {
    ( $x:expr ) => {

        {