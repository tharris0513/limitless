# Spellblade

## Summary

A high damage, low defense hybrid melee spellcaster who enchants their weapons with elemental energies, unleashing powerful combination strikes. Focuses on buffing themselves and their weapons to kill their enemies quickly. Focuses on might, magic, and agility at the cost of defense and resistance.

## Starting Stats

- Max Health: 80
- Max Mana: 20
- Might: 12
- Defense: 8
- Magic: 12
- Resistance: 8
- Agility: 10

## Stat Growth Per Level

- Max Health: +10
- Max Mana: +5
- Might: +1
- Defense: +1/3 (one per three levels)
- Magic: +1
- Resistance: +1/3 (one per three levels)
- Agility: +0.5 (one per two levels)

## Abilities

### Fire Strike

- **Description**: Imbue your weapon with the power of fire, striking your enemy for physical and fire damage.
- **Level**: 1
- **Type**: Combat
- **Tags**: Combat, Fire, Physical, Strike, Enchant Fire, Enchant Weapon, Attack
- **Mana Cost**: 2
- **Cooldown**: 0
- **Damage Formula**: `(might * 1.0) + (magic * 1.0)`
- **Damage Types**: Fire, Physical
- **Effects**: If enchant weapon is unlocked, enchant your primary weapon with the power of fire for `(magic * 0.5 + weapon might)`. If dual wield is unlocked and the primary weapon is already enchanted, enchant the offhand weapon. If both weapons are already enchanted, no effect.

### Dual Wield

- **Description**: Wield a weapon in your offhand.
- **Level**: 2
- **Type**: Passive
- **Tags**: Passive, Dual Wield
- **Effects**: Allows the equip and use of an offhand weapon. Regular melee attacks will cause a second attack for full damage from the offhand weapon. The offhand weapon can be enchanted if enchant weapon is unlocked.

### Ice Strike

- **Description**: Imbue your weapon with the power of ice, striking your enemy for physical and ice damage.
- **Level**: 3
- **Type**: Combat
- **Tags**: Combat, Ice, Physical, Strike, Enchant Ice, Enchant Weapon, Attack
- **Mana Cost**: 2
- **Cooldown**: 0
- **Damage Formula**: `(might * 1.0) + (magic * 1.0)`
- **Damage Types**: Ice, Physical
- **Effects**: If enchant weapon is unlocked, enchant your primary weapon with the power of ice for `(magic * 0.5 + weapon might)`. If dual wield is unlocked and the primary weapon is already enchanted, enchant the offhand weapon. If both weapons are already enchanted, no effect.

### Enchant Weapon

- **Description**: Enchant your weapons with the power of the elements.
- **Level**: 4
- **Type**: Passive
- **Tags**: Passive, Enchant Weapon, Strike
- **Effects**: When using an elemental strike ability, enchant your weapon with the power of that element. If dual wielding, the second strike will enchant your off hand weapon. These enchantments add `(magic * 0.5 + weapon might)` extra damage during melee attacks.

### Thunder Strike

- **Description**: Imbue your weapon with the power of lightning, striking your enemy for physical and lightning damage.
- **Level**: 5
- **Type**: Combat
- **Tags**: Combat, Lightning, Physical, Strike, Enchant Lightning, Enchant Weapon, Attack
- **Mana Cost**: 2
- **Cooldown**: 0
- **Damage Formula**: `(might * 1.0) + (magic * 1.0)`
- **Damage Types**: Ice, Physical
- **Effects**: If enchant weapon is unlocked, enchant your primary weapon with the power of lightning for `(magic * 0.5 + weapon might)`. If dual wield is unlocked and the primary weapon is already enchanted, enchant the offhand weapon. If both weapons are already enchanted, no effect.

### Overdrive

- **Description**: Drop your defenses to 0 in return for a strong buff to might, magic, and agility.
- **Level**: 6
- **Type**: Buff
- **Tags**: Buff, Noncombat, Might, Defense, Magic, Resistance, Agility
- **Mana Cost**: 10
- **Turn Duration**: 10
- **Effects**:
  - Might: +20%
  - Defense: Set to 0
  - Magic: +20%
  - Resistance: Set to 0
  - Agility: +10%

### Unleash

- **Description**: Consume your combined weapon energies into a powerful attack that varies based on the elemental combination.
- **Level**: 7
- **Type**: Combat
- **Tags**: Combat, Enchant Weapon, Dual Wield, Enchant Fire, Enchant Ice, Enchant Lightning, Attack
- **Mana Cost**: 10
- **Cooldown**: 5
- **Condition**: Must have dual wield and enchant weapon, both weapons must be enchanted.
- **Effects**: Different effects depending on the elements consumed:
  - Fire + Fire -> **Volcanic Outburst**: Deal `(magic * 4.0)` fire damage and inflict a damage over time effect (burn) for `(magic * 1.0)` per turn.
  - Fire + Ice -> **Thermal Crash**: Deal `(magic * 3.0)` fire and ice damage and prevent enemy special abilities for three turns.
  - Fire + Lightning -> **Fulmination**: Deal `(magic * 3.0)` fire and lightning damage, and melt enemy armor (50% reduction) for four turns.
  - Ice + Ice -> **Glacial Blast**: Deal `(magic * 4.0)` ice damage and freeze your target, stunning them for one turn.
  - Ice + Lightning -> **Winter Storm**: Deal `(magic * 3.0)` ice and lightning damage, reducing enemy stats by 10% for four turns.
  - Lightning + Lightning -> **Thunderous Explosion**: Deal `(magic * 5.0)` lightning damage.
