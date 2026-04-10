import React, { RefObject, useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Lock, AlertCircle, ArrowRight, Timer } from "lucide-react";
import clsx from "clsx";
import styles from "../styles/App.module.css";
import { AuthMode, LockedApp, AppConfig } from "../types";

interface GatekeeperProps {
  blockedApp: LockedApp | null;
  authMode: AuthMode;
  gatekeeperPIN: string;
  error: string | null;
  isLaunching: boolean;
  gatekeeperInputRef: RefObject<HTMLInputElement | null>;
  setGatekeeperPIN: (val: string) => void;
  setError?: (val: string | null) => void;
  config: AppConfig;
  handleGatekeeperUnlock: (e: React.FormEvent, override?: string) => void;
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
  setError,
  config,
  handleGatekeeperUnlock,
  closeWindow
}: GatekeeperProps) => {
  const [timeLeft, setTimeLeft] = useState(0);

  useEffect(() => {
    if (config.lockout_until) {
      const updateTimer = () => {
        const now = Math.floor(Date.now() / 1000);
        const diff = config.lockout_until! - now;
        if (diff > 0) {
          setTimeLeft(diff);
        } else {
          setTimeLeft(0);
        }
      };
      updateTimer();
      const interval = setInterval(updateTimer, 1000);
      return () => clearInterval(interval);
    } else {
      setTimeLeft(0);
    }
  }, [config.lockout_until]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        closeWindow();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [closeWindow]);

  const attempts = config.wrong_attempts || 0;
  const limit = config.attempt_limit || 5;

  return (
    <motion.div
      key="gatekeeper"
      initial={{ opacity: 0, scale: 0.98, y: 10 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.98, y: 10 }}
      className={clsx(styles.gatekeeperCard, styles.solidBg)}
    >
      <div className={styles.gatekeeperBrand}>
        <motion.div 
          initial={{ scale: 0.8, opacity: 0 }} 
          animate={{ scale: 1, opacity: 1 }} 
          className={styles.appLogoContainer}
        >
          {blockedApp?.icon ? <img src={blockedApp.icon} alt={blockedApp.name} className={styles.appLogo} /> : <Lock size={36} color="var(--accent-color)" />}
        </motion.div>
        <h2 className={styles.gatekeeperTitle}>{blockedApp?.name || "Access Restricted"}</h2>
        <p className={styles.gatekeeperSubtitle}>Please authenticate to continue</p>
      </div>

      {timeLeft > 0 ? (
        <div className={styles.lockoutContent}>
           <motion.div 
             initial={{ scale: 0.9, opacity: 0 }} 
             animate={{ scale: 1, opacity: 1 }} 
             className={styles.lockoutIcon}
           >
             <Timer size={40} color="#EF233C" />
           </motion.div>
           <h3 className={styles.lockoutTitle}>Security Lockout</h3>
           <p className={styles.lockoutText}>Cooldown active. Try again in:</p>
           <div className={styles.countdownTimer}>{timeLeft}s</div>
        </div>
      ) : (
        <form onSubmit={handleGatekeeperUnlock} className={styles.gatekeeperForm} style={{ width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
          {isLaunching ? (
            <div className={styles.launchingState}>
              <div className={styles.spinner} />
              <span>Verifying Access...</span>
            </div>
          ) : (
            <>
              <div style={{ width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '2.5rem' }}>
                {authMode === "PIN" ? (
                  <div className={styles.pinDisplayGroup}>
                    {[0, 1, 2, 3].map(i => (
                      <div 
                        key={i} 
                        className={clsx(
                          styles.pinBox, 
                          error && styles.pinBoxError,
                          !error && gatekeeperPIN.length === i && styles.pinBoxActive, 
                          gatekeeperPIN.length > i && styles.pinBoxFilled
                        )}
                      >
                        {gatekeeperPIN.length > i ? "●" : ""}
                      </div>
                    ))}
                  </div>
                ) : (
                  <div style={{ width: '100%', display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '1.5rem' }}>
                    <input 
                      ref={gatekeeperInputRef} 
                      type="password" 
                      className={clsx(styles.modernInput, error && styles.modernInputError)} 
                      placeholder="Enter Password" 
                      value={gatekeeperPIN} 
                      onChange={(e) => {
                        if (error) setError?.(null);
                        setGatekeeperPIN(e.target.value);
                      }} 
                    />
                    <motion.button
                      type="submit"
                      whileHover={{ scale: 1.02 }}
                      whileTap={{ scale: 0.98 }}
                      className={styles.unlockAction}
                      disabled={gatekeeperPIN.length === 0}
                    >
                      <span>Grant Access</span>
                      <ArrowRight size={18} />
                    </motion.button>
                  </div>
                )}
              </div>

              <div className={styles.attemptInfo}>
                 {attempts > 0 ? (
                   <span style={{ color: attempts >= limit - 1 ? '#EF233C' : 'inherit' }}>
                     {attempts} of {limit} attempts
                   </span>
                 ) : "Secure Session"}
              </div>

              <AnimatePresence>
                {error && (
                  <motion.div 
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: 'auto', opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    style={{ overflow: 'hidden', width: '100%', marginTop: '1.5rem' }}
                  >
                    <div className={styles.errorMessage} style={{ width: '100%' }}>
                      <AlertCircle size={14} /> 
                      {error}
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>

              {authMode === "PIN" && (
                <input
                  ref={gatekeeperInputRef}
                  type="password"
                  inputMode="numeric"
                  pattern="\d*"
                  maxLength={4}
                  className={styles.hiddenInput}
                  autoComplete="one-time-code"
                  name="gatekeeper-pin-hidden"
                  value={gatekeeperPIN}
                  onChange={(e) => {
                    const val = e.target.value.replace(/\D/g, "").slice(0, 4);
                    if (error && val.length < gatekeeperPIN.length) setError?.(null);
                    setGatekeeperPIN(val);
                    if (val.length === 4) {
                      handleGatekeeperUnlock({ preventDefault: () => {} } as React.FormEvent, val);
                    }
                  }}
                  onKeyDown={(e) => {
                    if (!/[0-9]/.test(e.key) && e.key !== 'Backspace' && e.key !== 'Tab' && e.key !== 'Delete' && e.key !== 'ArrowLeft' && e.key !== 'ArrowRight' && !e.ctrlKey && !e.metaKey) {
                      e.preventDefault();
                    }
                  }}
                />
              )}
            </>
          )}
        </form>
      )}

      <div className={styles.gatekeeperFooter}>
        <div className={styles.keyHint}>
           <kbd>ESC</kbd> <span>to dismiss</span>
        </div>
        <button type="button" onClick={closeWindow} className={styles.cancelBtn}>Cancel & Block</button>
      </div>
    </motion.div>
  );
};
