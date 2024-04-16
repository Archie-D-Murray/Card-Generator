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
    Great
}

fn cost_from_rarity(rarity: Rarity) -> Vec<u32> {
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
    Good
}

fn multiplier_from_efficiency(efficiency: Efficiency) -> f32 {
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
    DoT(u32, u32)
}

fn cost_from_effect(effect: Effect, budget: u32) -> (Effect, u32) {
    match effect {
        Effect::Heal(_) => {
            let amount = (budget as f32 / 1.5).floor() as u32;
            (Effect::Heal(amount), (amount as f32 * 1.5).floor() as u32)
        },
        Effect::AcidHeal(_) => { 
            let amount = (budget as f32 / 1.75).floor() as u32;
            (Effect::AcidHeal(amount), (amount as f32 * 1.75).floor() as u32)    
        },
        Effect::Damage(_) => (Effect::Damage(budget), budget),
        Effect::DoT(_, duration) => { 
            let tick = budget / duration;
            (Effect::DoT(tick, duration), tick * duration)
        },
    }
}

#[derive(Clone, Debug)]
enum Range {
    Single,
    Multiple,
    AoE,
    ExtendedAoE
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
    budget_share: (f32, f32),
    range: Option<Range>,
    effect: Option<Effect>,
}

fn in_range(value: u32, min: u32, max: u32) -> bool {
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
        let range = cost_from_rarity(rarity);
        let mut rng = rand::thread_rng();
        let budget = apply_multiplier(range[rng.gen_range(0..range.len())], multiplier_from_efficiency(efficiency));
        Card { budget, priority: DEFAULT_PRIORITY, budget_share: (effect_share, 1.0 - effect_share), range: None, effect: None }
    }

    pub fn with_range(&mut self, range: Range) -> &mut Card {
        let cost = cost_from_range(range.clone());
        self.range = Some(range);
        self.budget -= cost;
        self
    }

    pub fn with_effect(&mut self, effect: Effect) -> &mut Card {
        let (created_effect, used) = cost_from_effect(effect, apply_multiplier(self.budget, self.budget_share.0));
        self.budget -= used;
        self.effect = Some(created_effect);
        self
    }

    pub fn build(&mut self) -> Result<Card, String> {
        self.priority = priority_from_budget(self.budget);
        if self.priority == DEFAULT_PRIORITY {
            Err(String::from(format!("Card prio {} due to budget: {}", self.priority, self.budget)))
        } else {
            Ok(self.clone())
        }
    }
}

fn main() {
    let card_result = Card::new(Rarity::Normal, Efficiency::Good, 0.6)
        .with_range(Range::Single)
        .with_effect(Effect::Damage(2))
        .build();
    match card_result {
        Ok(card) => println!("{:?}", card),
        Err(msg) => println!("{}", msg)
    }
}
