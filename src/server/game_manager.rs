//! A game manager, mostly for the server.

use rand::{thread_rng, Rng};
use time;

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};

use eventual::{Future, Complete, Async};

use libcoinche::{bid, cards, pos, g