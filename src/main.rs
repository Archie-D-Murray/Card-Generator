use std::{fs::OpenOptions, io::{Read, Write}};
use crate::{input::*, card::*, generators::*};

mod card;
mod input;
mod generators;

static PATH: &str = "config.json";

fn main() {
    let config = load_config();
    match std::env::args().nth(1).unwrap_or(String::from("")).as_str() {
        "--deck-template" => generate_deck_file(),
        "--deck" => generate_deck(std::env::args().nth(2), config),
        "--deck-examples" => {
            generate_deck(Some(String::from("starter")), config.clone());
            generate_deck(Some(String::from("journeyman")), config.clone());
            generate_deck(Some(String::from("legendary")), config.clone());
        }
        _ => generate_cards(config)
    };
}
