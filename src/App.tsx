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
  const [activeTab, setActiveTab] = useState<Tab>(() => (localStorage.getItem("applock_tab") as Tab) || "home");
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
  const [isLaunching] = useState(false);
  const [showResetConfirm, setShowResetConfirm] = useState(false);
  const [showResetFinal, setShowResetFinal] = useState(false);
  const [isCompleting, setIsCompleting] = useState(false);
  const [completingStep, setCompletingStep] = useState(0);
  const [isUpdatingFromSettings, setIsUpdatingFromSettings] = useState(false);
  const [showUpdateSuccess, setShowUpdateSuccess] = useState(false);
  const [toast, setToast] = useState<{ message: string; visible: boolean; type: 'lock' | 'unlock' | 'success' }>({ message: "", visible: false, type: 'success' });
  const [appToRemove, setAppToRemove] = useState<LockedApp | InstalledApp | null>(null);
  const [appsToBulkUnlock, setAppsToBulkUnlock] = useState<LockedApp[] | null>(null);
  
  // Function to fetch fresh app list from backend
  const fetchDetailedApps = async () => {
    try {
      setIsScanning(true);
      // Use get_detailed_apps for fresh registry and Store data
      const apps = await invoke<InstalledApp[]>("get_detailed_apps");
      setAllApps(apps);
      setIsScanning(false);
    } catch (err) {
      console.error("Failed to fetch apps:", err);
      setIsScanning(false);
    }
  };

  const triggerToast = (message: string, type: 'lock' | 'unlock' | 'success' = 'success') => {
    setToast({ message, visible: true, type });
    setTimeout(() => setToast({ message: "", visible: false, type: 'success' }), 3000);
  };
  
  const pinInputRef = useRef<HTMLInputElement>(null);
  const confirmInputRef = useRef<HTMLInputElement>(null);
  const mainInputRef = useRef<HTMLInputElement>(null);
  const gatekeeperInputRef = useRef<HTMLInputElement>(null);

  const [settingsTab, setSettingsTab] = useState(() => localStorage.getItem("applock_settings_tab") || "account");
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
        const currentWin = getCurrentWindow();
        const windowLabel = currentWin.label;

        // AUTHENTICATION POPUP LOGIC
        if (windowLabel === "gatekeeper") {
          // Load config so authMode and attempt limits work correctly
          const cfg = await invoke<AppConfig>("get_config");
          setConfig(cfg);
          if (cfg.auth_mode) setAuthMode(cfg.auth_mode);

          const blocked = await invoke<LockedApp | null>("get_blocked_app");
          if (blocked) {
            setBlockedApp(blocked);
            setView("gatekeeper");
          } else {
            currentWin.close();
          }
          return;
        }

        // MAIN APPLICATION LOGIC
        const cfg = await invoke<AppConfig>("get_config");
        setConfig(cfg);
        if (cfg.auth_mode) setAuthMode(cfg.auth_mode);

        const isSetup = await invoke<boolean>("check_setup");
        if (!isSetup) {
          setView("onboarding");
        } else {
          const isUnlocked = await invoke<boolean>("get_is_unlocked");
          if (isUnlocked) {
            const persistedView = localStorage.getItem("applock_view") as View;
            if (persistedView && ["dashboard", "setup", "verify"].includes(persistedView)) {
              setView(persistedView);
            } else {
              setView("dashboard");
            }

            const persistedTab = localStorage.getItem("applock_tab") as Tab;
            const persistedSettingsTab = localStorage.getItem("applock_settings_tab");
            if (persistedTab) setActiveTab(persistedTab);
            if (persistedSettingsTab) setSettingsTab(persistedSettingsTab);
          } else {
            setView("unlock");
          }
        }

        const locked = await invoke<LockedApp[]>("get_apps");
        setLockedApps(locked);

        // Fetch detailed apps on initial load
        fetchDetailedApps();
      } catch (err) {
        console.error(err);
      }
    };
    init();
  }, []);

  // Apply visual settings dynamically
  useEffect(() => {
    if (config.animations_intensity === "low") {
      document.documentElement.setAttribute("data-reduced-motion", "true");
    } else {
      document.documentElement.removeAttribute("data-reduced-motion");
    }
  }, [config.animations_intensity]);

  useEffect(() => {
    const unlisten = listen<LockedApp>("app-blocked", async (event) => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const currentWin = getCurrentWindow();
      // Only the popup window should show the gatekeeper screen
      if (currentWin.label === "gatekeeper") {
        setBlockedApp(event.payload);
        setView("gatekeeper");
        setGatekeeperPIN("");
        setError(null);
        currentWin.unminimize();
        currentWin.setFocus();
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

  const handleUnlock = async (e: React.FormEvent, passwordOverride?: string) => {
    if (e) e.preventDefault();
    const passwordToVerify = passwordOverride || password;
    try {
      const isValid = await invoke<boolean>("verify_password", { password: passwordToVerify });
      if (isValid) {
        if (view === "verify") {
          setView("setup");
        } else {
          setView("dashboard");
        }
        setError(null);
        setPassword("");
      } else {
        setError("Invalid security credentials");
        setPassword("");
      }
    } catch (err) { setError(String(err)); }
  };

  const handleGatekeeperUnlock = async (e: React.FormEvent, pinOverride?: string) => {
    if (e) e.preventDefault();
    const pinToVerify = pinOverride || gatekeeperPIN;
    if (!blockedApp) return;
    try {
      const isValid = await invoke<boolean>("verify_gatekeeper", { password: pinToVerify });
      if (isValid) {
        // Close the window IMMEDIATELY on success — before any state updates
        // to prevent re-render races from showing an error state
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        getCurrentWindow().close();
      } else {
        const cfg = await invoke<AppConfig>("get_config");
        setConfig(cfg);
        setError("Invalid security credentials");
        setGatekeeperPIN("");
      }
    } catch (err) {
      // An Err from backend means lockout or config issue, not success
      setError(String(err).replace("Error: ", ""));
      setGatekeeperPIN("");
      const cfg = await invoke<AppConfig>("get_config").catch(() => null);
      if (cfg) setConfig(cfg);
    }
  };

  const toggleApp = async (app: LockedApp | InstalledApp, fromTab?: Tab) => {
    const isLocked = lockedApps.some(la => la.name === app.name);
    if (isLocked) {
      if (fromTab === "all") {
        setAppToRemove(app);
        return;
      }

      // Release app immediately for Unlocked section
      const newLocked = lockedApps.filter(la => la.name !== app.name);
      setLockedApps(newLocked);
      try {
        await invoke("save_selection", { apps: newLocked });
        triggerToast(`${app.name} Unlocked Successfully`, 'unlock');
      } catch (err) { setError(String(err)); }
      return;
    }

    // Lock app immediately
    const newLocked: LockedApp[] = [...lockedApps, {
      id: Math.random().toString(36).substring(2, 9),
      name: app.name,
      exec_name: ((app as any).path || (app as any).exec_name) || "",
      icon: app.icon
    }];
    setLockedApps(newLocked);
    try { 
      await invoke("save_selection", { apps: newLocked }); 
      triggerToast(`${app.name} Locked Successfully`, 'lock');
    } catch (err) { setError(String(err)); }
  };

  const confirmRemoval = async () => {
    if (!appToRemove) return;
    const newLocked = lockedApps.filter(la => la.name !== appToRemove.name);
    setLockedApps(newLocked);
    const removedAppName = appToRemove.name;
    setAppToRemove(null);
    try { 
      await invoke("save_selection", { apps: newLocked }); 
      triggerToast(`${removedAppName} Unlocked Successfully`, 'unlock');
    } catch (err) { setError(String(err)); }
  };

  const bulkUnlock = (apps: LockedApp[]) => {
    setAppsToBulkUnlock(apps);
  };

  const confirmBulkUnlock = async () => {
    if (!appsToBulkUnlock) return;
    const namesToUnlock = new Set(appsToBulkUnlock.map(a => a.name));
    const newLocked = lockedApps.filter(la => !namesToUnlock.has(la.name));
    const count = appsToBulkUnlock.length;
    setLockedApps(newLocked);
    setAppsToBulkUnlock(null);
    try {
      await invoke("save_selection", { apps: newLocked });
      triggerToast(`${count} Apps Unlocked Successfully`, 'unlock');
    } catch (err) { setError(String(err)); }
  };

  const handleLockSession = async () => {
    await invoke("lock_session");
    setView("unlock");
  };

  const updateConfig = async (updates: Partial<AppConfig>) => {
    const newConfig = { ...config, ...updates };
    setConfig(newConfig);
    if (updates.auth_mode) setAuthMode(updates.auth_mode);
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

  // Persist state changes
  useEffect(() => {
    if (view && view !== "onboarding" && view !== "unlock" && view !== "gatekeeper") {
      localStorage.setItem("applock_view", view);
    }
    if (activeTab) localStorage.setItem("applock_tab", activeTab);
    if (settingsTab) localStorage.setItem("applock_settings_tab", settingsTab);
  }, [view, activeTab, settingsTab]);

  useEffect(() => {
    const handleFocus = () => {
      let target: HTMLInputElement | null = null;
      if (view === "unlock" || view === "verify") target = mainInputRef.current;
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
      <AnimatePresence>
        {view === 'gatekeeper' && (
          <motion.div 
            initial={{ opacity: 0 }} 
            animate={{ opacity: 1 }} 
            exit={{ opacity: 0 }} 
            className={styles.gatekeeperOverlay} 
          />
        )}
      </AnimatePresence>
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
        {(view === "unlock" || view === "verify") && (
          <Unlock 
            appName={APP_NAME} authMode={authMode} password={password} error={error}
            isVerify={view === "verify"}
            mainInputRef={mainInputRef} setPassword={setPassword} setError={setError} handleUnlock={handleUnlock}
            onCancel={() => { setView("dashboard"); setPassword(""); setError(null); }}
          />
        )}
        {view === "dashboard" && (
          <Dashboard 
            appName={APP_NAME} activeTab={activeTab} setActiveTab={setActiveTab} showUpdateSuccess={showUpdateSuccess}
            toast={toast}
            search={search} setSearch={setSearch} placeholder={placeholder} handleLockSession={handleLockSession}
            isScanning={isScanning} allApps={allApps} lockedApps={lockedApps} toggleApp={toggleApp}
            refreshApps={fetchDetailedApps}
            bulkUnlock={bulkUnlock}
            settingsTab={settingsTab} setSettingsTab={setSettingsTab} authMode={authMode} setAuthMode={setAuthMode}
            setView={(v) => {
              if (v === "setup") {
                 setIsUpdatingFromSettings(true);
                 setView("verify");
              } else {
                 setView(v);
              }
            }}
            setIsUpdatingFromSettings={setIsUpdatingFromSettings} config={config}
            updateConfig={updateConfig} setShowResetConfirm={setShowResetConfirm}
          />
        )}
        {view === "gatekeeper" && (
          <Gatekeeper 
            blockedApp={blockedApp} authMode={authMode} gatekeeperPIN={gatekeeperPIN} error={error}
            isLaunching={isLaunching} gatekeeperInputRef={gatekeeperInputRef} setGatekeeperPIN={setGatekeeperPIN}
            setError={setError} config={config}
            handleGatekeeperUnlock={handleGatekeeperUnlock} closeWindow={async () => { await invoke("release_app"); }}
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
        {appsToBulkUnlock && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className={styles.modalOverlay}>
            <motion.div initial={{ scale: 0.95 }} animate={{ scale: 1 }} className={styles.modalCard}>
              <div className={styles.modalIcon}><AlertCircle size={40} color="var(--error-color)" /></div>
              <h3>Unlock Multiple?</h3>
              <p>Are you sure you want to remove protection from <strong>{appsToBulkUnlock.length}</strong> applications?</p>
              <div className={styles.modalActions}>
                <button className={styles.modalCancel} onClick={() => setAppsToBulkUnlock(null)}>Cancel</button>
                <button className={styles.modalConfirm} onClick={confirmBulkUnlock}>Remove Locks</button>
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
