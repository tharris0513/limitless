use serde::{Deserialize, Serialize};

/// Effect type enum - active or passive
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    Active,
    Passive,
}

/// Active effect type - what the active effect does
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActiveEffectType {
    Damage,
    Heal,
    Buff,
    Debuff,
}

/// Passive effect type - specific passive abilities
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PassiveEffectType {
    DualWield,
    EnchantWeapon,
    EnchantWeaponFire,
    EnchantWeaponFrost,
    EnchantWeaponLightning,
    IncreasedCrit,
    Lifesteal,
    Thorns,
    ManaRegen,
    HealthRegen,
    DodgeBonus,
    ArmorPierce,
    SpellAmp,
    IronSkin,
    ArcaneShield,
    Berserker,
    FirstStrike,
    CounterAttack,
    VampiricAura,
}

impl PassiveEffectType {
    /// Convert enum to string representation for legacy compatibility
    pub fn as_str(&self) -> &'static str {
        match self {
            PassiveEffectType::DualWield => "dual_wield",
            PassiveEffectType::EnchantWeapon => "enchant_weapon",
            PassiveEffectType::EnchantWeaponFire => "enchant_weapon_fire",
            PassiveEffectType::EnchantWeaponFrost => "enchant_weapon_frost",
            PassiveEffectType::EnchantWeaponLightning => "enchant_weapon_lightning",
            PassiveEffectType::IncreasedCrit => "increased_crit",
            PassiveEffectType::Lifesteal => "lifesteal",
            PassiveEffectType::Thorns => "thorns",
            PassiveEffectType::ManaRegen => "mana_regen",
            PassiveEffectType::HealthRegen => "health_regen",
            PassiveEffectType::DodgeBonus => "dodge_bonus",
            PassiveEffectType::ArmorPierce => "armor_pierce",
            PassiveEffectType::SpellAmp => "spell_amp",
            PassiveEffectType::IronSkin => "iron_skin",
            PassiveEffectType::ArcaneShield => "arcane_shield",
            PassiveEffectType::Berserker => "berserker",
            PassiveEffectType::FirstStrike => "first_strike",
            PassiveEffectType::CounterAttack => "counter_attack",
            PassiveEffectType::VampiricAura => "vampiric_aura",
        }
    }

    /// Get the element name for enchantment types
    pub fn enchantment_element(&self) -> Option<&'static str> {
        match self {
            PassiveEffectType::EnchantWeaponFire => Some("fire"),
            PassiveEffectType::EnchantWeaponFrost => Some("frost"),
            PassiveEffectType::EnchantWeaponLightning => Some("lightning"),
            _ => None,
        }
    }
}

/// Stat modifier for passive effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatModifier {
    pub stat: String, // 'might', 'defense', 'magic', 'resistance', 'agility', 'maxHealth', 'maxMana'
    pub value: f64,
    #[serde(rename = "type")]
    pub modifier_type: String, // 'flat', 'percentage', 'set'
}

/// Active buff tracking for noncombat abilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBuff {
    #[serde(rename = "abilityId")]
    pub ability_id: String,
    #[serde(rename = "abilityName")]
    pub ability_name: String,
    #[serde(rename = "remainingAdventures")]
    pub remaining_adventures: i64,
    pub effects: Vec<AbilityEffect>,
}

/// Represents a single effect within an ability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityEffect {
    pub id: String, // Unique ID for this effect within the ability

    #[serde(rename = "effectType")]
    pub effect_type: EffectType,

    // For active effects
    #[serde(rename = "activeType", skip_serializing_if = "Option::is_none")]
    pub active_type: Option<ActiveEffectType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>, // The calculation formula

    #[serde(rename = "attackDescription", skip_serializing_if = "Option::is_none")]
    pub attack_description: Option<String>, // Description template for this effect

    // For passive effects
    #[serde(rename = "passiveType", skip_serializing_if = "Option::is_none")]
    pub passive_type: Option<PassiveEffectType>,

    #[serde(rename = "passiveMode", skip_serializing_if = "Option::is_none")]
    pub passive_mode: Option<String>, // 'effect' or 'stat_modifier'

    #[serde(rename = "statModifier", skip_serializing_if = "Option::is_none")]
    pub stat_modifier: Option<StatModifier>,
}

/// Default ability type for backward compatibility
fn default_ability_type() -> String {
    "combat".to_string()
}

/// Ability represents a skill or spell that characters can use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ability {
    pub id: String,
    pub name: String,
    pub description: String,

    // Ability type: "combat" = shows in combat action bar, "passive" = hidden/automatic, "noncombat" = buff used before combat
    #[serde(rename = "abilityType", default = "default_ability_type")]
    pub ability_type: String,

    // Multi-effect system (new)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effects: Vec<AbilityEffect>,

    #[serde(rename = "manaCost", default, skip_serializing_if = "Option::is_none")]
    pub mana_cost: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown: Option<i64>,

    // For noncombat abilities - how many adventures the buff lasts
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<i64>,

    // Legacy fields (deprecated but kept for backward compatibility)
    #[serde(rename = "damageFormula", skip_serializing_if = "Option::is_none")]
    pub damage_formula: Option<String>, // e.g., "(might * 0.8) + 15"
    #[serde(rename = "healFormula", skip_serializing_if = "Option::is_none")]
    pub heal_formula: Option<String>, // e.g., "(magic * 1.2) + 20"
    #[serde(rename = "effectFormula", skip_serializing_if = "Option::is_none")]
    pub effect_formula: Option<String>, // For complex status effects
    #[serde(rename = "passiveEffect", skip_serializing_if = "Option::is_none")]
    pub passive_effect: Option<String>, // e.g., "dual_wield", "increased_crit", etc.
    #[serde(rename = "attackDescription", skip_serializing_if = "Option::is_none")]
    pub attack_description: Option<String>, // e.g., "You unleash ${name} for ${damage} damage!"
}

/// Available passive ability effects
pub const PASSIVE_EFFECTS: &[(&str, &str)] = &[
    ("dual_wield", "Dual Wield - Attack with both weapons"),
    (
        "enchant_weapon",
        "Weapon Enchantment - Allows elemental enchantments on weapons",
    ),
    (
        "enchant_weapon_fire",
        "Fire Enchantment - Adds fire damage to weapon attacks",
    ),
    (
        "enchant_weapon_frost",
        "Frost Enchantment - Adds frost damage to weapon attacks",
    ),
    (
        "enchant_weapon_lightning",
        "Lightning Enchantment - Adds lightning damage to weapon attacks",
    ),
    (
        "increased_crit",
        "Increased Critical - +10% critical hit chance",
    ),
    ("lifesteal", "Lifesteal - Heal for 15% of damage dealt"),
    ("thorns", "Thorns - Reflect 20% of damage taken"),
    ("mana_regen", "Mana Regeneration - Restore 5 mana per turn"),
    (
        "health_regen",
        "Health Regeneration - Restore 10 health per turn",
    ),
    ("dodge_bonus", "Dodge Bonus - +15% chance to dodge attacks"),
    (
        "armor_pierce",
        "Armor Piercing - Ignore 25% of enemy defense",
    ),
    ("spell_amp", "Spell Amplification - +20% magic damage"),
    ("iron_skin", "Iron Skin - +15% physical damage reduction"),
    (
        "arcane_shield",
        "Arcane Shield - +15% magical damage reduction",
    ),
    ("berserker", "Berserker - +5% damage per 10% missing health"),
    (
        "first_strike",
        "First Strike - Always attack first in combat",
    ),
    (
        "counter_attack",
        "Counter Attack - 30% chance to attack when hit",
    ),
    ("vampiric_aura", "Vampiric Aura - Restore health on kill"),
];

impl Ability {
    /// Get all active damage effects
    pub fn damage_effects(&self) -> Vec<&AbilityEffect> {
        self.effects
            .iter()
            .filter(|e| {
                e.effect_type == EffectType::Active
                    && e.active_type == Some(ActiveEffectType::Damage)
            })
            .collect()
    }

    /// Get all active heal effects
    pub fn heal_effects(&self) -> Vec<&AbilityEffect> {
        self.effects
            .iter()
            .filter(|e| {
                e.effect_type == EffectType::Active && e.active_type == Some(ActiveEffectType::Heal)
            })
            .collect()
    }

    /// Check if ability has a specific passive effect
    pub fn has_passive(&self, passive_type: PassiveEffectType) -> bool {
        self.effects.iter().any(|e| {
            e.effect_type == EffectType::Passive && e.passive_type == Some(passive_type)
        }) ||
        // Legacy fallback (remove after migration)
        (self.ability_type == "passive" && self.passive_effect.as_deref() == Some(passive_type.as_str()))
    }

    /// Get all passive effects
    pub fn passive_effects(&self) -> Vec<&AbilityEffect> {
        self.effects
            .iter()
            .filter(|e| e.effect_type == EffectType::Passive)
            .collect()
    }

    /// Get the weapon enchantment type if this ability provides one
    /// Returns Some(PassiveEffectType::EnchantWeaponFire), etc., or None
    pub fn get_enchantment_type(&self) -> Option<PassiveEffectType> {
        for effect in &self.effects {
            if effect.effect_type == EffectType::Passive {
                if let Some(passive_type) = effect.passive_type {
                    match passive_type {
                        PassiveEffectType::EnchantWeaponFire
                        | PassiveEffectType::EnchantWeaponFrost
                        | PassiveEffectType::EnchantWeaponLightning => {
                            return Some(passive_type);
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }

    /// Is this ability usable in combat? (not passive-only)
    pub fn is_combat_ability(&self) -> bool {
        self.ability_type == "combat"
    }

    /// Check if this ability provides weapon enchantment damage
    /// (has both passive enchantment effect and active damage effect)
    pub fn provides_enchantment_damage(&self) -> bool {
        let has_enchant_passive = self.effects.iter().any(|e| {
            e.effect_type == EffectType::Passive
                && matches!(
                    e.passive_type,
                    Some(PassiveEffectType::EnchantWeaponFire)
                        | Some(PassiveEffectType::EnchantWeaponFrost)
                        | Some(PassiveEffectType::EnchantWeaponLightning)
                )
        });

        let has_damage_effect = self.effects.iter().any(|e| {
            e.effect_type == EffectType::Active && e.active_type == Some(ActiveEffectType::Damage)
        });

        has_enchant_passive && has_damage_effect
    }
}

/// ClassAbility maps abilities to classes with unlock levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassAbility {
    pub class_id: String,
    pub ability_id: String,
    pub unlock_level: i64,
}

/// CharacterAbility tracks which abilities a character has unlocked
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAbility {
    pub character_id: String,
    pub ability_id: String,
    pub unlocked_at_level: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability: Option<Ability>, // Joined ability data
}
