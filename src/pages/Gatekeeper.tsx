import React, { RefObject } from "react";
import { motion } from "framer-motion";
import { Lock, AlertCircle, ArrowRight } from "lucide-react";
import clsx from "clsx";
import styles from "../styles/App.module.css";
import { AuthMode, LockedApp } from "../types";

interface GatekeeperProps {
  blockedApp: LockedApp | null;
  authMode: AuthMode;
  gatekeeperPIN: string;
  error: string | null;
  isLaunching: boolean;
  gatekeeperInputRef: RefObject<HTMLInputElement | null>;
  setGatekeeperPIN: (val: string) => void;
  handleGatekeeperUnlock: (e: React.FormEvent) => void;
  closeWindow: () => void;
}

export const Gatekeeper = ({
  blockedApp,
  authMode,
  gatekeeperPIN,
  error,
  isLaunching,
  gatekeeperInputRef,
  setGatekeeperPIN,
  handleGatekeeperUnlock,
  closeWindow
}: GatekeeperProps) => {
  return (
    <motion.div
      key="gatekeeper"
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
      className={clsx(styles.gatekeeperCard, styles.solidBg)}
    >
      <div className={styles.gatekeeperBrand}>
        <div className={styles.appLogoContainer}>
          {blockedApp?.icon ? <img src={blockedApp.icon} alt={blockedApp.name} className={styles.appLogo} /> : <Lock size={32} />}
        </div>
        <h2 className={styles.gatekeeperTitle}>{blockedApp?.name}</h2>
        <p className={styles.gatekeeperSubtitle}>Secure Authentication Required</p>
      </div>
      <form onSubmit={handleGatekeeperUnlock} className={styles.gatekeeperForm} style={{ width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
        {error && <div className={styles.errorMessage} style={{ marginBottom: '2rem' }}><AlertCircle size={14} /> {error}</div>}
        {isLaunching ? <div className={styles.launchingState}><div className={styles.spinner} /><span>Launching...</span></div> : (
          <>
            {authMode === "PIN" ? (
              <div className={styles.pinDisplayGroup}>
                {[0, 1, 2, 3].map(i => <div key={i} className={clsx(styles.pinBox, gatekeeperPIN.length === i && styles.pinBoxActive, gatekeeperPIN.length > i && styles.pinBoxFilled)}>{gatekeeperPIN.length > i ? "●" : ""}</div>)}
              </div>
            ) : (
              <div style={{ width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1.5rem' }}>
                <input ref={gatekeeperInputRef} type="password" className={styles.modernInput} placeholder="Enter Password" value={gatekeeperPIN} onChange={(e) => setGatekeeperPIN(e.target.value)} />
                <motion.button
                  type="submit"
                  initial={{ opacity: 0, y: 5 }}
                  animate={{ opacity: gatekeeperPIN.length > 0 ? 1 : 0.5, y: 0, scale: gatekeeperPIN.length > 0 ? 1 : 0.98 }}
                  className={styles.unlockAction}
                  disabled={gatekeeperPIN.length === 0}
                >
                  <span>Unlock App</span>
                  <ArrowRight size={18} />
                </motion.button>
              </div>
            )}
            {authMode === "PIN" && (
              <input
                ref={gatekeeperInputRef}
                type="password"
                inputMode="numeric"
                pattern="\\d*"
                maxLength={4}
                className={styles.hiddenInput}
                autoComplete="one-time-code"
                name="gatekeeper-pin-hidden"
                value={gatekeeperPIN}
                onChange={(e) => {
                  const val = e.target.value.replace(/\\D/g, "").slice(0, 4);
                  setGatekeeperPIN(val);
                  if (val.length === 4) {
                    const form = (e.target as any).form;
                    if (form) setTimeout(() => form.dispatchEvent(new Event('submit', { cancelable: true, bubbles: true })), 100);
                  }
                }}
              />
            )}
          </>
        )}
      </form>
      <div className={styles.gatekeeperFooter}>
        <button type="button" onClick={closeWindow} className={styles.cancelBtn}>Cancel</button>
      </div>
    </motion.div>
  );
};
