use std::ffi::OsStr;

use crate::*;

pub fn generate_cards(config: Config) {
    loop {
        let name = get_name();
        if name.is_empty() {
            break;
        }
        let rarity = get_rarity();
        let efficiency = get_efficiency();
        let mut card = Card::new(name, rarity, efficiency, config.clone());
        card.print_budget_mut();
        card.with_priority_allocation(get_priority_allocation(&card));
        card.print_budget_mut();
        card.with_range(get_range());
        card.print_budget_mut();
        card.with_effect(get_effect(&card));
        let card_result = card.build();

        match card_result {
            Ok(card) => {
                let card_str = card.to_string();
                println!("\nGenerated Card:\n{}", card.to_string());
                let Ok(mut card_file) = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(format!("cards/{}.card", card.name))
                else {
                    println!("Could not open file: {}.card", card.name);
                    return;
                };
                let write_result = card_file.write_all(card_str.as_bytes());
                if write_result.is_ok() {
                    println!("Wrote card to file: {}.card", card.name);
                }
            }
            Err(err) => eprintln!("ERROR: {}", err),
        }
    }
}

pub fn generate_deck_from_template(deck_name_opt: Option<String>, config: Config) {
    let Some(deck_name) = deck_name_opt else {
        println!("No deck name provided - must match folder name containing .deck file...");
        return;
    };
    let mut options = OpenOptions::new();
    let deck_folder = format!("decks/{}/", deck_name);
    let Ok(cards) = std::fs::read_dir(deck_folder.as_str()) else {
        println!("Could not read directory {}!", deck_folder);
        return;
    };
    for card in cards
        .filter_map(|res| res.ok())
        .map(|dir| dir.path())
        .filter(|path| path.extension().unwrap_or(&OsStr::new("")) == "card")
    {
        let path = card.clone().to_str().unwrap_or("Unknown").to_string();
        if std::fs::remove_file(card).is_err() {
            println!("Could not remove existing card file: {}!", path);
            return;
        }
    }
    let deck_file = format!("{}{}.deck", deck_folder, deck_name);
    let Ok(mut deck_file) = options.read(true).open(deck_file) else {
        println!("No deck file present in {}", deck_folder);
        return;
    };
    let mut deck_buf = String::new();
    if deck_file.read_to_string(&mut deck_buf).is_err() {
        println!("Could not read file: {:?}", deck_file);
        return;
    }
    let Ok(deck) = serde_json::from_str::<DeckInputs>(deck_buf.as_str()) else {
        println!("Could not parse deck!");
        return;
    };
    for card_input in deck.inputs {
        let card_result = Card::new(
            card_input.name.clone(),
            card_input.rarity,
            card_input.efficiency,
            config.clone(),
        )
        .with_priority_allocation(card_input.priority_allocation)
        .with_range(card_input.range)
        .with_effect(card_input.effect)
        .build();
        if let Ok(card) = card_result {
            let Ok(mut card_file) = options
                .write(true)
                .create(true)
                .open(format!("{}{}.card", deck_folder, card_input.name))
            else {
                println!("Could not create file: {}", card_input.name);
                return;
            };
            if card_file.write_all(card.to_string().as_bytes()).is_err() {
                println!("Could not write to file: {}.card", card_input.name)
            }
        }
    }
}

pub fn generate_deck_file() {
    let deck_type = match get_num(
        1,
        3,
        String::from("1: Starter\n2: Journeyman\n3: Legendary\nEnter deck type (1..3)... "),
    ) - 1
    {
        2 => DeckType::Legendary,
        1 => DeckType::Journeyman,
        _ => DeckType::Starter,
    };
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(format!("{:?} Template.json", deck_type).as_str())
        .expect("Could not create template");
    file.set_len(0).expect("File could not be modified!");
    let deck = DeckInputs::new(deck_type);
    file.write_all(
        serde_json::to_string_pretty(&deck)
            .expect("Bad type")
            .as_bytes(),
    )
    .expect("Could not write to file!");
}

pub fn generate_deck() {
    let config = load_config();
    let mut options = OpenOptions::new();
    let deck_type = match get_num(
        1,
        3,
        String::from("1: Starter\n2: Journeyman\n3: Legendary\nEnter deck type (1..3)... "),
    ) - 1
    {
        2 => DeckType::Legendary,
        1 => DeckType::Journeyman,
        _ => DeckType::Starter,
    };

    let deck_name = get_string(String::from("Enter deck name: "));
    let mut deck = DeckInputs::new(deck_type);
    let root_path = format!("decks/{}/", deck_name);
    let mut last_card: Option<(Rarity, i32)> = Option::None;

    for card_input in deck.inputs.iter_mut() {
        let card = configure_card(card_input, &config, &mut last_card);
        card_input.apply_configuration(&card);
    }

    // Write out deck
    if std::fs::create_dir(&root_path).is_err() {
        println!("Could not create deck folder!");
        return;
    }
    let Ok(mut deck_file) = options
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("{}{}.deck", root_path, deck_name))
    else {
        println!("Could not create deck file!");
        return;
    };
    let Ok(deck_buf) = serde_json::to_string_pretty(&deck) else {
        println!("Bad type!");
        return;
    };
    if deck_file.write_all(deck_buf.as_bytes()).is_err() {
        println!("Bad type!");
        return;
    }
    generate_deck_from_template(Some(deck_name.clone()), config);
    println!("Generated deck: {}", deck_name.clone());
}

fn get_card_suffix(last_card: &mut Option<(Rarity, i32)>, current_rarity: &Rarity) -> String {
    match last_card {
        Some((rarity, count)) => {
            if *rarity == *current_rarity {
                format!("{}", count)
            } else {
                String::from("")
            }
        }
        None => String::from(""),
    }
}

fn configure_card(card_input: &mut CardInput, config: &Config, last_card: &mut Option<(Rarity, i32)>) -> Card {
    let card_name = get_string(format!("Enter name for {:?} card {}: ", &card_input.rarity, get_card_suffix(last_card, &card_input.rarity)));
    let efficiency = get_efficiency();
    loop {
        let mut card = Card::new(card_name.clone(), card_input.rarity.clone(), efficiency.clone(), config.clone());
        card.print_budget_mut();
        card.with_priority_allocation(get_priority_allocation(&card));
        card.print_budget_mut();
        card.with_range(get_range());
        card.print_budget_mut();
        card.with_effect(get_effect(&card));
         let card_result = card.build();
        if card_result.is_ok() {
            let last = if last_card.is_some() {
                last_card.as_ref().unwrap().1 + 1
            } else {
                1
            };
            *last_card = Some((card.rarity, last));
            return card_result.unwrap();
        } else {
            println!("Invalid configuration!");
        }
    }
}
