import React from 'react';
import styles from './TermsOfService.module.css';

export const TermsOfService: React.FC = () => {
  return (
    <div className={styles.container}>
      <div className={styles.content}>
        <h1 className={styles.title}>Terms of Service</h1>
        <p className={styles.lastUpdated}>Last Updated: December 29, 2025</p>

        <div className={styles.section}>
          <h2>Welcome to Limitless!</h2>
          <p>
            By accessing and playing Limitless, you agree to be bound by these Terms of Service. 
            If you do not agree to these terms, please do not use our game.
          </p>
        </div>

        <div className={styles.section}>
          <h2>1. Account Registration</h2>
          <p>
            You must be at least 13 years old to create an account and play Limitless. 
            By creating an account, you confirm that you meet this age requirement.
          </p>
          <p>
            You are responsible for maintaining the security of your account and are fully 
            responsible for all activities that occur under your account.
          </p>
        </div>

        <div className={styles.section}>
          <h2>2. Acceptable Use</h2>
          <p>You agree to use Limitless only for lawful purposes. Specifically, you will NOT:</p>
          <ul>
            <li>Attempt to exploit, hack, or break the game in any way</li>
            <li>Use cheats, automation tools, bots, or any third-party software to gain an unfair advantage</li>
            <li>Harass, bully, threaten, or be rude to other players</li>
            <li>Share offensive, inappropriate, or illegal content</li>
            <li>Impersonate other users or administrators</li>
            <li>Attempt to gain unauthorized access to other users' accounts</li>
            <li>Intentionally overload or disrupt the game servers</li>
            <li>Sell, trade, or transfer your account to others</li>
          </ul>
        </div>

        <div className={styles.section}>
          <h2>3. Player Conduct</h2>
          <p>
            We strive to maintain a friendly and welcoming community. Players are expected to:
          </p>
          <ul>
            <li>Be respectful and courteous to all other players</li>
            <li>Use appropriate language and avoid offensive content</li>
            <li>Report bugs and exploits to administrators rather than abusing them</li>
            <li>Play fairly and within the spirit of the game</li>
          </ul>
        </div>

        <div className={styles.section}>
          <h2>4. Game Content and Ownership</h2>
          <p>
            All content in Limitless, including but not limited to text, graphics, game mechanics, 
            characters, and code, is the property of Limitless and is protected by copyright and 
            other intellectual property laws.
          </p>
          <p>
            Any content you create within the game (character names, in-game actions, etc.) 
            becomes part of the game and may be used by Limitless for game operations.
          </p>
        </div>

        <div className={styles.section}>
          <h2>5. Account Termination</h2>
          <p>
            We reserve the right to suspend or terminate your account at any time for violating 
            these Terms of Service, particularly for:
          </p>
          <ul>
            <li>Cheating or exploiting game mechanics</li>
            <li>Harassment or abusive behavior toward other players</li>
            <li>Repeatedly violating community guidelines</li>
            <li>Attempting to damage or disrupt the game</li>
          </ul>
          <p>
            Account terminations may be temporary or permanent, depending on the severity of 
            the violation.
          </p>
        </div>

        <div className={styles.section}>
          <h2>6. Service Availability</h2>
          <p>
            Limitless is provided "as is" without any guarantees of availability or uptime. 
            We may need to perform maintenance, updates, or fixes that temporarily make the 
            game unavailable.
          </p>
          <p>
            We are not responsible for any loss of game progress, items, or characters due to 
            technical issues, bugs, or necessary maintenance.
          </p>
        </div>

        <div className={styles.section}>
          <h2>7. Disclaimer of Warranties</h2>
          <p>
            Limitless is a free-to-play game provided for entertainment purposes. We make no 
            warranties or representations about the accuracy or completeness of the game content 
            or its suitability for any particular purpose.
          </p>
          <p>
            You use Limitless at your own risk. We are not liable for any damages arising from 
            your use of the game.
          </p>
        </div>

        <div className={styles.section}>
          <h2>8. Changes to Terms</h2>
          <p>
            We reserve the right to modify these Terms of Service at any time. Changes will be 
            posted on this page with an updated "Last Updated" date. Continued use of Limitless 
            after changes constitutes acceptance of the new terms.
          </p>
        </div>

        <div className={styles.section}>
          <h2>9. Contact</h2>
          <p>
            If you have questions about these Terms of Service or need to report a violation, 
            please contact the game administrators through the in-game admin panel or community channels.
          </p>
        </div>

        <div className={styles.footer}>
          <p>
            By playing Limitless, you acknowledge that you have read, understood, and agree to be 
            bound by these Terms of Service.
          </p>
          <p className={styles.thankYou}>
            Thank you for being part of our community and playing fairly!
          </p>
        </div>
      </div>
    </div>
  );
};
