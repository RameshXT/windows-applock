import { useState, useEffect, useRef } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { AlertCircle, AlertTriangle } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import styles from "./styles/App.module.css";
import clsx from "clsx";

// Types
import { View, AuthMode, Tab, InstalledApp, LockedApp, AppConfig } from "./types";

// Components
import { Onboarding } from "./pages/Onboarding";
import { Setup } from "./pages/Setup";
import { Unlock } from "./pages/Unlock";
import { Dashboard } from "./pages/Dashboard";
import { Gatekeeper } from "./pages/Gatekeeper";

const APP_NAME = "Windows AppLock";

function App() {
  const [view, setView] = useState<View | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("home");
  const [authMode, setAuthMode] = useState<AuthMode>("PIN");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [allApps, setAllApps] = useState<InstalledApp[]>([]);
  const [lockedApps, setLockedApps] = useState<LockedApp[]>([]);
  const [search, setSearch] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [blockedApp, setBlockedApp] = useState<LockedApp | null>(null);
  const [gatekeeperPIN, setGatekeeperPIN] = useState("");
  const [isLaunching, setIsLaunching] = useState(false);
  const [appToRemove, setAppToRemove] = useState<LockedApp | InstalledApp | null>(null);
  const [showResetConfirm, setShowResetConfirm] = useState(false);
  const [showResetFinal, setShowResetFinal] = useState(false);
  const [isCompleting, setIsCompleting] = useState(false);
  const [completingStep, setCompletingStep] = useState(0);
  const [isUpdatingFromSettings, setIsUpdatingFromSettings] = useState(false);
  const [showUpdateSuccess, setShowUpdateSuccess] = useState(false);
  
  const pinInputRef = useRef<HTMLInputElement>(null);
  const confirmInputRef = useRef<HTMLInputElement>(null);
  const mainInputRef = useRef<HTMLInputElement>(null);
  const gatekeeperInputRef = useRef<HTMLInputElement>(null);

  const [settingsTab, setSettingsTab] = useState("account");
  const [config, setConfig] = useState<AppConfig>({
    locked_apps: [],
    auth_mode: "PIN",
    attempt_limit: 5,
    lockout_duration: 60,
    theme: "dark"
  });

  useEffect(() => {
    const init = async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const windowLabel = getCurrentWindow().label;

        if (windowLabel === "gatekeeper") {
          const blocked = await invoke<LockedApp | null>("get_blocked_app");
          if (blocked) {
            setBlockedApp(blocked);
            setView("gatekeeper");
          } else {
            getCurrentWindow().close();
          }
          return;
        }

        const cfg = await invoke<AppConfig>("get_config");
        setConfig(cfg);
        if (cfg.auth_mode) setAuthMode(cfg.auth_mode);
        if (cfg.theme) document.documentElement.setAttribute("data-theme", cfg.theme);

        const isSetup = await invoke<boolean>("check_setup");
        if (!isSetup) {
          setView("onboarding");
        } else {
          const isUnlocked = await invoke<boolean>("get_is_unlocked");
          if (isUnlocked) {
            setView("dashboard");
          } else {
            setView("unlock");
          }
        }

        const locked = await invoke<LockedApp[]>("get_apps");
        setLockedApps(locked);

        setIsScanning(true);
        const apps = await invoke<InstalledApp[]>("get_system_apps");
        setAllApps(apps);
        setIsScanning(false);
      } catch (err) {
        console.error(err);
      }
    };
    init();

    const unlisten = listen<LockedApp>("app-blocked", async (event) => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      if (getCurrentWindow().label === "gatekeeper") {
        setBlockedApp(event.payload);
        setView("gatekeeper");
        setGatekeeperPIN("");
      }
    });

    const unlistenReload = listen("reload-app", () => {
      window.location.reload();
    });

    return () => {
      unlisten.then(f => f());
      unlistenReload.then(f => f());
    };
  }, []);

  const handleSetup = async (e: React.FormEvent) => {
    e.preventDefault();
    if (authMode === "PIN" && !/^\d{4}$/.test(password)) {
      setError("PIN must be exactly 4 digits");
      return;
    }
    if (password !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }
    try {
      await invoke("setup_password", { password, mode: authMode });
      setError(null);
      
      if (isUpdatingFromSettings) {
        setShowUpdateSuccess(true);
        setView("dashboard");
        setIsUpdatingFromSettings(false);
        setTimeout(() => setShowUpdateSuccess(false), 3000);
      } else {
        setIsCompleting(true);
        const messages = ["Great! You're all set.", "We're preparing your perimeter..", "One moment..", "Here we go!"];
        for (let i = 0; i < messages.length; i++) {
          setCompletingStep(i);
          await new Promise(r => setTimeout(r, 1400));
        }
        setView("dashboard");
        setIsCompleting(false);
      }
    } catch (err) { setError(String(err)); }
  };

  const handleUnlock = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const isValid = await invoke<boolean>("verify_password", { password });
      if (isValid) {
        setView("dashboard");
        setError(null);
        setPassword("");
      } else {
        setError("Invalid credentials");
      }
    } catch (err) { setError(String(err)); }
  };

  const handleGatekeeperUnlock = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!blockedApp) return;
    try {
      const isValid = await invoke<boolean>("verify_password", { password: gatekeeperPIN });
      if (isValid) {
        setIsLaunching(true);
        await invoke("release_app", { appPath: blockedApp.exec_name });
        setTimeout(async () => {
          setBlockedApp(null);
          setGatekeeperPIN("");
          setIsLaunching(false);
          const { getCurrentWindow } = await import("@tauri-apps/api/window");
          getCurrentWindow().close();
        }, 1000);
      } else {
        setGatekeeperPIN("");
        setError("Invalid credentials");
      }
    } catch (err) { setError(String(err)); }
  };

  const toggleApp = async (app: LockedApp | InstalledApp) => {
    const isLocked = lockedApps.some(la => la.name === app.name);
    if (isLocked) {
      setAppToRemove(app);
      return;
    }
    const newLocked: LockedApp[] = [...lockedApps, {
      id: Math.random().toString(36).substring(2, 9),
      name: app.name,
      exec_name: ((app as any).path || (app as any).exec_name) || "",
      icon: app.icon
    }];
    setLockedApps(newLocked);
    try { await invoke("save_selection", { apps: newLocked }); } catch (err) { setError(String(err)); }
  };

  const confirmRemoval = async () => {
    if (!appToRemove) return;
    const newLocked = lockedApps.filter(la => la.name !== appToRemove.name);
    setLockedApps(newLocked);
    setAppToRemove(null);
    try { await invoke("save_selection", { apps: newLocked }); } catch (err) { setError(String(err)); }
  };

  const handleLockSession = async () => {
    await invoke("lock_session");
    setView("unlock");
  };

  const updateConfig = async (updates: Partial<AppConfig>) => {
    const newConfig = { ...config, ...updates };
    setConfig(newConfig);
    if (updates.auth_mode) setAuthMode(updates.auth_mode);
    if (updates.theme) document.documentElement.setAttribute("data-theme", updates.theme);
    try { await invoke("update_settings", { newConfig }); } catch (err) { setError(String(err)); }
  };

  const sensitiveApps = ["WhatsApp", "Slack", "Teams", "Telegram", "Instagram", "VS Code"];
  const [placeholder, setPlaceholder] = useState("");
  const [appIndex, setAppIndex] = useState(0);
  const [isDeleting, setIsDeleting] = useState(false);
  const [charIndex, setCharIndex] = useState(0);

  useEffect(() => {
    const typingSpeed = isDeleting ? 40 : 100;
    const currentApp = sensitiveApps[appIndex];
    const timeout = setTimeout(() => {
      if (!isDeleting && charIndex < currentApp.length) {
        setPlaceholder(currentApp.substring(0, charIndex + 1));
        setCharIndex(charIndex + 1);
      } else if (isDeleting && charIndex > 0) {
        setPlaceholder(currentApp.substring(0, charIndex - 1));
        setCharIndex(charIndex - 1);
      } else if (!isDeleting && charIndex === currentApp.length) {
        setTimeout(() => setIsDeleting(true), 2000);
      } else if (isDeleting && charIndex === 0) {
        setIsDeleting(false);
        setAppIndex((appIndex + 1) % sensitiveApps.length);
      }
    }, typingSpeed);
    return () => clearTimeout(timeout);
  }, [charIndex, isDeleting, appIndex]);

  useEffect(() => {
    const handleFocus = () => {
      let target: HTMLInputElement | null = null;
      if (view === "unlock") target = mainInputRef.current;
      else if (view === "gatekeeper") target = gatekeeperInputRef.current;
      else if (view === "setup") {
        if (authMode === "PIN") target = password.length < 4 ? pinInputRef.current : confirmInputRef.current;
        else if (document.activeElement !== pinInputRef.current && document.activeElement !== confirmInputRef.current) target = pinInputRef.current;
      }
      if (target && document.activeElement !== target) target.focus();
    };
    handleFocus();
    const interval = setInterval(handleFocus, 100);
    window.addEventListener("focus", handleFocus);
    window.addEventListener("click", handleFocus);
    return () => {
      clearInterval(interval);
      window.removeEventListener("focus", handleFocus);
      window.removeEventListener("click", handleFocus);
    };
  }, [view, authMode, password.length]);

  if (view === null) return null;

  return (
    <div className={clsx(styles.container, view === 'gatekeeper' && styles.transparentBg)}>
      <AnimatePresence mode="wait">
        {view === "onboarding" && <Onboarding appName={APP_NAME} onContinue={() => setView('setup')} />}
        {view === "setup" && (
          <Setup 
            authMode={authMode} password={password} confirmPassword={confirmPassword} error={error}
            isCompleting={isCompleting} completingStep={completingStep} allAppsCount={allApps.length}
            pinInputRef={pinInputRef} confirmInputRef={confirmInputRef} setAuthMode={setAuthMode}
            setPassword={setPassword} setConfirmPassword={setConfirmPassword} setError={setError}
            handleSetup={handleSetup} setView={setView}
          />
        )}
        {view === "unlock" && (
          <Unlock 
            appName={APP_NAME} authMode={authMode} password={password} error={error}
            mainInputRef={mainInputRef} setPassword={setPassword} handleUnlock={handleUnlock}
          />
        )}
        {view === "dashboard" && (
          <Dashboard 
            appName={APP_NAME} activeTab={activeTab} setActiveTab={setActiveTab} showUpdateSuccess={showUpdateSuccess}
            search={search} setSearch={setSearch} placeholder={placeholder} handleLockSession={handleLockSession}
            isScanning={isScanning} allApps={allApps} lockedApps={lockedApps} toggleApp={toggleApp}
            settingsTab={settingsTab} setSettingsTab={setSettingsTab} authMode={authMode} setAuthMode={setAuthMode}
            setView={setView} setIsUpdatingFromSettings={setIsUpdatingFromSettings} config={config}
            updateConfig={updateConfig} setShowResetConfirm={setShowResetConfirm}
          />
        )}
        {view === "gatekeeper" && (
          <Gatekeeper 
            blockedApp={blockedApp} authMode={authMode} gatekeeperPIN={gatekeeperPIN} error={error}
            isLaunching={isLaunching} gatekeeperInputRef={gatekeeperInputRef} setGatekeeperPIN={setGatekeeperPIN}
            handleGatekeeperUnlock={handleGatekeeperUnlock} closeWindow={async () => { const { getCurrentWindow } = await import("@tauri-apps/api/window"); getCurrentWindow().close(); }}
          />
        )}
      </AnimatePresence>

      <AnimatePresence>
        {appToRemove && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className={styles.modalOverlay}>
            <motion.div initial={{ scale: 0.95 }} animate={{ scale: 1 }} className={styles.modalCard}>
              <div className={styles.modalIcon}><AlertCircle size={40} color="var(--error-color)" /></div>
              <h3>Remove Protection?</h3>
              <p>Are you sure you want to unlock <strong>{appToRemove.name}</strong>? This application will no longer be protected by {APP_NAME}.</p>
              <div className={styles.modalActions}>
                <button className={styles.modalCancel} onClick={() => setAppToRemove(null)}>Cancel</button>
                <button className={styles.modalConfirm} onClick={confirmRemoval}>Remove Lock</button>
              </div>
            </motion.div>
          </motion.div>
        )}
        {showResetConfirm && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className={styles.modalOverlay}>
            <motion.div initial={{ scale: 0.95 }} animate={{ scale: 1 }} className={styles.modalCard}>
              <div className={styles.modalIcon}><AlertTriangle size={40} color="#f59e0b" /></div>
              <h3>Wipe All Data?</h3>
              <p>Are you sure you want to reset {APP_NAME}? This will remove all your protected apps and security settings.</p>
              <div className={styles.modalActions}>
                <button className={styles.modalCancel} onClick={() => setShowResetConfirm(false)}>Cancel</button>
                <button className={styles.modalConfirm} style={{ background: '#f59e0b' }} onClick={() => { setShowResetConfirm(false); setShowResetFinal(true); }}>Yes, Continue</button>
              </div>
            </motion.div>
          </motion.div>
        )}
        {showResetFinal && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className={styles.modalOverlay}>
            <motion.div initial={{ scale: 0.95 }} animate={{ scale: 1 }} className={styles.modalCard}>
              <div className={styles.modalIcon}><AlertCircle size={40} color="var(--error-color)" /></div>
              <h3>Final Warning</h3>
              <p>This action is <strong>irreversible</strong>. All your configurations will be permanently deleted and cannot be recovered.</p>
              <div className={styles.modalActions}>
                <button className={styles.modalCancel} onClick={() => setShowResetFinal(false)}>Abort</button>
                <button className={styles.modalConfirm} onClick={async () => { await invoke("reset_app"); window.location.reload(); }}>Reset Everything</button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
