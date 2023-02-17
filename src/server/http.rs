
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