
extern crate coinched;
extern crate libcoinche;
extern crate clap;

use std::io;
use std::io::{BufRead, Write};
use std::str::FromStr;
use libcoinche::{bid, cards, pos};
use coinched::EventType;
use coinched::client;
use clap::{Arg, App};

struct CliFrontend {
    hand: cards::Hand,
    pos: pos::PlayerPos,
}

fn parse_bid(line: &str) -> Result<(cards::Suit, bid::Target), String> {
    let tokens: Vec<&str> = line.trim().split(" ").collect();
    if tokens.len() != 2 {
        return Err("Invalid number of tokens".to_string());
    }

    let target = try!(bid::Target::from_str(tokens[0]));
    let suit = try!(cards::Suit::from_str(tokens[1]));

    Ok((suit, target))
}

impl CliFrontend {
    fn new(pos: pos::PlayerPos) -> Self {
        CliFrontend {
            pos: pos,
            hand: cards::Hand::new(),
        }
    }

    fn input() -> String {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        // Discard the `\n` at the end
        buffer.pop().unwrap();
        buffer
    }

    fn print_hand(&self) {
        print!("Cards: [");
        let cards = self.hand.list();
        let len = cards.len();
        for card in cards {
            print!(" {}", card.to_string());
        }
        println!(" ]");
        print!("        ");
        for i in 0..len {
            print!("  {}", i);
        }
        println!("");
    }
}

impl client::Frontend<client::http::HttpBackend> for CliFrontend {
    fn show_error(&mut self, error: client::http::Error) {
        println!("Error: {:?}", error);
    }