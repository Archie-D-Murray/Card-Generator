use std::{io::Write, str::FromStr};

use rand::Rng;

const DEFAULT_PRIORITY: u32 = 11;

fn apply_multiplier(value: u32, multiplier: f32) -> u32 {
    (value as f32 * multiplier).floor() as u32
}

enum Rarity {
    Bad,
    NotGreat,
    Normal,
    Good,
    Great,
}

fn cost_from_rarity(rarity: &Rarity) -> Vec<u32> {
    match rarity {
        Rarity::Bad => vec![1, 2],
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
    Heal(u32),
    AcidHeal(u32),
    Damage(u32),
    DoT(u32, u32),
}

fn cost_from_effect(effect: Effect, budget: u32) -> (Effect, u32) {
    match effect {
        Effect::Heal(_) => {
            let amount = (budget as f32 / 1.5).floor() as u32;
            (Effect::Heal(amount), (amount as f32 * 1.5).floor() as u32)
        }
        Effect::AcidHeal(_) => {
            let amount = (budget as f32 / 1.75).floor() as u32;
            (
                Effect::AcidHeal(amount),
                (amount as f32 * 1.75).floor() as u32,
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

fn cost_from_range(range: Range) -> u32 {
    match range {
        Range::Single => 0,
        Range::Multiple => 1,
        Range::AoE => 2,
        Range::ExtendedAoE => 4,
    }
}

#[derive(Clone, Debug)]
struct Card {
    budget: u32,
    priority: u32,
    barnacles: u32,
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

fn priority_from_budget(budget: u32) -> u32 {
    if budget == 0 {
        11
    } else {
        apply_multiplier(10, (budget as f32 / 10.0).min(1.0))
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
            cost_from_effect(effect, apply_multiplier(self.budget, self.budget_share.0));
        self.budget -= used;
        self.effect = Some(created_effect);
        self
    }

    pub fn build(&mut self) -> Result<Card, String> {
        self.priority = priority_from_budget(self.budget);
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
    let range = get_range();
    let effect = get_effect();
    let card_result = Card::new(rarity, efficiency, effect_share)
        .with_range(range)
        .with_effect(effect)
        .build();
    match card_result {
        Ok(card) => println!("{:?}", card),
        Err(msg) => println!("{}", msg),
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
    get_num(0.0, 1.0, String::from("Enter effect share: (0.0..1.0)\t"))
}

fn get_effect() -> Effect {
    let effect_type: u32 = get_num(
        1,
        4,
        String::from(
            "Enter effect type: \n1: Damage\n2: Heal\n3: DoT\n4: Acid Healing\n(1..4)..\t",
        ),
    ) - 1;
    match effect_type {
        0 => Effect::Damage(0),
        1 => Effect::Heal(0),
        2 => {
            let turns: u32 = get_num(2, 4, String::from("Enter turn duration (2..4):\t"));
            Effect::DoT(0, turns)
        }
        3 => Effect::AcidHeal(0),
        _ => Effect::Damage(0),
    }
}

fn get_range() -> Range {
    match get_num(1, 4, String::from("Enter range type: \n1: Single\n2: Multiple (2)\n3: AoE (room)\n4: AoE (Extended)\n(1..4)..\t")) - 1u32 {
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
        String::from("Enter efficiency: \n1: Bad\n2: Normal\n3: Good\n(1..3)..\t"),
    ) - 1u32
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
            "Enter rarity type: \n1: Bad\n2: Not Great\n3: Normal\n4: Good\n5: Great\n(1..5)..\t",
        ),
    ) - 1u32
    {
        0 => Rarity::Bad,
        1 => Rarity::NotGreat,
        2 => Rarity::Normal,
        3 => Rarity::Good,
        4 => Rarity::Great,
        _ => Rarity::Bad,
    }
}
