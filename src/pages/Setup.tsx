import React, { RefObject } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { AlertCircle, ArrowRight } from "lucide-react";
import clsx from "clsx";
import styles from "../styles/App.module.css";
import logo from "../assets/logo.png";
import { AuthMode } from "../types";

interface SetupProps {
  authMode: AuthMode;
  password: String;
  confirmPassword: String;
  error: string | null;
  isCompleting: boolean;
  completingStep: number;
  allAppsCount: number;
  pinInputRef: RefObject<HTMLInputElement | null>;
  confirmInputRef: RefObject<HTMLInputElement | null>;
  setAuthMode: (mode: AuthMode) => void;
  setPassword: (val: string) => void;
  setConfirmPassword: (val: string) => void;
  setError: (val: string | null) => void;
  handleSetup: (e: React.FormEvent) => void;
  setView: (view: any) => void;
}

export const Setup = ({
  authMode,
  password,
  confirmPassword,
  error,
  isCompleting,
  completingStep,
  allAppsCount,
  pinInputRef,
  confirmInputRef,
  setAuthMode,
  setPassword,
  setConfirmPassword,
  setError,
  handleSetup,
  setView,
}: SetupProps) => {
  if (isCompleting) {
    return (
      <motion.div
        key="completing"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className={styles.unlockScreen}
      >
        <div
          className={styles.premiumLoader}
          style={{ width: "80px", height: "80px", marginBottom: "2rem" }}
        >
          <motion.div
            className={styles.loaderRing}
            style={{ borderWidth: "3px" }}
            animate={{ rotate: 360 }}
            transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
          />
          <img
            src={logo}
            style={{ width: 64, height: 64, objectFit: "contain" }}
            className={styles.loaderIcon}
            alt=""
          />
        </div>

        <AnimatePresence mode="wait">
          <motion.div
            key={completingStep}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.4 }}
            style={{ textAlign: "center" }}
          >
            <h2
              className={styles.statusTitle}
              style={{ fontSize: "1.5rem", marginBottom: "0.5rem" }}
            >
              {
                [
                  "Great! You're all set.",
                  "We're preparing your perimeter...",
                  "One moment...",
                  "Here we go!",
                ][completingStep]
              }
            </h2>
            <p className={styles.statusSubtitle}>
              Initializing secure environment
            </p>
          </motion.div>
        </AnimatePresence>
      </motion.div>
    );
  }

  return (
    <motion.div
      key="setup"
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
      className={styles.unlockScreen}
      style={{ maxWidth: "440px" }}
    >
      <div className={styles.gatekeeperBrand} style={{ marginBottom: "1rem" }}>
        <div
          className={styles.statusCircle}
          style={{ width: "64px", height: "64px", marginBottom: "1.5rem" }}
        >
          <img
            src={logo}
            style={{ width: 48, height: 48, objectFit: "contain" }}
            alt=""
          />
        </div>
        <h1 className={styles.statusTitle} style={{ fontSize: "1.75rem" }}>
          Security Protocol
        </h1>
        <p className={styles.statusSubtitle}>
          Configure your master authentication method
        </p>
      </div>

      <div className={styles.tabs} style={{ marginBottom: "0.5rem" }}>
        <button
          className={clsx(styles.tab, authMode === "PIN" && styles.tabActive)}
          onClick={() => {
            setAuthMode("PIN");
            setPassword("");
            setConfirmPassword("");
            setError(null);
          }}
        >
          PIN
        </button>
        <button
          className={clsx(
            styles.tab,
            authMode === "Password" && styles.tabActive
          )}
          onClick={() => {
            setAuthMode("Password");
            setPassword("");
            setConfirmPassword("");
            setError(null);
          }}
        >
          Password
        </button>
      </div>

      <form
        onSubmit={handleSetup}
        className={styles.unlockInputWrapper}
        style={{ gap: "2rem", width: "100%" }}
      >
        {authMode === "PIN" ? (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "2rem",
              alignItems: "center",
              width: "100%",
            }}
          >
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "0.75rem",
                alignItems: "center",
                width: "100%",
                opacity: (password as string).length < 4 ? 1 : 0.4,
                transition: "all 0.3s ease",
              }}
            >
              <span
                style={{
                  fontSize: "0.7rem",
                  fontWeight: 700,
                  color: "var(--text-secondary)",
                  textTransform: "uppercase",
                  letterSpacing: "0.1em",
                  opacity: 0.5,
                }}
              >
                New Secret PIN
              </span>
              <div className={styles.pinDisplayGroup}>
                {[0, 1, 2, 3].map((i) => (
                  <div
                    key={i}
                    className={clsx(
                      styles.pinBox,
                      error &&
                      (password as string).length < 4 &&
                      styles.pinBoxError,
                      !error &&
                      (password as string).length < 4 &&
                      (password as string).length === i &&
                      styles.pinBoxActive,
                      (password as string).length > i && styles.pinBoxFilled
                    )}
                  >
                    {(password as string).length > i ? "●" : ""}
                  </div>
                ))}
              </div>
            </div>

            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "0.75rem",
                alignItems: "center",
                width: "100%",
                opacity: (password as string).length === 4 ? 1 : 0.15,
                transition: "all 0.3s ease",
              }}
            >
              <span
                style={{
                  fontSize: "0.7rem",
                  fontWeight: 700,
                  color: "var(--text-secondary)",
                  textTransform: "uppercase",
                  letterSpacing: "0.1em",
                  opacity: 0.5,
                }}
              >
                Confirm Secret PIN
              </span>
              <div className={styles.pinDisplayGroup}>
                {[0, 1, 2, 3].map((i) => (
                  <div
                    key={i}
                    className={clsx(
                      styles.pinBox,
                      error &&
                      (password as string).length === 4 &&
                      styles.pinBoxError,
                      !error &&
                      (password as string).length === 4 &&
                      (confirmPassword as string).length === i &&
                      styles.pinBoxActive,
                      (confirmPassword as string).length > i &&
                      styles.pinBoxFilled
                    )}
                  >
                    {(confirmPassword as string).length > i ? "●" : ""}
                  </div>
                ))}
              </div>
            </div>

            <input
              ref={pinInputRef}
              type="password"
              inputMode="numeric"
              pattern="\d*"
              maxLength={4}
              className={styles.hiddenInput}
              autoComplete="one-time-code"
              name="new-pin-hidden"
              value={password as string}
              onChange={(e) => {
                if (error) setError(null);
                setPassword(e.target.value.replace(/\D/g, "").slice(0, 4));
              }}
              onKeyDown={(e) => {
                if (
                  !/[0-9]/.test(e.key) &&
                  e.key !== "Backspace" &&
                  e.key !== "Tab" &&
                  e.key !== "Delete" &&
                  e.key !== "ArrowLeft" &&
                  e.key !== "ArrowRight" &&
                  !e.ctrlKey &&
                  !e.metaKey
                ) {
                  e.preventDefault();
                }
              }}
            />
            <input
              ref={confirmInputRef}
              type="password"
              inputMode="numeric"
              pattern="\d*"
              maxLength={4}
              className={styles.hiddenInput}
              autoComplete="one-time-code"
              name="confirm-pin-hidden"
              value={confirmPassword as string}
              onChange={(e) => {
                if (error) setError(null);
                setConfirmPassword(
                  e.target.value.replace(/\D/g, "").slice(0, 4)
                );
              }}
              onKeyDown={(e) => {
                if (e.key === "Backspace" && confirmPassword.length === 0) {
                  setPassword((password as string).slice(0, -1));
                }
                if (
                  !/[0-9]/.test(e.key) &&
                  e.key !== "Backspace" &&
                  e.key !== "Tab" &&
                  e.key !== "Delete" &&
                  e.key !== "ArrowLeft" &&
                  e.key !== "ArrowRight" &&
                  !e.ctrlKey &&
                  !e.metaKey
                ) {
                  e.preventDefault();
                }
              }}
            />
          </div>
        ) : (
          <div
            style={{
              width: "100%",
              display: "flex",
              flexDirection: "column",
              gap: "1rem",
            }}
          >
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "0.5rem",
              }}
            >
              <span
                style={{
                  fontSize: "0.7rem",
                  fontWeight: 700,
                  color: "var(--text-secondary)",
                  textTransform: "uppercase",
                  letterSpacing: "0.1em",
                  opacity: 0.5,
                  marginLeft: "0.5rem",
                }}
              >
                Master Password
              </span>
              <input
                ref={pinInputRef}
                type="password"
                className={clsx(
                  styles.modernInput,
                  error && styles.modernInputError
                )}
                placeholder="••••••••"
                value={password as string}
                onChange={(e) => {
                  if (error) setError(null);
                  setPassword(e.target.value);
                }}
              />
            </div>
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "0.5rem",
              }}
            >
              <span
                style={{
                  fontSize: "0.7rem",
                  fontWeight: 700,
                  color: "var(--text-secondary)",
                  textTransform: "uppercase",
                  letterSpacing: "0.1em",
                  opacity: 0.5,
                  marginLeft: "0.5rem",
                }}
              >
                Confirm Password
              </span>
              <input
                ref={confirmInputRef}
                type="password"
                className={clsx(
                  styles.modernInput,
                  error && styles.modernInputError
                )}
                placeholder="••••••••"
                value={confirmPassword as string}
                onChange={(e) => {
                  if (error) setError(null);
                  setConfirmPassword(e.target.value);
                }}
              />
            </div>
          </div>
        )}

        <AnimatePresence>
          {error && (
            <motion.div
              initial={{ height: 0, opacity: 0, margin: 0 }}
              animate={{ height: "auto", opacity: 1, margin: "1rem 0" }}
              exit={{ height: 0, opacity: 0, margin: 0 }}
              style={{ overflow: "hidden", width: "100%" }}
            >
              <div className={styles.errorMessage}>
                <AlertCircle size={14} /> {error}
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        <div
          style={{
            display: "flex",
            gap: "1rem",
            width: "100%",
            marginTop: "1rem",
          }}
        >
          {allAppsCount > 0 && (
            <button
              type="button"
              className={styles.modalCancel}
              style={{ flex: 1, height: "56px" }}
              onClick={() => setView("dashboard")}
            >
              Cancel
            </button>
          )}
          <button
            type="submit"
            className={styles.unlockAction}
            style={{ flex: 2, height: "56px", justifyContent: "center" }}
          >
            <span>{allAppsCount > 0 ? "Update" : `Continue Registration`}</span>
            <ArrowRight size={18} />
          </button>
        </div>
      </form>
    </motion.div>
  );
};
