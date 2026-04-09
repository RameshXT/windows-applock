import React, { RefObject } from "react";
import { motion } from "framer-motion";
import { Shield, AlertCircle, ArrowRight } from "lucide-react";
import clsx from "clsx";
import styles from "../styles/App.module.css";
import { AuthMode } from "../types";

interface UnlockProps {
  appName: string;
  authMode: AuthMode;
  password: string;
  error: string | null;
  mainInputRef: RefObject<HTMLInputElement | null>;
  setPassword: (val: string) => void;
  handleUnlock: (e: React.FormEvent) => void;
}

export const Unlock = ({
  appName,
  authMode,
  password,
  error,
  mainInputRef,
  setPassword,
  handleUnlock
}: UnlockProps) => {
  return (
    <motion.div
      key="unlock"
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
      className={styles.unlockScreen}
    >
      <div className={styles.unlockIcon}><Shield size={64} strokeWidth={1.5} /></div>
      <div className={styles.unlockTitle}>{appName} Access</div>
      <form onSubmit={handleUnlock} className={styles.unlockInputWrapper}>
        {error && <div className={styles.errorMessage} style={{ position: 'absolute', top: '-3rem' }}><AlertCircle size={14} /> {error}</div>}
        {authMode === "PIN" ? (
          <div className={styles.pinDisplayGroup}>
            {[0, 1, 2, 3].map(i => <div key={i} className={clsx(styles.pinBox, password.length === i && styles.pinBoxActive, password.length > i && styles.pinBoxFilled)}>{password.length > i ? "●" : ""}</div>)}
          </div>
        ) : (
          <div style={{ width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1.5rem' }}>
            <input ref={mainInputRef} type="password" className={styles.modernInput} placeholder="Enter Password" value={password} onChange={(e) => setPassword(e.target.value)} />
            <motion.button
              type="submit"
              initial={{ opacity: 0, y: 5 }}
              animate={{ opacity: password.length > 0 ? 1 : 0.5, y: 0, scale: password.length > 0 ? 1 : 0.98 }}
              className={styles.unlockAction}
              disabled={password.length === 0}
            >
              <span>Unlock</span>
              <ArrowRight size={18} />
            </motion.button>
          </div>
        )}
        {authMode === "PIN" && (
          <input
            ref={mainInputRef}
            type="password"
            inputMode="numeric"
            pattern="\\d*"
            maxLength={4}
            className={styles.hiddenInput}
            autoComplete="one-time-code"
            name="pin-unlock-hidden"
            value={password}
            onChange={(e) => {
              const val = e.target.value.replace(/\\D/g, "").slice(0, 4);
              setPassword(val);
              if (val.length === 4) {
                const form = (e.target as any).form;
                if (form) setTimeout(() => form.dispatchEvent(new Event('submit', { cancelable: true, bubbles: true })), 100);
              }
            }}
          />
        )}
      </form>
    </motion.div>
  );
};
