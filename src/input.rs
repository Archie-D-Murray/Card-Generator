use std::{io::Write, str::FromStr};

use crate::card::*;

pub fn get_num<T>(min: T, max: T, prompt: String) -> T
where
    T: PartialOrd,
    T: FromStr,
    T: Copy,
{
    loop {
        print!("{}", prompt);
        std::io::stdout().flush().unwrap();
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).expect("Could not read buffer");
        let Ok(val) = buf.trim().parse() else {
            println!("Could not parse {}!", buf);
            std::io::stdout().flush().unwrap();
            continue;
        };
        if in_range(val, min, max) {
            return val;
        }
        println!("Not in range!");
        std::io::stdout().flush().unwrap();
    }
}

pub fn get_priority_allocation(card: &Card) -> i32 {
    get_num(1, card.budget - 1, format!("Enter priority allocation ({}..{}): ", 1, card.budget - 1))
}

pub fn display_effect_cost(effect_data: (Option<Effect>, i32)) -> String {
    if effect_data.0.is_some() {
        String::from(format!("{}", effect_data.1))
    } else {
        String::from("N/A")
    }
}

pub fn get_effect(card: &Card) -> Effect {
    let budget = card.budget;
    let effect_type: i32 = get_num(
        1,
        3,
        String::from(
            format!("{}{}{}\nEnter effect type: (1..3).. ", 
                pad_right(format!("1: Damage (Cost: {})", display_effect_cost(cost_from_effect(Effect::Damage(0), budget, &card.range, &card.config))), PADDING, ' '),
                pad_right(format!("2: Heal (Cost: {})", display_effect_cost(cost_from_effect(Effect::Heal(0), budget, &card.range, &card.config))), PADDING, ' '),
                pad_right(format!("3: Acid Healing (Cost: {})", display_effect_cost(cost_from_effect(Effect::AcidHeal(0), budget, &card.range, &card.config))), PADDING, ' '),
            )
        ),
    ) - 1;
    match effect_type {
        0 => Effect::Damage(0),
        1 => Effect::Heal(0),
        2 => Effect::AcidHeal(0),
        _ => Effect::Damage(0),
    }
}

pub fn get_range() -> Range {
    match get_num(
        1, 
        4,
        String::from(
            format!("{}{}{}{}\nEnter range type: (1..4).. ",
                pad_right(format!("1: Single (Cost: {})", cost_from_range(&Range::Single)), PADDING, ' '),
                pad_right(format!("2: Multiple (2) (Cost: {})", cost_from_range(&Range::Multiple)), PADDING, ' '),
                pad_right(format!("3: AoE (room) (Cost: {})", cost_from_range(&Range::AoE)), PADDING, ' '),
                pad_right(format!("4: AoE (Extended) (Cost: {})", cost_from_range(&Range::ExtendedAoE)), PADDING, ' '),
            )
        )) - 1i32 {
        0 => Range::Single,
        1 => Range::Multiple,
        2 => Range::AoE,
        3 => Range::ExtendedAoE,
        _ => Range::Single
    }
}

pub fn get_efficiency() -> Efficiency {
    match get_num(
        1,
        3,
        String::from(
            format!("{}{}{}\nEnter efficiency: (1..3).. ",
                pad_right("1: Bad".into(), PADDING, ' '),
                pad_right("2: Normal".into(), PADDING, ' '),
                pad_right("3: Good".into(), PADDING, ' '),
            )),
    ) - 1i32
    {
        0 => Efficiency::Bad,
        1 => Efficiency::Normal,
        2 => Efficiency::Good,
        _ => Efficiency::Bad,
    }
}

pub fn get_rarity() -> Rarity {
    match get_num(
        1,
        5,
        String::from(
            format!("{}{}{}{}{}\nEnter rarity: (1..5).. ",
                pad_right("1: Common".into(), PADDING, ' '),
                pad_right("2: Uncommon".into(), PADDING, ' '),
                pad_right("3: Rare".into(), PADDING, ' '),
                pad_right("4: Epic".into(), PADDING, ' '),
                pad_right("5: Legendary".into(), PADDING, ' '),
            )),
    ) - 1i32
    {
        0 => Rarity::Common,
        1 => Rarity::Uncommon,
        2 => Rarity::Rare,
        3 => Rarity::Epic,
        4 => Rarity::Legendary,
        _ => Rarity::Common,
    }
}

pub fn get_name() -> String {
    let mut buf = String::new();
    print!("Enter card name (<Enter> to exit): ");
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut buf);
    String::from(buf.trim())
}

pub fn get_string(prompt: String) -> String {
    let mut buf = String::new();
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut buf);
    String::from(buf.trim())
}

pub fn show_help() {
    println!("Options:");
    println!("--deck-template Generate a deck template \nWarning: must be placed in folder of the same name as the deck and the file renamed to <deck_name>.card all in the decks/ folder)");
    println!("--deck-from-template Generates a deck from a template");
    println!("--deck-generator: Interactive deck generation");
    println!("--deck-examples: Generates example decks for all tiers");
    println!("--generate-cards: Generate induvidual cards (written to cards/ folder)");
}
