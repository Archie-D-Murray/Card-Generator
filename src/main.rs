use std::{fs::OpenOptions, io::{Read, Write}};

mod card;
mod input;

use crate::input::*;
use crate::card::*;

static PATH: &str = "config.json";

fn main() {
    let Ok(mut config_file) = OpenOptions::new().read(true).write(true).create(true).open(PATH) else { return; };
    let mut contents = String::new();
    let mut config_was_empty = false;
    config_file.read_to_string(&mut contents).expect("Could not read file!");
    let config = if contents.trim().is_empty() {
        config_was_empty = true;
        Config::default()
    } else {
        let deserialize = serde_json::from_str(contents.as_str());
        if deserialize.is_err() {
            config_was_empty = true;
            let _ = config_file.set_len(0);
        }
        deserialize.unwrap_or(Config::default())
    };
    
    if config_was_empty {
        let _ = config_file.write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes());
    }
    loop {
        let name = get_name();
        if name.is_empty() {
            break;
        }
        let mut card = get_card_base(&name, config.clone());
        println!("Created card with power budget: {} and split {}/{}", card.budget, card.budget_share.0, card.budget_share.1);
        card.with_range(get_range());
        println!("New budget: {}", card.budget);
        card.with_effect(get_effect(apply_multiplier(card.budget, card.budget_share.0), &card));
        println!("New budget: {}", card.budget);
        let card_result = card.build();

        match card_result {
            Ok(card) => {
                let card_str = card.to_string();
                println!("\nGenerated Card:\n{}", card.to_string());
                let Ok(mut card_file) = OpenOptions::new().write(true).create(true).open(format!("{}.card", card.name)) else { println!("Could not open file: {}.card", card.name); return;};
                let write_result = card_file.write_all(card_str.as_bytes());
                if write_result.is_ok() {
                    println!("Wrote card to file: {}.card", card.name);
                }
            },
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }
    print!("Exiting... ");
    let _ = std::io::stdout().flush();
    std::io::stdin().read(&mut [0]).unwrap();
}
