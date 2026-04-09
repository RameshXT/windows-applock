import { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence, useMotionValue, useTransform, animate } from "framer-motion";
import { Lock, Unlock, Shield, AlertCircle, Search, ShieldCheck, ArrowRight, LogOut, Settings, User, Monitor, ChevronDown, RotateCcw, AlertTriangle, Home } from "lucide-react";
import logo from "./assets/logo.png";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import styles from "./App.module.css";
import clsx from "clsx";

const APP_NAME = "Windows AppLock";

const GithubIcon = ({ size = 18 }: { size?: number }) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.28 1.15-.28 2.35 0 3.5-.73 1.02-1.08 2.25-1 3.5 0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
    <path d="M9 18c-4.51 2-5-2-7-2" />
  </svg>
);

type View = "onboarding" | "setup" | "unlock" | "dashboard" | "gatekeeper";
type AuthMode = "Password" | "PIN";
type Tab = "home" | "all" | "system" | "settings";

interface InstalledApp {
  name: string;
  path: string | null;
  icon?: string | null;
}

interface LockedApp {
  id: string;
  name: string;
  exec_name: string;
  icon?: string | null;
}

interface AppConfig {
  hashed_password?: string;
  locked_apps: LockedApp[];
  auth_mode?: AuthMode;
  attempt_limit?: number;
  lockout_duration?: number;
  autostart?: boolean;
  theme?: "dark" | "light";
}

const ModernSelect = ({ value, options, onChange }: { value: string, options: { label: string, value: string }[], onChange: (val: string) => void }) => {
  const [isOpen, setIsOpen] = useState(false);
  const selectedLabel = options.find(o => o.value === value)?.label || "Select...";

  return (
    <div className={styles.selectWrapper}>
      <button
        className={styles.modernSelectBtn}
        onFocus={() => setIsOpen(true)}
        onBlur={() => setTimeout(() => setIsOpen(false), 200)}
      >
        <span>{selectedLabel}</span>
        <motion.div animate={{ rotate: isOpen ? 180 : 0 }}>
          <ChevronDown size={14} className={styles.selectIcon} />
        </motion.div>
      </button>

      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, y: 5 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 5 }}
            className={styles.selectMenu}
          >
            {options.map(opt => (
              <div
                key={opt.value}
                className={clsx(styles.selectOption, opt.value === value && styles.selectOptionActive)}
                onClick={() => { onChange(opt.value); setIsOpen(false); }}
              >
                {opt.label}
              </div>
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

const CountUp = ({ value, color }: { value: number, color?: string }) => {
  const count = useMotionValue(0);
  const rounded = useTransform(count, (latest) => Math.round(latest));

  useEffect(() => {
    const controls = animate(count, value, { duration: 1.5, ease: "easeOut" });
    return controls.stop;
  }, [value]);

  return <motion.span style={{ color }}>{rounded}</motion.span>;
};

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
        // Skip loading, show smart notification
        setShowUpdateSuccess(true);
        setView("dashboard");
        setIsUpdatingFromSettings(false);
        setTimeout(() => setShowUpdateSuccess(false), 3000);
      } else {
        // Initial Flow: Show Artificial Loading Experience
        setIsCompleting(true);
        const messages = [
          "Great! You're all set.",
          "We're preparing your perimeter..",
          "One moment..",
          "Here we go!"
        ];
        
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
        setTimeout(() => {
          setBlockedApp(null);
          setGatekeeperPIN("");
          setIsLaunching(false);
          const { getCurrentWindow } = import("@tauri-apps/api/window") as any;
          if (getCurrentWindow) getCurrentWindow().close();
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

    // Direct add
    const newLocked: LockedApp[] = [...lockedApps, {
      id: Math.random().toString(36).substring(2, 9),
      name: app.name,
      exec_name: ((app as any).path || (app as any).exec_name) || "",
      icon: app.icon
    }];
    setLockedApps(newLocked);
    try {
      await invoke("save_selection", { apps: newLocked });
    } catch (err) { setError(String(err)); }
  };

  const confirmRemoval = async () => {
    if (!appToRemove) return;
    const newLocked = lockedApps.filter(la => la.name !== appToRemove.name);
    setLockedApps(newLocked);
    setAppToRemove(null);
    try {
      await invoke("save_selection", { apps: newLocked });
    } catch (err) { setError(String(err)); }
  };

  const handleLockSession = async () => {
    await invoke("lock_session");
    setView("unlock");
  };

  const [settingsTab, setSettingsTab] = useState("account");
  const [config, setConfig] = useState<AppConfig>({
    locked_apps: [],
    auth_mode: "PIN",
    attempt_limit: 5,
    lockout_duration: 60,
    theme: "dark"
  });

  const updateConfig = async (updates: Partial<AppConfig>) => {
    const newConfig = { ...config, ...updates };
    setConfig(newConfig);
    if (updates.auth_mode) setAuthMode(updates.auth_mode);
    if (updates.theme) document.documentElement.setAttribute("data-theme", updates.theme);
    try {
      await invoke("update_settings", { newConfig });
    } catch (err) { setError(String(err)); }
  };

  // Typing Placeholder Logic
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
        setTimeout(() => setIsDeleting(true), 2000); // Wait 2s before deleting
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

      if (view === "unlock") {
        target = mainInputRef.current;
      } else if (view === "gatekeeper") {
        target = gatekeeperInputRef.current;
      } else if (view === "setup") {
        if (authMode === "PIN") {
          target = password.length < 4 ? pinInputRef.current : confirmInputRef.current;
        } else {
          // Password mode: focus first field if focused element isn't one of the password fields
          if (document.activeElement !== pinInputRef.current && document.activeElement !== confirmInputRef.current) {
            target = pinInputRef.current;
          }
        }
      }

      if (target && document.activeElement !== target) {
        target.focus();
      }
    };

    handleFocus();
    
    // Periodically check focus to ensure it stays on the input
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
        {view === "onboarding" && (
          <motion.div
            key="onboarding"
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
                <Shield size={64} className={styles.unlockIcon} strokeWidth={1} />
              </motion.div>
              <h1 className={styles.onboardingTitle}>{APP_NAME}</h1>
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
                onClick={() => setView('setup')}
                className={styles.primaryBtn}
              >
                <span>Initialize Security</span>
                <ArrowRight size={20} />
              </button>
              <span className={styles.versionBadge}>VERSION 2.1.0 • SECURED BY QUANTUM</span>
            </motion.div>
          </motion.div>
        )}

        {isCompleting && (
          <motion.div
            key="completing"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className={styles.unlockScreen}
          >
            <div className={styles.premiumLoader} style={{ width: '80px', height: '80px', marginBottom: '2rem' }}>
              <motion.div
                className={styles.loaderRing}
                style={{ borderWidth: '3px' }}
                animate={{ rotate: 360 }}
                transition={{ duration: 1, repeat: Infinity, ease: "linear" }}
              />
              <Shield size={32} className={styles.loaderIcon} />
            </div>
            
            <AnimatePresence mode="wait">
              <motion.div
                key={completingStep}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                transition={{ duration: 0.4 }}
                style={{ textAlign: 'center' }}
              >
                <h2 className={styles.statusTitle} style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>
                  {[
                    "Great! You're all set.",
                    "We're preparing your perimeter...",
                    "One moment...",
                    "Here we go!"
                  ][completingStep]}
                </h2>
                <p className={styles.statusSubtitle}>Initializing secure environment</p>
              </motion.div>
            </AnimatePresence>
          </motion.div>
        )}

        {view === "setup" && !isCompleting && (
          <motion.div
            key="setup"
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4 }}
            className={styles.unlockScreen}
            style={{ maxWidth: '440px' }}
          >
            <div className={styles.gatekeeperBrand} style={{ marginBottom: '1rem' }}>
              <div className={styles.statusCircle} style={{ width: '64px', height: '64px', marginBottom: '1.5rem' }}>
                <Shield size={32} strokeWidth={1.5} />
              </div>
              <h1 className={styles.statusTitle} style={{ fontSize: '1.75rem' }}>Security Protocol</h1>
              <p className={styles.statusSubtitle}>Configure your master authentication method</p>
            </div>

            <div className={styles.tabs} style={{ marginBottom: '0.5rem' }}>
              <button
                className={clsx(styles.tab, authMode === "PIN" && styles.tabActive)}
                onClick={() => { setAuthMode("PIN"); setPassword(""); setConfirmPassword(""); setError(null); }}
              >
                PIN
              </button>
              <button
                className={clsx(styles.tab, authMode === "Password" && styles.tabActive)}
                onClick={() => { setAuthMode("Password"); setPassword(""); setConfirmPassword(""); setError(null); }}
              >
                Password
              </button>
            </div>

            <form onSubmit={handleSetup} className={styles.unlockInputWrapper} style={{ gap: '2rem', width: '100%' }}>
              {error && <div className={styles.errorMessage} style={{ position: 'absolute', top: '-3.5rem' }}><AlertCircle size={14} /> {error}</div>}

              {authMode === "PIN" ? (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem', alignItems: 'center', width: '100%' }}>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', alignItems: 'center', width: '100%', opacity: password.length < 4 ? 1 : 0.4, transition: 'all 0.3s ease' }}>
                    <span style={{ fontSize: '0.7rem', fontWeight: 700, color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.1em', opacity: 0.5 }}>New Secret PIN</span>
                    <div className={styles.pinDisplayGroup}>
                      {[0, 1, 2, 3].map(i => (
                        <div key={i} className={clsx(
                          styles.pinBox, 
                          password.length < 4 && password.length === i && styles.pinBoxActive, 
                          password.length > i && styles.pinBoxFilled
                        )}>
                          {password.length > i ? "●" : ""}
                        </div>
                      ))}
                    </div>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', alignItems: 'center', width: '100%', opacity: password.length === 4 ? 1 : 0.15, transition: 'all 0.3s ease' }}>
                    <span style={{ fontSize: '0.7rem', fontWeight: 700, color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.1em', opacity: 0.5 }}>Confirm Secret PIN</span>
                    <div className={styles.pinDisplayGroup}>
                      {[0, 1, 2, 3].map(i => (
                        <div key={i} className={clsx(
                          styles.pinBox, 
                          password.length === 4 && confirmPassword.length === i && styles.pinBoxActive, 
                          confirmPassword.length > i && styles.pinBoxFilled
                        )}>
                          {confirmPassword.length > i ? "●" : ""}
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
                    value={password} 
                    onChange={(e) => setPassword(e.target.value.replace(/\D/g, "").slice(0, 4))} 
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
                    value={confirmPassword} 
                    onChange={(e) => setConfirmPassword(e.target.value.replace(/\D/g, "").slice(0, 4))} 
                    onKeyDown={(e) => {
                      if (e.key === "Backspace" && confirmPassword.length === 0) {
                        setPassword(password.slice(0, -1));
                      }
                    }}
                  />
                </div>
              ) : (
                <div style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                    <span style={{ fontSize: '0.7rem', fontWeight: 700, color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.1em', opacity: 0.5, marginLeft: '0.5rem' }}>Master Password</span>
                    <input ref={pinInputRef} type="password" className={styles.modernInput} placeholder="••••••••" value={password} onChange={(e) => setPassword(e.target.value)} />
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                    <span style={{ fontSize: '0.7rem', fontWeight: 700, color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.1em', opacity: 0.5, marginLeft: '0.5rem' }}>Confirm Password</span>
                    <input ref={confirmInputRef} type="password" className={styles.modernInput} placeholder="••••••••" value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value)} />
                  </div>
                </div>
              )}

              <div style={{ display: 'flex', gap: '1rem', width: '100%', marginTop: '1rem' }}>
                {allApps.length > 0 && (
                  <button type="button" className={styles.modalCancel} style={{ flex: 1, height: '56px' }} onClick={() => setView('dashboard')}>Cancel</button>
                )}
                <button type="submit" className={styles.unlockAction} style={{ flex: 2, height: '56px', justifyContent: 'center' }}>
                  <span>{allApps.length > 0 ? "Update" : `Continue Registration`}</span>
                  <ArrowRight size={18} />
                </button>
              </div>
            </form>
          </motion.div>
        )}


        {view === "unlock" && (
          <motion.div
            key="unlock"
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4 }}
            className={styles.unlockScreen}
          >
            <div className={styles.unlockIcon}><Shield size={64} strokeWidth={1.5} /></div>
            <div className={styles.unlockTitle}>{APP_NAME} Access</div>
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
                  pattern="\d*"
                  maxLength={4}
                  className={styles.hiddenInput}
                  autoComplete="one-time-code"
                  name="pin-unlock-hidden"
                  value={password}
                  onChange={(e) => {
                    const val = e.target.value.replace(/\D/g, "").slice(0, 4);
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
        )}

        {view === "dashboard" && (
          <motion.div
            key="dashboard"
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4 }}
            className={styles.dashboard}
          >
            <header className={styles.header}>
              <AnimatePresence>
                {showUpdateSuccess && (
                  <motion.div
                    initial={{ opacity: 0, y: -20, x: '-50%' }}
                    animate={{ opacity: 1, y: 0, x: '-50%' }}
                    exit={{ opacity: 0, y: -20, x: '-50%' }}
                    className={styles.successToast}
                  >
                    <ShieldCheck size={16} />
                    <span>Credentials Updated Successfully</span>
                  </motion.div>
                )}
              </AnimatePresence>
              <div className={styles.headerTitleGroup}>
                <img src={logo} className={styles.headerLogo} alt={`${APP_NAME} Logo`} />
              </div>

              <div className={styles.tabs}>
                <button
                  className={clsx(styles.tab, activeTab === "home" && styles.tabActive)}
                  onClick={() => setActiveTab("home")}
                >
                  <Home size={18} /> <span>Home</span>
                </button>
                <button
                  className={clsx(styles.tab, activeTab === "all" && styles.tabActive)}
                  onClick={() => setActiveTab("all")}
                >
                  <Lock size={18} /> <span>Locked Apps</span>
                </button>
                <button
                  className={clsx(styles.tab, activeTab === "system" && styles.tabActive)}
                  onClick={() => setActiveTab("system")}
                >
                  <Unlock size={18} /> <span>Unlocked Apps</span>
                </button>
                <button
                  className={clsx(styles.tab, activeTab === "settings" && styles.tabActive)}
                  onClick={() => setActiveTab("settings")}
                >
                  <Settings size={18} /> <span>Settings</span>
                </button>
              </div>

              <div className={styles.headerActions}>
                {activeTab !== "settings" && activeTab !== "home" && (
                  <div className={styles.searchBar}>
                    <Search size={16} color="var(--text-secondary)" />
                    <input
                      placeholder={`Search ${placeholder}|`}
                      value={search}
                      onChange={(e) => setSearch(e.target.value)}
                    />
                  </div>
                )}

                <button className={styles.logoutBtn} onClick={handleLockSession} title="Lock Session">
                  <LogOut size={20} />
                </button>
              </div>
            </header>

            <div className={styles.listDivider}>
              <div className={styles.dividerLine} />
              {(activeTab === "all" || activeTab === "system") && !isScanning && (() => {
                const count = (activeTab === "all" ? lockedApps : allApps).filter(app => app.name.toLowerCase().includes(search.toLowerCase())).length;
                return (
                  <span className={styles.dividerText}>
                    {count === 0 ? "No Apps Found" : `${count} ${count === 1 ? "App" : "Apps"} Found`}
                  </span>
                )
              })()}
              {(activeTab === "all" || activeTab === "system") && !isScanning && <div className={styles.dividerLine} />}
            </div>

            <main className={styles.mainScrollArea}>
              {activeTab === "home" ? (
                <motion.div
                  key="home"
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.5 }}
                  className={styles.homeMinimal}
                >
                  <div className={styles.homeStatusSection}>
                    <div className={styles.statusCircle}>
                      <ShieldCheck size={48} strokeWidth={1.5} />
                    </div>
                    <div className={styles.statusInfo}>
                      <h2 className={styles.statusTitle}>{APP_NAME} Active</h2>
                      <p className={styles.statusSubtitle}>System perimeter is currently secured</p>
                    </div>
                  </div>

                  <div className={styles.minimalStats}>
                    <div className={styles.minStat}>
                      {isScanning ? (
                        <div className={styles.skeletonValue} />
                      ) : (
                        <span className={styles.minStatValue}>
                          <CountUp value={allApps.length} />
                        </span>
                      )}
                      <span className={styles.minStatLabel}>Total Apps</span>
                    </div>
                    <div className={styles.minStatDivider} />
                    <div className={styles.minStat}>
                      {isScanning ? (
                        <div className={styles.skeletonValue} />
                      ) : (
                        <span className={styles.minStatValue}>
                          <CountUp value={lockedApps.length} color="var(--accent-color)" />
                        </span>
                      )}
                      <span className={styles.minStatLabel}>Locked</span>
                    </div>
                  </div>

                  <button className={styles.minimalAction} onClick={() => setActiveTab("all")}>
                    Manage Protection <ArrowRight size={18} />
                  </button>
                </motion.div>
              ) : activeTab === "settings" ? (
                <div className={styles.settingsContainer}>
                  <aside className={styles.settingsSidebar}>
                    <button className={clsx(styles.settingsNavBtn, settingsTab === "account" && styles.settingsNavBtnActive)} onClick={() => setSettingsTab("account")}>
                      <User size={18} /> Account & Setup
                    </button>
                    <button className={clsx(styles.settingsNavBtn, settingsTab === "security" && styles.settingsNavBtnActive)} onClick={() => setSettingsTab("security")}>
                      <ShieldCheck size={18} /> Security Policy
                    </button>
                    <button className={clsx(styles.settingsNavBtn, settingsTab === "system" && styles.settingsNavBtnActive)} onClick={() => setSettingsTab("system")}>
                      <Monitor size={18} /> System & Style
                    </button>
                    <button className={clsx(styles.settingsNavBtn, settingsTab === "contribution" && styles.settingsNavBtnActive)} onClick={() => setSettingsTab("contribution")}>
                      <GithubIcon size={18} /> Contribution
                    </button>
                    <div style={{ flex: 1 }} />
                    <button
                      className={styles.dangerBtnMinimal}
                      onClick={() => setShowResetConfirm(true)}
                    >
                      <RotateCcw size={18} />
                      Factory Reset
                    </button>
                  </aside>

                  <div className={styles.settingsContent}>
                    {settingsTab === "account" && (
                      <section className={styles.settingsGroup}>
                        <div className={styles.settingsHeader}>
                          <h2>Account & Setup</h2>
                          <p>Manage your entry protocol and core identity.</p>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Authentication Mode</span>
                            <span>Choose between a numeric PIN or a text password.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <div className={styles.miniToggle}>
                              <button className={clsx(authMode === "PIN" && styles.miniToggleActive)} onClick={() => setAuthMode("PIN")}>PIN</button>
                              <button className={clsx(authMode === "Password" && styles.miniToggleActive)} onClick={() => setAuthMode("Password")}>Password</button>
                            </div>
                          </div>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Security Credential</span>
                            <span>Update your {authMode} to keep your {APP_NAME} secure.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <button className={styles.iconBtn} onClick={() => { setIsUpdatingFromSettings(true); setView("setup"); }}>Update {authMode}</button>
                          </div>
                        </div>
                      </section>
                    )}

                    {settingsTab === "security" && (
                      <section className={styles.settingsGroup}>
                        <div className={styles.settingsHeader}>
                          <h2>Security Policy</h2>
                          <p>Configure how {APP_NAME} responds to intrusions.</p>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Attempt Limit</span>
                            <span>Number of failures before lockout triggers.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <ModernSelect
                              value={String(config.attempt_limit)}
                              onChange={(val) => updateConfig({ attempt_limit: parseInt(val) })}
                              options={[
                                { label: "3 Attempts", value: "3" },
                                { label: "5 Attempts", value: "5" },
                                { label: "10 Attempts", value: "10" }
                              ]}
                            />
                          </div>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Lockout Duration</span>
                            <span>Cooldown period when limit is reached.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <ModernSelect
                              value={String(config.lockout_duration)}
                              onChange={(val) => updateConfig({ lockout_duration: parseInt(val) })}
                              options={[
                                { label: "30 Seconds", value: "30" },
                                { label: "1 Minute", value: "60" },
                                { label: "5 Minutes", value: "300" }
                              ]}
                            />
                          </div>
                        </div>
                      </section>
                    )}

                    {settingsTab === "system" && (
                      <section className={styles.settingsGroup}>
                        <div className={styles.settingsHeader}>
                          <h2>System & Appearance</h2>
                          <p>Personalize your workspace and {APP_NAME} behavior.</p>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Launch at Startup</span>
                            <span>Automatically wake {APP_NAME} when Windows starts.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <div className={styles.miniToggle}>
                              <button className={clsx(config.autostart && styles.miniToggleActive)} onClick={() => updateConfig({ autostart: true })}>Enable</button>
                              <button className={clsx(!config.autostart && styles.miniToggleActive)} onClick={() => updateConfig({ autostart: false })}>Disable</button>
                            </div>
                          </div>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Appearance Mode</span>
                            <span>Select your preferred visual style.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <div className={styles.miniToggle}>
                              <button className={clsx(config.theme === "dark" && styles.miniToggleActive)} onClick={() => updateConfig({ theme: "dark" })}>Dark</button>
                              <button className={clsx(config.theme === "light" && styles.miniToggleActive)} onClick={() => updateConfig({ theme: "light" })}>Light</button>
                            </div>
                          </div>
                        </div>
                      </section>
                    )}

                    {settingsTab === "contribution" && (
                      <section className={styles.settingsGroup}>
                        <div className={styles.settingsHeader}>
                          <h2>Contribution</h2>
                          <p>{APP_NAME} is open source. Help us shape the future of privacy.</p>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Open Source</span>
                            <span>Explore the source code, report bugs, or submit features.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <a
                              href="https://github.com/RameshXT/windows-applock"
                              target="_blank"
                              rel="noopener noreferrer"
                              className={styles.iconBtn}
                              style={{ textDecoration: 'none', display: 'flex', alignItems: 'center', gap: '8px' }}
                            >
                              <GithubIcon size={16} />
                              Repository
                            </a>
                          </div>
                        </div>
                      </section>
                    )}

                    <footer className={styles.settingsFooter}>
                      <div style={{ flex: 1, display: 'flex', alignItems: 'center', gap: '0.6rem' }}>
                        <span style={{ fontSize: '0.65rem', fontWeight: 800, color: 'var(--accent-color)', letterSpacing: '0.1em' }}>{APP_NAME.toUpperCase()}</span>
                        <div style={{ width: '3px', height: '3px', borderRadius: '50%', background: 'rgba(255,255,255,0.2)' }} />
                        <span style={{ fontSize: '0.7rem', fontWeight: 500, color: '#fff', opacity: 0.3 }}>V1.0.4</span>
                      </div>

                      <div style={{ flex: 1, display: 'flex', justifyContent: 'center', alignItems: 'center', gap: '0.5rem' }}>
                        <ShieldCheck size={12} color="var(--accent-color)" style={{ opacity: 0.5 }} />
                        <span style={{ fontSize: '0.6rem', fontWeight: 700, letterSpacing: '0.1em', color: '#fff', opacity: 0.3 }}>VERIFIED</span>
                      </div>

                      <div style={{ flex: 1, display: 'flex', justifyContent: 'flex-end', alignItems: 'center', gap: '0.4rem', fontSize: '0.7rem' }}>
                        <span style={{ opacity: 0.4 }}>Designed & Developed by</span>
                        <a
                          href="https://rameshxt.pages.dev/"
                          target="_blank"
                          rel="noopener noreferrer"
                          className={styles.developerLink}
                        >
                          Ramesh XT
                        </a>
                      </div>
                    </footer>
                  </div>
                </div>
              ) : (
                <motion.div
                  key={activeTab}
                  initial={{ opacity: 0, y: 8 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4 }}
                  className={styles.appListWrapper}
                >
                  {isScanning ? (
                    <div className={styles.emptyState}>
                      <div className={styles.premiumLoader}>
                        <motion.div
                          className={styles.loaderRing}
                          animate={{ rotate: 360 }}
                          transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }}
                        />
                        <Shield size={20} className={styles.loaderIcon} />
                      </div>
                      <span className={styles.loaderText}>Scanning Workspace</span>
                    </div>
                  ) : (
                    <div className={styles.appList}>
                      {(activeTab === "all" ? lockedApps : allApps)
                        .filter(app => app.name.toLowerCase().includes(search.toLowerCase()))
                        .map(app => {
                          const isLocked = lockedApps.some(la => la.name === app.name);
                          return (
                            <motion.div
                              layout
                              key={app.name}
                              initial={{ opacity: 0 }}
                              animate={{ opacity: 1 }}
                              transition={{ duration: 0.4 }}
                              whileHover={{ y: -2 }}
                              className={clsx(styles.appCard, isLocked && styles.appCardLocked)}
                              onClick={() => toggleApp(app)}
                            >
                              {isLocked && <div className={styles.lockedBadge}><Lock size={8} /> LOCKED</div>}
                              <div className={styles.appIconContainer}>
                                {app.icon ? (
                                  <img src={app.icon} className={styles.appIconImg} alt="" />
                                ) : (
                                  <img src={logo} className={styles.appIconImg} style={{ opacity: isLocked ? 1 : 0.4 }} alt="" />
                                )}
                              </div>
                              <div className={styles.appInfo}>
                                <div className={styles.appName}>{app.name}</div>
                                <div className={styles.appPath}>{(app as any).exec_name || (app as any).path}</div>
                              </div>
                              <div className={styles.lockIndicator}>
                                {isLocked ? <Lock size={18} className={styles.lockedIcon} /> : <Unlock size={18} style={{ opacity: 0.2 }} />}
                              </div>
                            </motion.div>
                          );
                        })}
                    </div>
                  )}
                </motion.div>
              )}
            </main>
          </motion.div>
        )}

        {view === "gatekeeper" && (
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
                      pattern="\d*"
                      maxLength={4}
                      className={styles.hiddenInput}
                      autoComplete="one-time-code"
                      name="gatekeeper-pin-hidden"
                      value={gatekeeperPIN}
                      onChange={(e) => {
                        const val = e.target.value.replace(/\D/g, "").slice(0, 4);
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
              <button type="button" onClick={async () => { const { getCurrentWindow } = await import("@tauri-apps/api/window"); getCurrentWindow().close(); }} className={styles.cancelBtn}>Cancel</button>
            </div>
          </motion.div>
        )}

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
                <button
                  className={styles.modalConfirm}
                  style={{ background: '#f59e0b' }}
                  onClick={() => {
                    setShowResetConfirm(false);
                    setShowResetFinal(true);
                  }}
                >
                  Yes, Continue
                </button>
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
                <button
                  className={styles.modalConfirm}
                  onClick={async () => {
                    await invoke("reset_app");
                    window.location.reload();
                  }}
                >
                  Reset Everything
                </button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
