// Profanity filter utility for username validation
// This list includes common inappropriate words and variations
// Note: This is a basic filter and should be expanded based on your needs

const PROFANITY_LIST = [
  // Common profanity
  'fuck',
  'shit',
  'ass',
  'bitch',
  'damn',
  'hell',
  'crap',
  'piss',
  'dick',
  'cock',
  'pussy',
  'cunt',
  'bastard',
  'slut',
  'whore',
  'fag',
  'nigger',
  'nigga',
  'retard',
  'rape',
  'nazi',
  'hitler',
  'kill',
  'kys',
  'suicide',
  'die',
  'death',

  // Variants and leetspeak
  'fuk',
  'fck',
  'sht',
  'btch',
  'dmn',
  'dck',
  'dik',
  'psy',
  'cnt',
  'bastrd',
  'f.u.c.k',
  'f*ck',
  'f**k',
  'sh*t',
  'a**',
  'b*tch',
  'd*ck',
  'p*ssy',

  // Offensive terms
  'sex',
  'porn',
  'xxx',
  'anal',
  'blowjob',
  'cumshot',
  'dildo',
  'masturbate',
  'orgasm',
  'penis',
  'vagina',
  'breast',
  'boob',
  'tit',
  'nude',
  'naked',

  // Hate speech and slurs
  'racist',
  'sexist',
  'bigot',
  'supremacy',
  'genocide',
  'terrorism',
  'terrorist',

  // Impersonation attempts
  'admin',
  'moderator',
  'support',
  'staff',
  'official',
  'system',
  'bot',

  // Numbers that spell words (1337 speak)
  '69',
  '420',
  '1488',
  '88',
];

// Additional patterns to check
const PROFANITY_PATTERNS = [
  /n+[i1l!]+[g9]+[g9]+[e3a4@]+r+/i, // n-word variations
  /f+[u*]+c+k+/i, // f-word variations
  /s+h+[i1!]+t+/i, // s-word variations
  /b+[i1!]+t+c+h+/i, // b-word variations
  /a+s+s+h+o+l+e+/i, // a-word variations
  /d+[i1!]+c+k+/i, // d-word variations
  /c+[u*]+n+t+/i, // c-word variations
];

/**
 * Checks if a username contains profanity or inappropriate content
 * @param username - The username to check
 * @returns true if profanity is detected, false otherwise
 */
export function containsProfanity(username: string): boolean {
  const normalized = username.toLowerCase().replace(/[\s._-]/g, '');

  // Check against word list
  for (const word of PROFANITY_LIST) {
    if (normalized.includes(word)) {
      return true;
    }
  }

  // Check against patterns
  for (const pattern of PROFANITY_PATTERNS) {
    if (pattern.test(normalized)) {
      return true;
    }
  }

  return false;
}

/**
 * Gets a user-friendly error message for profanity detection
 * @returns Error message to display to the user
 */
export function getProfanityErrorMessage(): string {
  return 'Username contains inappropriate content. Please choose a different username.';
}

/**
 * Validates username for both format and content
 * @param username - The username to validate
 * @returns Object with isValid boolean and error message if invalid
 */
export function validateUsername(username: string): {
  isValid: boolean;
  error?: string;
} {
  // Length checks
  if (username.length < 3) {
    return {
      isValid: false,
      error: 'Username must be at least 3 characters long',
    };
  }

  if (username.length > 20) {
    return {
      isValid: false,
      error: 'Username must be no more than 20 characters',
    };
  }

  // Format check
  if (!/^[a-zA-Z0-9_-]+$/.test(username)) {
    return {
      isValid: false,
      error:
        'Username can only contain letters, numbers, underscores, and hyphens',
    };
  }

  // Profanity check
  if (containsProfanity(username)) {
    return { isValid: false, error: getProfanityErrorMessage() };
  }

  return { isValid: true };
}
