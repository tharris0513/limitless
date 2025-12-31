use crate::models::{Ability, Class};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassAbilityMapping {
    pub ability_id: String,
    pub unlock_level: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassWithAbilities {
    #[serde(flatten)]
    pub class: Class,
    pub abilities: Vec<ClassAbilityMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfigExport {
    pub version: String,
    pub exported_at: String,
    pub classes: Vec<ClassWithAbilities>,
    pub abilities: Vec<Ability>,
}

impl GameConfigExport {
    pub fn new(classes: Vec<ClassWithAbilities>, abilities: Vec<Ability>) -> Self {
        Self {
            version: "1.0".to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            classes,
            abilities,
        }
    }
}
