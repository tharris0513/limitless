# Damage Formula System

## Overview

The damage formula system allows you to create complex, dynamic ability calculations using mathematical expressions. Formulas are stored as strings and evaluated safely on the backend.

## Available Variables

When writing formulas, you have access to these character stat variables:

### Caster Stats

- `might` - Physical power
- `defense` - Physical defense
- `magic` - Magical power
- `resistance` - Magical defense
- `agility` - Speed/agility
- `level` - Character level

### Random Functions

- `rand()` - Returns a random float between 0.0 and 1.0
- `randRange(min, max)` - Returns a random float between min and max (inclusive)

**Important:** All damage calculations automatically include a ±10% damage variation unless you explicitly use `rand()` or `randRange()` in your formula.

### Target Stats (when applicable)

- `target_defense` - Target's physical defense
- `target_resistance` - Target's magical defense
- `target_level` - Target's level
- `target_health` - Target's current health
- `target_max_health` - Target's maximum health

## Formula Types

### Damage Formula

Calculates the damage dealt by an ability.

**Examples:**

```
(might * 0.8) + 15              # Physical damage (auto ±10% variation)
(magic * 1.2) + (level * 3)     # Magic damage (auto ±10% variation)
((might * 0.5) + (magic * 0.5) + 20) * randRange(0.9, 1.3)  # Hybrid with custom variation
```

### Heal Formula

Calculates the amount healed by an ability.

**Examples:**

```
(magic * 1.2) + 20              # Magic-based healing
(level * 5) + 10                # Level-based healing
magic + (level * 2)             # Combined scaling
```

### Effect Formula

Calculates the strength of status effects.

**Examples:**

```
magic * 0.5                     # Shield amount
agility * 0.1                   # Slow percentage
(might + magic) / 2             # Average stat buff
```

## Supported Operators

- `+` Addition
- `-` Subtraction
- `*` Multiplication
- `/` Division
- `%` Modulo
- `^` Exponentiation
- `()` Parentheses for grouping

## Advanced Examples

### Random Damage Variation

```
# Critical strike (25-40% bonus damage, bypasses auto-variation)
(might * 1.5) * randRange(1.25, 1.4)

# Chaos bolt (0-200% of base magic, bypasses auto-variation)
((magic * 1.2) + 20) * (rand() * 2)

# Unstable attack (50-150% variation, bypasses auto-variation)
((might * 0.8) + 15) * randRange(0.5, 1.5)
```

### Critical Hit Damage

```
might * 2.5
```

### Defense-Penetrating Attack

```
might - (target_defense * 0.5)
```

### Percentage-Based Healing

```
target_max_health * 0.3
```

### Level-Scaled Damage

```
(might * 0.5) + (level * 5) + 10
```

### Agility-Based Multi-Hit

```
might + (agility * 0.2)
```

## Creating Abilities with Formulas

### In the Admin UI

1. Navigate to **Admin Panel** → **Manage Classes**
2. Click the **Abilities** tab
3. Click **New Ability**
4. Fill in the basic fields (Name, Description, Type, etc.)
5. In the **Formulas (Advanced)** section, enter your formula:
   - **Damage Formula**: For attack abilities
   - **Heal Formula**: For healing abilities
   - **Effect Formula**: For buffs/debuffs/shields
6. Click **Save**

The backend will validate your formula immediately and show an error if it's invalid.

### Example: Fire Strike

```
Name: Fire Strike
Description: A powerful fire-based attack
Ability Type: Active
Mana Cost: 15
Cooldown: 2

Damage Formula: (magic * 1.5) + 10
```

This creates an ability that deals damage based on the caster's magic stat.

### Example: Healing Touch

```
Name: Healing Touch
Description: Restore health to yourself
Ability Type: Active
Mana Cost: 20
Cooldown: 3

Heal Formula: (magic * 2) + (level * 5)
```

This creates a healing ability that scales with both magic and level.

## Legacy Support

The system maintains backward compatibility with the old `damageMultiplier` field:

- If a `damageFormula` is present, it takes precedence
- If no `damageFormula` exists, falls back to `damageMultiplier`
- Both can coexist in the database

## Formula Validation

Formulas are validated when:

1. Creating a new ability
2. Updating an existing ability

Invalid formulas will return an error immediately, preventing bad data from being saved.

## Testing Formulas

You can test formulas in the backend using the `DamageCalculator::test_formula()` method:

```rust
use crate::damage_calculator::DamageCalculator;
use crate::models::CharacterStats;

let stats = CharacterStats {
    might: 12,
    defense: 8,
    magic: 10,
    resistance: 6,
    agility: 15,
    adventures: 0,
    max_adventures: 10,
};

let result = DamageCalculator::test_formula("(might * 0.8) + 15", &stats, 1);
// result = Ok(24)
```

## Best Practices

1. **Keep formulas simple** - Complex formulas are harder to balance
2. **Test thoroughly** - Verify formulas with different stat values
3. **Use parentheses** - Make order of operations explicit
4. **Round intelligently** - Formula results are converted to integers
5. **Prevent negatives** - Results are clamped to 0 minimum
6. **Document intent** - Use clear ability descriptions
7. **Control randomness** - Use `randRange(min, max)` for precise control over variation
8. **Disable auto-variation** - Include `rand()` or `randRange()` to bypass automatic ±10% variation
9. **Reasonable ranges** - Keep random multipliers between 0.5x and 2.0x for game balance

## Security

- All formulas run in a sandboxed environment
- Only mathematical expressions are allowed
- No file system or network access
- No ability to execute arbitrary code
- Variables are strictly typed and validated

## Performance

- Formulas are parsed once and cached
- Evaluation is fast (microseconds)
- No performance impact on gameplay
