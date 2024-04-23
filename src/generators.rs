use std::ffi::OsStr;

use crate::*;

pub fn generate_cards(config: Config) {
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
}

pub fn generate_deck(deck_name_opt: Option<String>, config: Config) {
    let Some(deck_name) = deck_name_opt else { println!("No deck name provided - must match folder name containing .deck file..."); return; };
    let mut options = OpenOptions::new();
    let deck_folder = format!("{}/", deck_name);
    let Ok(cards) = std::fs::read_dir(deck_folder.as_str()) else { println!("Could not read directory {}!", deck_folder); return; };
    for card in cards.filter_map(|res| res.ok()).map(|dir| dir.path()).filter(|path| path.extension().unwrap_or(&OsStr::new("")) == "card") {
        let path = card.clone().to_str().unwrap_or("Unknown").to_string();
        if std::fs::remove_file(card).is_err() {
            println!("Could not remove existing card file: {}!", path);
            return;
        }
    }
    let deck_file = format!("{}{}.deck", deck_folder, deck_name);
    let Ok(mut deck_file) = options.read(true).open(deck_file) else { println!("No deck file present in {}", deck_folder); return;
    };
    let mut deck_buf = String::new();
    if deck_file.read_to_string(&mut deck_buf).is_err() {
        println!("Could not read file: {:?}", deck_file);
        return;
    }
    let Ok(deck) = serde_json::from_str::<DeckInputs>(deck_buf.as_str()) else { println!("Could not parse deck!"); return; };
    for card_input in deck.get_inputs() {
        let card_result = Card::new(
                card_input.name.clone(), 
                card_input.get_rarity(), 
                card_input.get_efficiency(), 
                card_input.effect_share, 
                config.clone()
            )
            .with_range(card_input.get_range())
            .with_effect(card_input.get_effect())
            .build();
        if let Ok(card) = card_result {
            let Ok(mut card_file) = options
                .write(true)
                .create(true)
                .open(format!("{}{}.card", deck_folder, card_input.name)) 
                else { println!("Could not create file: {}", card_input.name); return; 
            };
            if card_file.write_all(card.to_string().as_bytes()).is_err() {
                println!("Could not write to file: {}.card", card_input.name)
            }
        }
    }
}

pub fn generate_deck_file() {
    let mut file = OpenOptions::new().write(true).create(true).open("Deck_Template.json").expect("Could not create template");
    file.set_len(0).expect("File could not be modified!");
    let deck = DeckInputs::default();
    file.write_all(serde_json::to_string_pretty(&deck).expect("Bad type").as_bytes()).expect("Could not write to file!");
}
