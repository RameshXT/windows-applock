import React from "react";
import styles from "../../styles/App.module.css";
import { Heart } from "lucide-react";

interface CreditsProps {
  appName: string;
}

export const Credits: React.FC<CreditsProps> = ({ appName }) => {
  return (
    <section className={styles.settingsGroup}>
      <div className={styles.settingsHeader}>
        <h2>Credits</h2>
        <p>Recognizing the people behind {appName}.</p>
      </div>

      <div className={styles.creditsGrid}>
        <a
          href="https://rameshxt.pages.dev/"
          target="_blank"
          rel="noopener noreferrer"
          className={styles.creditsCardMain}
        >
          <div className={styles.creditsIcon}>
            <Heart size={24} color="#fff" fill="#fff" />
          </div>
          <div className={styles.creditsContent}>
            <span className={styles.creditsTitle}>Designed & Developed by</span>
            <span className={styles.creditsDeveloperName}>Ramesh XT</span>
            <span className={styles.creditsDesc}>
              A DevOps Engineer & Freelancer
            </span>
          </div>
        </a>
      </div>
    </section>
  );
};
