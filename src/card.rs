use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RarityRanges {
    pub bad: [i32; 2],
    pub not_great: [i32; 2],
    pub normal: [i32; 2],
    pub good: [i32; 2],
    pub great: [i32; 2],
}

impl Default for RarityRanges {
    fn default() -> Self {
        RarityRanges {
            bad: [2, 2],
            not_great: [3, 4],
            normal: [5, 6],
            good: [7, 8],
            great: [9, 10]
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Config {
    pub rarity_ranges: RarityRanges,
    pub power_to_priority: f32,
    pub damage_range_modifiers: RangeModifiers,
    pub heal_range_modifiers: RangeModifiers,
    pub acid_heal_range_modifiers: RangeModifiers,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rarity_ranges: RarityRanges::default(),
            power_to_priority: 1.0,
            damage_range_modifiers: RangeModifiers::default(),
            heal_range_modifiers: RangeModifiers::default(),
            acid_heal_range_modifiers: RangeModifiers::default(),
        }
    }
}

impl Config {
    fn get_effect_range_modifier(&self, effect: &Effect, range: &Range) -> f32 {
        match *effect {
            Effect::Heal(_) => self.heal_range_modifiers.get_modifier(range),
            Effect::AcidHeal(_) => self.acid_heal_range_modifiers.get_modifier(range),
            Effect::Damage(_) => self.damage_range_modifiers.get_modifier(range),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RangeModifiers {
    pub single: f32,
    pub multiple: f32,
    pub aoe: f32,
    pub aoe_extended: f32,
}

impl RangeModifiers {
    pub fn get_modifier(&self, range: &Range) -> f32 {
        match *range {
            Range::Single => self.single,
            Range::Multiple => self.multiple,
            Range::AoE => self.aoe,
            Range::ExtendedAoE => self.aoe_extended,
        }
    }
}

impl Default for RangeModifiers {
    fn default() -> Self {
        RangeModifiers { single: 1.0, multiple: 1.0, aoe: 1.0, aoe_extended: 1.0 }
    }
}

use rand::Rng;

pub const DEFAULT_PRIORITY: u32 = 11;
pub const PADDING: usize = 36;

pub fn pad_right(string: String, len: usize, whitespace_ch: char) -> String {
    let mut padded = String::with_capacity(len); 
    padded.push_str(string.as_str());
    for _ in 0..(len - string.len()) {
    padded.push(whitespace_ch);
    }

    padded
}

pub fn apply_multiplier(value: i32, multiplier: f32) -> i32 {
    (value as f32 * multiplier).floor() as i32
}

#[derive(Debug, Clone)]
pub enum Rarity {
    Bad,
    NotGreat,
    Normal,
    Good,
    Great,
}

pub fn cost_from_rarity(rarity: &Rarity, config: &Config) -> Vec<i32> {
    match rarity {
        Rarity::Bad => config.rarity_ranges.bad.to_vec(),
        Rarity::NotGreat => config.rarity_ranges.not_great.to_vec(),
        Rarity::Normal => config.rarity_ranges.normal.to_vec(),
        Rarity::Good => config.rarity_ranges.good.to_vec(),
        Rarity::Great => config.rarity_ranges.great.to_vec(),
    }
}

pub enum Efficiency {
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
pub enum Effect {
    Heal(i32),
    AcidHeal(i32),
    Damage(i32),
}

pub fn cost_from_effect(effect: Effect, budget: i32, range: &Option<Range>, config: &Config) -> (Option<Effect>, i32) {
    let effect_modifier = config.get_effect_range_modifier(&effect, &range.as_ref().expect("No Range in card... How?"));
    let budget = apply_multiplier(budget, 1.0 / effect_modifier);
    match effect {
        Effect::Heal(_) => (Some(Effect::Heal(budget)), apply_multiplier(budget, effect_modifier)),
        Effect::AcidHeal(_) => (Some(Effect::AcidHeal(budget)), apply_multiplier(budget, effect_modifier)),
        Effect::Damage(_) => (Some(Effect::Damage(budget)), apply_multiplier(budget, effect_modifier)),
    }
}

#[derive(Clone, Debug)]
pub enum Range {
    Single,
    Multiple,
    AoE,
    ExtendedAoE,
}

pub fn cost_from_range(range: &Range) -> i32 {
    match range {
        Range::Single => 0,
        Range::Multiple => 1,
        Range::AoE => 2,
        Range::ExtendedAoE => 4,
    }
}

#[derive(Clone, Debug)]
pub struct Card {
    pub name: String,
    pub budget: i32,
    pub efficiency: f32,
    pub priority: u32,
    pub barnacles: i32,
    pub rarity: Rarity,
    pub budget_share: (f32, f32),
    pub range: Option<Range>,
    pub effect: Option<Effect>,
    pub config: Config
}

pub fn in_range<T>(value: T, min: T, max: T) -> bool
where
    T: PartialOrd,
{
    value >= min && value <= max
}

pub fn priority_from_budget(budget: i32, config: &Config) -> i32 {
    if budget < 0 {
        0
    } else {
        (apply_multiplier(budget, config.power_to_priority) + 1).min(DEFAULT_PRIORITY as i32)
    }
}

impl Card {
    pub fn new(name: String, rarity: Rarity, efficiency: Efficiency, effect_share: f32, config: Config) -> Card {
        let range = cost_from_rarity(&rarity, &config);
        let efficiency = multiplier_from_efficiency(&efficiency);
        let mut rng = rand::thread_rng();
        let rarity_value = range[rng.gen_range(0..range.len())];
        let budget = apply_multiplier(rarity_value, efficiency);
        Card {
            name,
            budget,
            rarity,
            priority: DEFAULT_PRIORITY,
            efficiency,
            barnacles: 100000000,
            budget_share: (effect_share, 1.0 - effect_share),
            range: None,
            effect: None,
            config
        }
    }

    pub fn with_range(&mut self, range: Range) -> &mut Card {
        let cost = cost_from_range(&range);
        self.range = Some(range);
        self.budget -= cost;
        self
    }

    pub fn with_effect(&mut self, effect: Effect) -> &mut Card {
        let (created_effect, used) =
            cost_from_effect(effect, apply_multiplier(self.budget.max(0), self.budget_share.0), &self.range, &self.config);
        self.budget -= used;
        self.effect = created_effect;
        self
    }

    pub fn get_recast(&self) -> i32 {
        apply_multiplier(self.barnacles, 1.5)
    }

    pub fn to_string(&self) -> String {
        // Rarity, Effect, Cost, Recast Cost
        String::from(
            format!("{}: \n\tPriority: {}\n\tRarity: {:?}\n\tCast: {} barnacles\n\tRecast: {} barnacles\n\tEffect: {:?}, Range: {:?}", self.name, self.priority, self.rarity, self.barnacles, self.get_recast(), self.effect.clone().unwrap(), self.range.clone().unwrap())
        )
    }

    pub fn build(&mut self) -> Result<Card, String> {
        self.priority -= priority_from_budget(self.budget, &self.config) as u32;
        self.barnacles = get_barnacles(self);
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

fn get_barnacles(card: &Card) -> i32 {
    // Formula = magnitude_of_effect * effect_type + range_modifier / efficiency
    apply_multiplier(barnacles_from_effect(&card.effect) + cost_from_range(&card.range.as_ref().unwrap_or(&Range::Single)), 1.0 / card.efficiency)
}

fn barnacles_from_effect(effect: &Option<Effect>) -> i32 {
    if effect.as_ref().is_none() { return 100000000; }
    match effect.as_ref().unwrap() {
        Effect::Heal(magnitude) => apply_multiplier(*magnitude, 1.25),
        Effect::AcidHeal(magnitude) => apply_multiplier(*magnitude, 1.125),
        Effect::Damage(magnitude) => *magnitude,
    }
}
