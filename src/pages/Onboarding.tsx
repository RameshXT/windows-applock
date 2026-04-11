import { motion } from "framer-motion";
import { Lock, Search, ShieldCheck, Shield, ArrowRight } from "lucide-react";
import styles from "../styles/App.module.css";
import logo from "../assets/logo.png";

interface OnboardingProps {
  appName: string;
  onContinue: () => void;
}

export const Onboarding = ({ appName, onContinue }: OnboardingProps) => {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className={styles.onboarding}
    >
      <div className={styles.heroBackground} />

      <motion.div
        initial={{ y: -20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ duration: 0.6, ease: "easeOut" }}
        className={styles.onboardingHeader}
      >
        <motion.div
          animate={{ y: [0, -10, 0] }}
          transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
          className={styles.heroIconWrapper}
        >
          <div className={styles.heroIconGlow} />
            <img src={logo} style={{ width: 120, height: 120, objectFit: 'contain' }} className={styles.unlockIcon} alt={appName} />
        </motion.div>
        <h1 className={styles.onboardingTitle}>{appName}</h1>
        <p className={styles.onboardingSubtitle}>Precision Privacy for Windows</p>
      </motion.div>

      <motion.div
        className={styles.featureGrid}
        initial="hidden"
        animate="visible"
        variants={{
          visible: { transition: { staggerChildren: 0.1 } }
        }}
      >
        {[
          { icon: <Lock size={24} />, title: "Secure Access", desc: "Military-grade encryption for your master credentials." },
          { icon: <Search size={24} />, title: "Smart Mapping", desc: "Instantly discover and protect any application." },
          { icon: <ShieldCheck size={24} />, title: "Active Shield", desc: "Real-time background protection that never sleeps." }
        ].map((f, i) => (
          <motion.div
            key={i}
            variants={{
              hidden: { y: 20, opacity: 0 },
              visible: { y: 0, opacity: 1 }
            }}
            className={styles.featureCard}
          >
            <div className={styles.featureIcon}>{f.icon}</div>
            <h3 className={styles.featureTitle}>{f.title}</h3>
            <p className={styles.featureDesc}>{f.desc}</p>
          </motion.div>
        ))}
      </motion.div>

      <motion.div
        initial={{ y: 20, opacity: 0 }}
        animate={{ y: 0, opacity: 1 }}
        transition={{ delay: 0.5, duration: 0.5 }}
        className={styles.onboardingActions}
      >
        <button
          onClick={onContinue}
          className={styles.primaryBtn}
        >
          <span>Initialize Security</span>
          <ArrowRight size={20} />
        </button>
        <span className={styles.versionBadge}>VERSION 2.1.0 • SECURED BY QUANTUM</span>
      </motion.div>
    </motion.div>
  );
};
