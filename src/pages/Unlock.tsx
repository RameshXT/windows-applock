import React, { RefObject } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Shield, AlertCircle, ArrowRight } from "lucide-react";
import clsx from "clsx";
import styles from "../styles/App.module.css";
import logo from "../assets/logo.png";
import { AuthMode } from "../types";

interface UnlockProps {
  appName: string;
  authMode: AuthMode;
  password: string;
  error: string | null;
  isVerify?: boolean;
  mainInputRef: RefObject<HTMLInputElement | null>;
  setPassword: (val: string) => void;
  setError?: (val: string | null) => void;
  handleUnlock: (e: React.FormEvent, override?: string) => void;
  onCancel?: () => void;
}

export const Unlock = ({
  appName,
  authMode,
  password,
  error,
  isVerify,
  mainInputRef,
  setPassword,
  setError,
  handleUnlock,
  onCancel
}: UnlockProps) => {
  return (
    <motion.div
      key={isVerify ? "verify" : "unlock"}
      initial={{ opacity: 0, scale: 0.98 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.98 }}
      transition={{ duration: 0.3 }}
      className={styles.unlockScreen}
    >
      <div className={styles.unlockIcon}><img src={logo} style={{ width: 100, height: 100, objectFit: 'contain' }} alt={appName} /></div>
      <div className={styles.unlockTitle}>{isVerify ? "Identity Verification" : `${appName} Access`}</div>
      <p style={{ opacity: 0.5, fontSize: '0.8rem', marginTop: '-0.5rem', marginBottom: '1.5rem' }}>
        {isVerify ? "Please confirm your current security credentials" : "System perimeter is currently secured"}
      </p>
      
      <form onSubmit={handleUnlock} className={styles.unlockInputWrapper} style={{ gap: '2.5rem' }}>
        <div style={{ width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '2rem' }}>
          {authMode === "PIN" ? (
            <div className={styles.pinDisplayGroup}>
              {[0, 1, 2, 3].map(i => (
                <div 
                  key={i} 
                  className={clsx(
                    styles.pinBox, 
                    error && styles.pinBoxError,
                    !error && password.length === i && styles.pinBoxActive, 
                    password.length > i && styles.pinBoxFilled
                  )}
                >
                  {password.length > i ? "●" : ""}
                </div>
              ))}
            </div>
          ) : (
            <input 
              ref={mainInputRef} 
              type="password" 
              className={clsx(styles.modernInput, error && styles.modernInputError)} 
              placeholder="Enter Password" 
              value={password} 
              onChange={(e) => {
                if (error) setError?.(null);
                setPassword(e.target.value);
              }} 
            />
          )}

          {(isVerify || authMode !== "PIN") && (
            <div style={{ 
              display: (isVerify && authMode !== "PIN") ? 'grid' : 'flex', 
              gridTemplateColumns: (isVerify && authMode !== "PIN") ? '1fr 1fr' : 'none',
              gap: '1rem', 
              width: '100%' 
            }}>
              {isVerify && (
                <button type="button" onClick={onCancel} className={styles.modalCancel} style={{ height: '52px', width: '100%', fontWeight: 600, borderRadius: '12px' }}>
                  Cancel
                </button>
              )}
              {authMode !== "PIN" && (
                <motion.button
                  type="submit"
                  className={styles.unlockAction}
                  disabled={password.length === 0}
                  style={{ 
                    height: '52px', 
                    flex: isVerify ? 'unset' : 2, 
                    width: '100%',
                    display: 'flex', 
                    justifyContent: 'center',
                    borderRadius: '12px',
                    padding: '0' // Remove padding to prevent internal expansion
                  }}
                >
                  <span>{isVerify ? "Verify" : "Unlock"}</span>
                  {!isVerify && <ArrowRight size={18} />}
                </motion.button>
              )}
            </div>
          )}
        </div>

        <AnimatePresence>
          {error && (
            <motion.div 
              initial={{ height: 0, opacity: 0, margin: 0 }}
              animate={{ height: 'auto', opacity: 1, margin: '1.5rem 0' }}
              exit={{ height: 0, opacity: 0, margin: 0 }}
              style={{ overflow: 'hidden', width: '100%' }}
            >
              <div className={styles.errorMessage}><AlertCircle size={14} /> {error}</div>
            </motion.div>
          )}
        </AnimatePresence>

        {authMode === "PIN" && (
          <input
            ref={mainInputRef}
            type="password"
            inputMode="numeric"
            pattern="\d*"
            maxLength={4}
            className={styles.hiddenInput}
            autoComplete="one-time-code"
            name="pin-unlock-hidden"
            value={password}
            onChange={(e) => {
              const val = e.target.value.replace(/\D/g, "").slice(0, 4);
              if (error && val.length < password.length) setError?.(null);
              setPassword(val);
              if (val.length === 4) {
                handleUnlock({ preventDefault: () => {} } as React.FormEvent, val);
              }
            }}
            onKeyDown={(e) => {
              if (!/[0-9]/.test(e.key) && e.key !== 'Backspace' && e.key !== 'Tab' && e.key !== 'Delete' && e.key !== 'ArrowLeft' && e.key !== 'ArrowRight' && !e.ctrlKey && !e.metaKey) {
                e.preventDefault();
              }
            }}
          />
        )}
      </form>
    </motion.div>
  );
};
