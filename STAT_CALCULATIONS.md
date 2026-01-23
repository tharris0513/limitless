# Stat Calcuations

The order of calculation should be that any effects which set a stat directly should be done first, then flat bonuses, then multiplactive bonuses, which looks something like this:

`(base stat OR set stat + flat bonuses) * (1 * multiplicative bonuses)`

For example:

- A character has 10 might. They have a buff that set their might to 5, and another buff that adds +10 might. This should first set their might to 5 then add 10 to give a total of 15 might. `5 + 10`.

- A character has 20 resistance. They have a buff that gives +10 to resistance, a buff that gives 20%, and another buff that gives an additional 10%. This should calculate as `(20 + 10) * (1 + (.20 + .10))`, for a total of 39 resistance.

- A character has 10 agility. They have a buff that sets their agility to 0, then a percentage buff of 50%. This should calculate to `0 * (1 + .5)`, or 0.

- A charcter has 10 defense. They have a buff that adds 10 defense, and another buff that reduces defense by 20%. This should calculate to `(10 + 10) * (1 + -.20)`, or 16.
