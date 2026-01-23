# General Principles

- Read the other md files for understanding on mechanics of how the game works:
  - `SPELLBLADE.md` has information on the spellblade class.
  - `DAMAGE_FORMULAS.md` contains information on how the damage formulas work.
  - `GAME_MECHANICS.md` explains how the combat system works.

# Backend

- The backend should always be in control of everything related to state and progression.

# Frontend

- The frontend is specifically for viewing results from the backend.
- The front end should only be able to request that actions be performed, such as attacking, buffing when out of combat, using abilities, or fleeing during combat.
