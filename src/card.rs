use std::{fs::OpenOptions, io::{Read, Write}};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub enum DeckType {
    #[default]
    Starter,
    Journeyman,
    Legendary
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DeckInputs {
    pub inputs: [CardInput; 5]
}

impl DeckInputs {
    pub fn new(deck_type: DeckType) -> Self {
        DeckInputs {
            inputs: match deck_type {
                DeckType::Starter => {
                    [
                        CardInput::new(Rarity::Rare),
                        CardInput::new(Rarity::Rare),
                        CardInput::new(Rarity::Uncommon),
                        CardInput::new(Rarity::Uncommon),
                        CardInput::new(Rarity::Common),
                    ]
                },
                DeckType::Journeyman => {
                    [
                        CardInput::new(Rarity::Epic),
                        CardInput::new(Rarity::Epic),
                        CardInput::new(Rarity::Rare),
                        CardInput::new(Rarity::Rare),
                        CardInput::new(Rarity::Uncommon),
                    ]
                },
                DeckType::Legendary => {
                    [
                        CardInput::new(Rarity::Legendary),
                        CardInput::new(Rarity::Epic),
                        CardInput::new(Rarity::Epic),
                        CardInput::new(Rarity::Uncommon),
                        CardInput::new(Rarity::Rare),
                    ]
                },
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct CardInput {
    pub name: String,
    pub rarity: Rarity,
    pub efficiency: Efficiency,
    pub priority_allocation: i32,
    pub range: Range,
    pub effect: Effect,
}

impl CardInput {
    pub fn new(rarity: Rarity) -> Self {
        CardInput { 
            name: format!("{:?}", rarity), 
            rarity, 
            efficiency: Efficiency::Bad, 
            priority_allocation: 1, 
            range: Range::Single, 
            effect: Effect::Damage(0) 
        }
    }

    pub fn apply_configuration(&mut self, card: &Card) {
        assert_eq!(self.rarity, card.rarity, "Error in configuration, rarity does not match!");
        self.name = card.name.clone();
        self.efficiency = card.efficiency.clone();
        self.priority_allocation = card.priority_allocation;
        self.range = card.range.as_ref().unwrap().clone();
        self.effect = card.effect.as_ref().unwrap().clone();
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RarityRanges {
   pub common: PowerRange,
    pub uncommon: PowerRange,
    pub rare: PowerRange,
    pub epic: PowerRange,
    pub legendary: PowerRange 
}

impl RarityRanges {
    pub fn get_power(&self, rng: &mut ThreadRng, rarity: &Rarity) -> i32 {
        match rarity {
            Rarity::Common => self.common.get(rng),
            Rarity::Uncommon => self.uncommon.get(rng),
            Rarity::Rare => self.rare.get(rng),
            Rarity::Epic => self.epic.get(rng),
            Rarity::Legendary => self.legendary.get(rng),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct PowerRange {
    pub min: i32,
    pub max: i32,
}

impl PowerRange {
    pub fn new(min: i32, max: i32) -> Self {
        PowerRange { min, max }
    }

    pub fn get(&self, rng: &mut ThreadRng) -> i32 {
        if rng.gen_bool(0.5) {
            self.min
        } else {
            self.max
        }
    }
}

impl Default for RarityRanges {
    fn default() -> Self {
        RarityRanges { 
            common:     PowerRange::new(4,   4), 
            uncommon:   PowerRange::new(5,   8), 
            rare:       PowerRange::new(10, 12), 
            epic:       PowerRange::new(14, 16), 
            legendary:  PowerRange::new(18, 20) }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RarityPriorityModifiers {
    pub common: f32,
    pub uncommon: f32,
    pub rare: f32,
    pub epic: f32,
    pub legendary: f32
}

impl Default for RarityPriorityModifiers {
    fn default() -> Self {
        RarityPriorityModifiers { 
            common: 4.0, 
            uncommon: 3.5, 
            rare: 1.75, 
            epic: 1.5, 
            legendary: 0.75 
        }
    }
}

impl RarityPriorityModifiers {
    fn get_modifier(&self, rarity: &Rarity) -> f32 {
        match rarity {
            Rarity::Common => self.common,
            Rarity::Uncommon => self.uncommon,
            Rarity::Rare => self.rare,
            Rarity::Epic => self.epic,
            Rarity::Legendary => self.legendary,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Config {
    pub rarity_ranges: RarityRanges,
    pub power_to_priority: RarityPriorityModifiers,
    pub damage_range_modifiers: RangeModifiers,
    pub heal_range_modifiers: RangeModifiers,
    pub acid_heal_range_modifiers: RangeModifiers,
    pub shield_heal_range_modifiers: RangeModifiers,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rarity_ranges: RarityRanges::default(),
            power_to_priority: RarityPriorityModifiers::default(),
            damage_range_modifiers: RangeModifiers::new(Effect::Damage(0)),
            heal_range_modifiers: RangeModifiers::new(Effect::Heal(0)),
            acid_heal_range_modifiers: RangeModifiers::new(Effect::AcidHeal(0)),
            shield_heal_range_modifiers: RangeModifiers::new(Effect::Shield(0)),
        }
    }
}

impl Config {
    fn get_effect_range_modifier(&self, effect: &Effect, range: &Range) -> f32 {
        match *effect {
            Effect::Heal(_) => self.heal_range_modifiers.get_modifier(range),
            Effect::AcidHeal(_) => self.acid_heal_range_modifiers.get_modifier(range),
            Effect::Damage(_) => self.damage_range_modifiers.get_modifier(range),
            Effect::Shield(_) => self.shield_heal_range_modifiers.get_modifier(range),
        }
    }
}

pub fn load_config() -> Config {
    let mut config_file = OpenOptions::new().read(true).write(true).create(true).open(crate::PATH).expect("Could not load file!");
    let mut contents = String::new();
    config_file.read_to_string(&mut contents).expect("Could not read file!");
    let (config, config_empty) = if contents.trim().is_empty() {
        (Config::default(), true)
    } else {
        match serde_json::from_str(contents.as_str()) {
            Ok(deserialize) => (deserialize, false),
            Err(_) => {
                let _ = config_file.set_len(0); 
                let _ = config_file.flush();
                (Config::default(), true) 
            }
        }
    };
    if config_empty {
        let _ = config_file.write_all(serde_json::to_string_pretty(&config).unwrap().as_bytes());
    }
    config
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

    pub fn new(effect: Effect) -> Self {
        match effect {
            Effect::Damage(_) => RangeModifiers { single: 1.0, multiple: 1.25, aoe: 0.875, aoe_extended: 0.75 },
            Effect::Heal(_) => RangeModifiers { single: 1.5, multiple: 2.0, aoe: 1.25, aoe_extended: 1.5 },
            Effect::AcidHeal(_) => RangeModifiers { single: 1.25, multiple: 1.25, aoe: 1.75, aoe_extended: 2.0 },
            Effect::Shield(_) => RangeModifiers { single: 1.5, multiple: 2.0, aoe: 1.25, aoe_extended: 1.5 },
        }
    }
}

impl Default for RangeModifiers {
    fn default() -> Self {
        RangeModifiers { single: 1.0, multiple: 1.0, aoe: 1.0, aoe_extended: 1.0 }
    }
}

use rand::{rngs::ThreadRng, Rng};

pub const DEFAULT_PRIORITY: i32 = 11;
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "magnitude")]
pub enum Effect {
    Heal(i32),
    AcidHeal(i32),
    Damage(i32),
    Shield(i32),
}

impl Effect {
    pub fn to_string(&self) -> String {
        match self {
            Effect::Heal(magnitude) => format!("Heal ({})", magnitude),
            Effect::AcidHeal(magnitude) => format!("Acid Heal ({})", magnitude),
            Effect::Damage(magnitude) => format!("Damage ({})", magnitude),
            Effect::Shield(magnitude) => format!("Shield ({})", magnitude),
        }
    }
}

pub fn cost_from_effect(effect: Effect, budget: i32, range: &Option<Range>, config: &Config) -> (Option<Effect>, i32) {
    let effect_modifier = config.get_effect_range_modifier(&effect, &range.as_ref().expect("No Range in card... How?"));
    let budget = apply_multiplier(budget, 1.0 / effect_modifier);
    match effect {
        Effect::Heal(_) => (Some(Effect::Heal(budget)), apply_multiplier(budget, effect_modifier)),
        Effect::AcidHeal(_) => (Some(Effect::AcidHeal(budget)), apply_multiplier(budget, effect_modifier)),
        Effect::Damage(_) => (Some(Effect::Damage(budget)), apply_multiplier(budget, effect_modifier)),
        Effect::Shield(_) => (Some(Effect::Shield(budget)), apply_multiplier(budget, effect_modifier)),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
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
    pub efficiency: Efficiency,
    pub priority: i32,
    pub barnacles: i32,
    pub rarity: Rarity,
    pub priority_allocation: i32,
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

pub fn priority_from_budget(budget: i32, rarity: &Rarity, config: &Config) -> i32 {
    if budget < 0 {
        0
    } else {
        let value = (apply_multiplier(budget, config.power_to_priority.get_modifier(rarity))).clamp(1, DEFAULT_PRIORITY);
        if value % 2 == 0 {
            value + 1
        } else {
            value
        }
    }
}

impl Card {
    pub fn new(name: String, rarity: Rarity, efficiency: Efficiency, config: Config) -> Card {
        let mut rng = rand::thread_rng();
        Card {
            name, 
            budget: config.rarity_ranges.get_power(&mut rng, &rarity),
            rarity,
            priority: DEFAULT_PRIORITY,
            efficiency,
            priority_allocation: 0,
            barnacles: 100000000,
            range: None,
            effect: None,
            config
        }
    }

    pub fn with_priority_allocation(&mut self, priority_allocation: i32) -> &mut Card {
        self.priority_allocation = priority_allocation;
        self.budget -= priority_allocation;
        self
    }

    pub fn with_range(&mut self, range: Range) -> &mut Card {
        let cost = cost_from_range(&range);
        self.range = Some(range);
        self.budget -= cost;
        self
    }

    pub fn with_effect(&mut self, effect: Effect) -> &mut Card {
        let (created_effect, used) =
            cost_from_effect(effect, self.budget, &self.range, &self.config);
        self.budget -= used;
        self.effect = created_effect;
        self
    }

    pub fn get_withdraw(&self) -> i32 {
        apply_multiplier(self.barnacles, 1.0/3.0).max(1)
    }

    pub fn print_budget_mut(&mut self) -> &mut Card {
        println!("Card power budget: {}", self.budget);
        self
    }

    pub fn to_string(&self) -> String {
        // Rarity, Effect, Cost, Recast Cost
        String::from(
            format!("{}: \n\tPriority: {}\n\tRarity: {:?}\n\tCast: {} barnacles\n\tWithdraw: {} barnacles\n\tEffect: {}, Range: {:?}", self.name, self.priority, self.rarity, self.barnacles, self.get_withdraw(), self.effect.clone().unwrap().to_string(), self.range.clone().unwrap())
        )
    }

    pub fn build(&mut self) -> Result<Card, String> {
        self.priority -= priority_from_budget(self.priority_allocation, &self.rarity, &self.config);
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
    apply_multiplier(barnacles_from_effect(&card.effect) + cost_from_range(&card.range.as_ref().unwrap_or(&Range::Single)), 1.0 / multiplier_from_efficiency(&card.efficiency))
}

fn barnacles_from_effect(effect: &Option<Effect>) -> i32 {
    if effect.as_ref().is_none() { return 100000000; }
    match effect.as_ref().unwrap() {
        Effect::Heal(magnitude) => apply_multiplier(*magnitude, 1.25),
        Effect::AcidHeal(magnitude) => apply_multiplier(*magnitude, 1.125),
        Effect::Damage(magnitude) => *magnitude,
        Effect::Shield(magnitude) => apply_multiplier(*magnitude, 1.375),
    }
}
