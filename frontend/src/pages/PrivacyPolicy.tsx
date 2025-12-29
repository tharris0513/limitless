import React from 'react';
import styles from './TermsOfService.module.css';

export const PrivacyPolicy: React.FC = () => {
  return (
    <div className={styles.container}>
      <div className={styles.content}>
        <h1 className={styles.title}>Privacy Policy</h1>
        <p className={styles.lastUpdated}>Last Updated: December 29, 2025</p>

        <div className={styles.section}>
          <h2>Introduction</h2>
          <p>
            This Privacy Policy explains how Limitless collects, uses, and protects your personal 
            information when you use our game. We are committed to protecting your privacy and 
            being transparent about our data practices.
          </p>
        </div>

        <div className={styles.section}>
          <h2>1. Information We Collect</h2>
          <p>When you use Limitless, we collect the following information:</p>
          <ul>
            <li><strong>Discord Account Information:</strong> Discord ID, Discord username</li>
            <li><strong>Account Information:</strong> Username, date of birth (for age verification)</li>
            <li><strong>Game Data:</strong> Characters, game progress, items, stats, and in-game actions</li>
            <li><strong>Technical Data:</strong> IP address, browser type, device information (for security and analytics)</li>
          </ul>
        </div>

        <div className={styles.section}>
          <h2>2. How We Use Your Information</h2>
          <p>We use your information to:</p>
          <ul>
            <li>Provide and operate the game</li>
            <li>Authenticate your account via Discord</li>
            <li>Save your game progress and characters</li>
            <li>Enforce our Terms of Service and prevent cheating</li>
            <li>Improve the game and fix bugs</li>
            <li>Communicate with you about game updates and issues</li>
          </ul>
        </div>

        <div className={styles.section}>
          <h2>3. Data Storage and Security</h2>
          <p>
            Your data is stored securely using AWS DynamoDB with industry-standard encryption and 
            security practices. We implement reasonable security measures to protect your information 
            from unauthorized access, alteration, or destruction.
          </p>
          <p>
            However, no method of transmission over the internet is 100% secure, and we cannot 
            guarantee absolute security.
          </p>
        </div>

        <div className={styles.section}>
          <h2>4. Data Sharing</h2>
          <p>
            We do NOT sell, trade, or share your personal information with third parties, except:
          </p>
          <ul>
            <li>With your explicit consent</li>
            <li>When required by law or legal process</li>
            <li>To protect our rights or the safety of others</li>
            <li>With service providers who help operate the game (e.g., AWS for hosting)</li>
          </ul>
        </div>

        <div className={styles.section}>
          <h2>5. Discord Integration</h2>
          <p>
            Limitless uses Discord OAuth for authentication. When you log in with Discord, we receive 
            your Discord ID and username from Discord. We do not have access to your Discord password 
            or private messages.
          </p>
          <p>
            Please review Discord's Privacy Policy to understand how Discord handles your data.
          </p>
        </div>

        <div className={styles.section}>
          <h2>6. Cookies and Tracking</h2>
          <p>
            We use JWT tokens stored in your browser's localStorage to maintain your login session. 
            We may use cookies for analytics and to improve user experience. You can disable cookies 
            in your browser settings, but this may affect game functionality.
          </p>
        </div>

        <div className={styles.section}>
          <h2>7. Children's Privacy</h2>
          <p>
            Limitless requires users to be at least 13 years old. We do not knowingly collect 
            personal information from children under 13. If we discover that a child under 13 has 
            provided us with personal information, we will delete it immediately.
          </p>
        </div>

        <div className={styles.section}>
          <h2>8. Your Rights</h2>
          <p>You have the right to:</p>
          <ul>
            <li>Access the personal information we have about you</li>
            <li>Request correction of inaccurate information</li>
            <li>Request deletion of your account and associated data</li>
            <li>Withdraw consent for data processing (which may require account deletion)</li>
          </ul>
          <p>
            To exercise these rights, contact the game administrators through the admin panel.
          </p>
        </div>

        <div className={styles.section}>
          <h2>9. Data Retention</h2>
          <p>
            We retain your account information and game data for as long as your account is active. 
            If you delete your account, we will delete your personal information within 30 days, 
            except where we are required to retain it for legal purposes.
          </p>
        </div>

        <div className={styles.section}>
          <h2>10. Changes to This Policy</h2>
          <p>
            We may update this Privacy Policy from time to time. Changes will be posted on this page 
            with an updated "Last Updated" date. Continued use of Limitless after changes constitutes 
            acceptance of the new policy.
          </p>
        </div>

        <div className={styles.section}>
          <h2>11. Contact</h2>
          <p>
            If you have questions or concerns about this Privacy Policy or how we handle your data, 
            please contact the game administrators.
          </p>
        </div>

        <div className={styles.footer}>
          <p>
            By using Limitless, you acknowledge that you have read and understood this Privacy Policy.
          </p>
        </div>
      </div>
    </div>
  );
};
