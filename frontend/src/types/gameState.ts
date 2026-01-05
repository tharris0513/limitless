// Game state types for tracking player progress and combat

export interface EnemyState {
  id: string;
  name: string;
  introductionText?: string;
  health: number;
  maxHealth: number;
  mana?: number;
  maxMana?: number;
  level: number;
  stats: {
    might: number;
    defense: number;
    magic: number;
    resistance: number;
    agility: number;
  };
  abilities?: string[];
}

export interface CombatState {
  inCombat: true;
  adventureId: string;
  adventureName: string;
  turnNumber: number;
  playerHealth: number;
  playerMana: number;
  playerBuffs?: Array<{
    id: string;
    name: string;
    duration: number;
    effect: string;
  }>;
  playerDebuffs?: Array<{
    id: string;
    name: string;
    duration: number;
    effect: string;
  }>;
  enemy: EnemyState;
  enemyBuffs?: Array<{
    id: string;
    name: string;
    duration: number;
    effect: string;
  }>;
  enemyDebuffs?: Array<{
    id: string;
    name: string;
    duration: number;
    effect: string;
  }>;
  combatLog: Array<{
    turn: number;
    message: string;
    timestamp: string;
  }>;
}

export interface ChoiceOption {
  id: string;
  text: string;
  description?: string;
  requirements?: {
    stat?: string;
    minValue?: number;
    item?: string;
  };
}

export interface ChoiceState {
  inChoice: true;
  adventureId: string;
  adventureName: string;
  sceneId: string;
  sceneName: string;
  sceneDescription: string;
  choices: ChoiceOption[];
  previousChoices?: Array<{
    sceneId: string;
    choiceId: string;
    timestamp: string;
  }>;
}

export interface IdleState {
  inCombat: false;
  inChoice: false;
}

// Union type for all possible game states
export type GameState = IdleState | CombatState | ChoiceState;

// Helper type guards
export function isCombatState(state: GameState): state is CombatState {
  return 'inCombat' in state && state.inCombat === true;
}

export function isChoiceState(state: GameState): state is ChoiceState {
  return 'inChoice' in state && state.inChoice === true;
}

export function isIdleState(state: GameState): state is IdleState {
  return !isCombatState(state) && !isChoiceState(state);
}

// Default idle state
export const DEFAULT_IDLE_STATE: IdleState = {
  inCombat: false,
  inChoice: false,
};
