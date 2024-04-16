use std::{io::Write, str::FromStr};

use rand::Rng;

const DEFAULT_PRIORITY: u32 = 11;

fn apply_multiplier(value: i32, multiplier: f32) -> i32 {
    (value as f32 * multiplier).floor() as i32
}

enum Rarity {
    Bad,
    NotGreat,
    Normal,
    Good,
    Great,
}

fn cost_from_rarity(rarity: &Rarity) -> Vec<i32> {
    match rarity {
        Rarity::Bad => vec![2, 2],
        Rarity::NotGreat => vec![3, 4],
        Rarity::Normal => vec![5, 6],
        Rarity::Good => vec![7, 8],
        Rarity::Great => vec![9, 10],
    }
}

enum Efficiency {
    Bad,
    Normal,
    Good,
}

fn multiplier_from_efficiency(efficiency: &Efficiency) -> f32 {
    match efficiency {
        Efficiency::Bad => 0.75,
        Efficiency::Normal => 1.0,
        Efficiency::Good => 1.5,
    }
}

#[derive(Clone, Debug)]
enum Effect {
    Heal(i32),
    AcidHeal(i32),
    Damage(i32),
    DoT(i32, i32),
}

fn cost_from_effect(effect: Effect, budget: i32) -> (Effect, i32) {
    match effect {
        Effect::Heal(_) => {
            let amount = (budget as f32 / 1.5).floor() as i32;
            (Effect::Heal(amount), (amount as f32 * 1.5).floor() as i32)
        }
        Effect::AcidHeal(_) => {
            let amount = (budget as f32 / 1.75).floor() as i32;
            (
                Effect::AcidHeal(amount),
                (amount as f32 * 1.75).floor() as i32,
            )
        }
        Effect::Damage(_) => (Effect::Damage(budget), budget),
        Effect::DoT(_, duration) => {
            let tick = budget / duration;
            (Effect::DoT(tick, duration), tick * duration)
        }
    }
}

#[derive(Clone, Debug)]
enum Range {
    Single,
    Multiple,
    AoE,
    ExtendedAoE,
}

fn cost_from_range(range: Range) -> i32 {
    match range {
        Range::Single => 0,
        Range::Multiple => 1,
        Range::AoE => 2,
        Range::ExtendedAoE => 4,
    }
}

#[derive(Clone, Debug)]
struct Card {
    budget: i32,
    priority: u32,
    barnacles: i32,
    budget_share: (f32, f32),
    range: Option<Range>,
    effect: Option<Effect>,
}

fn in_range<T>(value: T, min: T, max: T) -> bool
where
    T: PartialOrd,
{
    value >= min && value <= max
}

fn priority_from_budget(budget: i32) -> i32 {
    if budget < 0 {
        0
    } else {
        apply_multiplier(10, (budget as f32 / 10.0).min(1.0)).max(1)
    }
}

impl Card {
    pub fn new(rarity: Rarity, efficiency: Efficiency, effect_share: f32) -> Card {
        let range = cost_from_rarity(&rarity);
        let mut rng = rand::thread_rng();
        let rarity = range[rng.gen_range(0..range.len())];
        let budget = apply_multiplier(rarity, multiplier_from_efficiency(&efficiency));
        Card {
            budget,
            priority: DEFAULT_PRIORITY,
            barnacles: apply_multiplier(rarity, 1.0 / multiplier_from_efficiency(&efficiency)),
            budget_share: (effect_share, 1.0 - effect_share),
            range: None,
            effect: None,
        }
    }

    pub fn with_range(&mut self, range: Range) -> &mut Card {
        let cost = cost_from_range(range.clone());
        self.range = Some(range);
        self.budget -= cost;
        self
    }

    pub fn with_effect(&mut self, effect: Effect) -> &mut Card {
        let (created_effect, used) =
            cost_from_effect(effect, apply_multiplier(self.budget.max(0), self.budget_share.0));
        self.budget -= used;
        self.effect = Some(created_effect);
        self
    }

    pub fn build(&mut self) -> Result<Card, String> {
        self.priority -= priority_from_budget(self.budget) as u32;
        if self.priority == DEFAULT_PRIORITY || self.barnacles == 0 {
            Err(String::from(format!(
                "Card prio {} due to budget: {}",
                self.priority, self.budget
            )))
        } else {
            Ok(self.clone())
        }
    }
}

fn main() {
    let rarity = get_rarity();
    let efficiency = get_efficiency();
    let effect_share = get_effect_share();
    let mut card = Card::new(rarity, efficiency, effect_share);
    println!("Created card with power budget: {} and split {}/{}", card.budget, card.budget_share.0, card.budget_share.1);
    card.with_range(get_range());
    println!("New budget: {}", card.budget);
    card.with_effect(get_effect());
    println!("New budget: {}", card.budget);
    let card_result = card.build();
    match card_result {
        Ok(card) => println!("{:?}", card),
        Err(err) => eprintln!("ERROR: {}", err),
    }
}

fn get_num<T>(min: T, max: T, prompt: String) -> T
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

fn get_effect_share() -> f32 {
    get_num(0.0, 1.0, String::from("Enter effect share: (0.0..1.0).. "))
}

fn get_effect() -> Effect {
    let effect_type: i32 = get_num(
        1,
        4,
        String::from(
            "1: Damage\n2: Heal\n3: DoT\n4: Acid Healing\nEnter effect type: (1..4).. ",
        ),
    ) - 1;
    match effect_type {
        0 => Effect::Damage(0),
        1 => Effect::Heal(0),
        2 => {
            let turns: i32 = get_num(2, 4, String::from("Enter turn duration (2..4):\t"));
            Effect::DoT(0, turns)
        }
        3 => Effect::AcidHeal(0),
        _ => Effect::Damage(0),
    }
}

fn get_range() -> Range {
    match get_num(1, 4, String::from("1: Single\n2: Multiple (2)\n3: AoE (room)\n4: AoE (Extended)\nEnter range type: (1..4).. ")) - 1i32 {
        0 => Range::Single,
        1 => Range::Multiple,
        2 => Range::AoE,
        3 => Range::ExtendedAoE,
        _ => Range::Single
    }
}

fn get_efficiency() -> Efficiency {
    match get_num(
        1,
        3,
        String::from("1 Bad\n2: Normal\n3: Good\nEnter efficiency: (1..3).. "),
    ) - 1i32
    {
        0 => Efficiency::Bad,
        1 => Efficiency::Normal,
        2 => Efficiency::Good,
        _ => Efficiency::Bad,
    }
}

fn get_rarity() -> Rarity {
    match get_num(
        1,
        5,
        String::from(
            "1: Bad\n2: Not Great\n3: Normal\n4: Good\n5: Great\nEnter rarity type: (1..5).. ",
        ),
    ) - 1i32
    {
        0 => Rarity::Bad,
        1 => Rarity::NotGreat,
        2 => Rarity::Normal,
        3 => Rarity::Good,
        4 => Rarity::Great,
        _ => Rarity::Bad,
    }
}
