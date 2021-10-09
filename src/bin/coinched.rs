extern crate coinched;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::str::FromStr;
use clap::{Arg, App};

fn main() {
    env_logger::init().unwra