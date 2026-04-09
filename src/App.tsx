import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Lock, Unlock, Shield, AlertCircle, Search, ShieldCheck, ArrowRight, LogOut, Settings, User, Monitor, ChevronDown, RotateCcw, AlertTriangle } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import styles from "./App.module.css";
import clsx from "clsx";

type View = "onboarding" | "setup" | "unlock" | "dashboard" | "gatekeeper";
type AuthMode = "Password" | "PIN";
type Tab = "all" | "system" | "settings";

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

function App() {
  const [view, setView] = useState<View | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("all");
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
    return () => { unlisten.then(f => f()); };
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
      setView("dashboard");
      setError(null);
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
  const [lockoutLimit, setLockoutLimit] = useState("5");
  const [lockoutDuration, setLockoutDuration] = useState("60");

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
      const pinInput = document.getElementById("pin-input");
      const mainInput = document.getElementById("main-unlock-input");
      if (pinInput) pinInput.focus();
      else if (mainInput) mainInput.focus();
    };

    const interval = setInterval(() => {
      let target: HTMLElement | null = null;

      if (view === "unlock" || view === "gatekeeper") {
        target = document.getElementById("pin-input") || document.getElementById("main-unlock-input");
      } else if (view === "setup") {
        if (authMode === "PIN") {
          target = password.length < 4 ? document.getElementById("pin-input") : document.getElementById("confirm-input");
        }
        // In Password mode, we don't force focus because the user needs to 
        // click between the fields manually as they type.
      }

      if (target && document.activeElement !== target) {
        (target as any).focus();
      }
    }, 100);

    window.addEventListener("focus", handleFocus);
    window.addEventListener("click", handleFocus);
    return () => {
      clearInterval(interval);
      window.removeEventListener("focus", handleFocus);
      window.removeEventListener("click", handleFocus);
    };
  }, [view]);

  if (view === null) return null;

  return (
    <div className={clsx(styles.container, view === 'gatekeeper' && styles.transparentBg)}>
      <AnimatePresence mode="wait">
        {view === "onboarding" && (
          <motion.div 
            key="onboarding" 
            initial={{ opacity: 0, y: 8 }} 
            animate={{ opacity: 1, y: 0 }} 
            transition={{ duration: 0.4 }} 
            className={styles.unlockScreen} 
            style={{ maxWidth: '500px' }}
          >
            <div className={styles.unlockIcon}><Shield size={80} strokeWidth={1} /></div>
            <div style={{ textAlign: 'center' }}>
              <h1 className={styles.mainTitle} style={{ fontSize: '2.5rem', marginBottom: '0.5rem' }}>Guardian</h1>
              <p className={styles.unlockSubtitle} style={{ color: 'var(--text-secondary)', letterSpacing: '0.1em' }}>PRECISION PRIVACY FOR WINDOWS</p>
            </div>
            <div className={styles.steps} style={{ width: '100%', margin: '2rem 0' }}>
              {[
                { n: 1, t: "Secure Vault", d: "Set personal master credentials." },
                { n: 2, t: "Map Workspace", d: "Select protected applications." },
                { n: 3, t: "Active Shield", d: "Guardian handles the background." }
              ].map(s => (
                <div key={s.n} className={styles.stepItem} style={{ background: 'rgba(255,255,255,0.02)', border: '1px solid var(--border-color)' }}>
                  <div className={styles.stepNumber} style={{ background: 'var(--accent-color)' }}>{s.n}</div>
                  <div>
                    <div style={{ fontWeight: 600, color: '#fff' }}>{s.t}</div>
                    <div style={{ fontSize: '0.8125rem', color: 'var(--text-secondary)' }}>{s.d}</div>
                  </div>
                </div>
              ))}
            </div>
            <button onClick={() => setView('setup')} className={styles.unlockAction}><span>Initialize Security</span><ArrowRight size={18} /></button>
          </motion.div>
        )}

        {view === "setup" && (
          <motion.div 
            key="setup" 
            initial={{ opacity: 0, y: 8 }} 
            animate={{ opacity: 1, y: 0 }} 
            transition={{ duration: 0.4 }} 
            className={styles.unlockScreen}
          >
            <div className={styles.unlockTitle}>Security Initialization</div>
            <div className={styles.modeToggle} style={{ background: 'rgba(255,255,255,0.03)', padding: '4px', borderRadius: '12px', marginBottom: '1rem' }}>
              <button className={clsx(styles.modeBtn, authMode === "PIN" && styles.modeBtnActive)} onClick={() => { setAuthMode("PIN"); setPassword(""); setConfirmPassword(""); setError(null); }}>PIN</button>
              <button className={clsx(styles.modeBtn, authMode === "Password" && styles.modeBtnActive)} onClick={() => { setAuthMode("Password"); setPassword(""); setConfirmPassword(""); setError(null); }}>Password</button>
            </div>
            <form onSubmit={handleSetup} className={styles.unlockInputWrapper} style={{ gap: '1.5rem' }}>
              {error && <div className={styles.errorMessage} style={{ position: 'absolute', top: '-3.5rem' }}><AlertCircle size={14} /> {error}</div>}
              {authMode === "PIN" ? (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem', alignItems: 'center' }}>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', alignItems: 'center' }}>
                    <span style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', textTransform: 'uppercase' }}>New PIN</span>
                    <div className={styles.pinDisplayGroup}>
                      {[0, 1, 2, 3].map(i => <div key={i} className={clsx(styles.pinBox, password.length === i && styles.pinBoxActive, password.length > i && styles.pinBoxFilled)}>{password.length > i ? "●" : ""}</div>)}
                    </div>
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', alignItems: 'center' }}>
                    <span style={{ fontSize: '0.65rem', color: 'var(--text-secondary)', textTransform: 'uppercase' }}>Confirm PIN</span>
                    <div className={styles.pinDisplayGroup}>
                      {[0, 1, 2, 3].map(i => <div key={i} className={clsx(styles.pinBox, confirmPassword.length === i && styles.pinBoxActive, confirmPassword.length > i && styles.pinBoxFilled)}>{confirmPassword.length > i ? "●" : ""}</div>)}
                    </div>
                  </div>
                  <input id="pin-input" type="password" inputMode="numeric" pattern="\d*" maxLength={4} className={styles.hiddenInput} autoFocus autoComplete="one-time-code" name="new-pin-hidden" value={password} onChange={(e) => {
                    const val = e.target.value.replace(/\D/g, "").slice(0, 4);
                    setPassword(val);
                    if (val.length === 4) document.getElementById("confirm-input")?.focus();
                  }} />
                  <input id="confirm-input" type="password" inputMode="numeric" pattern="\d*" maxLength={4} className={styles.hiddenInput} autoComplete="one-time-code" name="confirm-pin-hidden" value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value.replace(/\D/g, "").slice(0, 4))} />
                </div>
              ) : (
                <div style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                  <input id="main-unlock-input" type="password" className={styles.modernInput} placeholder="Master Password" autoFocus value={password} onChange={(e) => setPassword(e.target.value)} />
                  <input id="confirm-input" type="password" className={styles.modernInput} placeholder="Confirm Password" value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value)} />
                </div>
              )}

              <div style={{ display: 'flex', gap: '1rem', width: '100%' }}>
                {allApps.length > 0 && (
                  <button type="button" className={styles.iconBtn} style={{ flex: 1 }} onClick={() => setView('dashboard')}>Cancel</button>
                )}
                <button type="submit" className={styles.unlockAction} style={{ flex: 2 }}>
                  <span>{allApps.length > 0 ? "Update Credential" : "Initialize Vault"}</span>
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
            <div className={styles.unlockTitle}>Vault Access</div>
            <form onSubmit={handleUnlock} className={styles.unlockInputWrapper}>
              {error && <div className={styles.errorMessage} style={{ position: 'absolute', top: '-3rem' }}><AlertCircle size={14} /> {error}</div>}
              {authMode === "PIN" ? (
                <div className={styles.pinDisplayGroup}>
                  {[0, 1, 2, 3].map(i => <div key={i} className={clsx(styles.pinBox, password.length === i && styles.pinBoxActive, password.length > i && styles.pinBoxFilled)}>{password.length > i ? "●" : ""}</div>)}
                </div>
              ) : <input id="main-unlock-input" type="password" className={styles.modernInput} placeholder="Enter Password" autoFocus value={password} onChange={(e) => setPassword(e.target.value)} />}
              {authMode === "PIN" && (
                <input
                  id="main-unlock-input"
                  type="password"
                  inputMode="numeric"
                  pattern="\d*"
                  maxLength={4}
                  className={styles.hiddenInput}
                  autoFocus
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
              <button type="submit" className={styles.unlockAction}><span>Unlock Vault</span><ArrowRight size={18} /></button>
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
              <div className={styles.headerTitleGroup}>
                <span className={styles.brandLabel}>AppLock</span>
                <h1 className={styles.mainTitle}>Vault</h1>
              </div>

              <div className={styles.headerActions}>
                {activeTab !== "settings" && (
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

            <div className={styles.tabs}>
              <button
                className={clsx(styles.tab, activeTab === "all" && styles.tabActive)}
                onClick={() => setActiveTab("all")}
              >
                <Lock size={16} /> Locked Apps
              </button>
              <button
                className={clsx(styles.tab, activeTab === "system" && styles.tabActive)}
                onClick={() => setActiveTab("system")}
              >
                <Unlock size={16} /> Unlocked Apps
              </button>
              <button
                className={clsx(styles.tab, activeTab === "settings" && styles.tabActive)}
                onClick={() => setActiveTab("settings")}
              >
                <Settings size={16} /> Settings
              </button>
            </div>

            <main className={styles.mainScrollArea}>
              {activeTab === "settings" ? (
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
                            <span>Update your {authMode} to keep your vault secure.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <button className={styles.iconBtn} onClick={() => setView("setup")}>Update {authMode}</button>
                          </div>
                        </div>
                      </section>
                    )}

                    {settingsTab === "security" && (
                      <section className={styles.settingsGroup}>
                        <div className={styles.settingsHeader}>
                          <h2>Security Policy</h2>
                          <p>Configure how the vault responds to intrusions.</p>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Attempt Limit</span>
                            <span>Number of failures before lockout triggers.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <ModernSelect 
                              value={lockoutLimit} 
                              onChange={setLockoutLimit}
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
                              value={lockoutDuration} 
                              onChange={setLockoutDuration}
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
                          <p>Personalize your workspace and launch behavior.</p>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Launch at Startup</span>
                            <span>Automatically wake the vault when Windows starts.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <div className={styles.miniToggle}>
                              <button>Enable</button>
                              <button className={styles.miniToggleActive}>Disable</button>
                            </div>
                          </div>
                        </div>

                        <div className={styles.settingRow}>
                          <div className={styles.settingLabel}>
                            <span>Self-Lock</span>
                            <span>Require authentication to open the dashboard.</span>
                          </div>
                          <div className={styles.settingControl}>
                            <div className={styles.miniToggle}>
                              <button className={styles.miniToggleActive}>Enable</button>
                              <button>Disable</button>
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
                              <button className={styles.miniToggleActive}>Dark</button>
                              <button>Light</button>
                            </div>
                          </div>
                        </div>
                      </section>
                    )}

                    <footer className={styles.settingsFooter}>
                      <div style={{ flex: 1, display: 'flex', alignItems: 'center', gap: '0.6rem' }}>
                        <span style={{ fontSize: '0.65rem', fontWeight: 800, color: 'var(--accent-color)', letterSpacing: '0.1em' }}>APPLOCK</span>
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
                          style={{ color: '#EF233C', fontWeight: 700, textDecoration: 'none' }}
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
                  className={styles.appList}
                >
                  {isScanning ? <div className={styles.emptyState}>Scanning Workspace...</div> :
                    (activeTab === "all" ? lockedApps : allApps)
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
                              {app.icon ? <img src={app.icon} className={styles.appIconImg} alt="" /> : <Shield size={24} color={isLocked ? "var(--accent-color)" : "var(--text-secondary)"} style={{ opacity: isLocked ? 1 : 0.3 }} />}
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
            <form onSubmit={handleGatekeeperUnlock} className={styles.gatekeeperForm}>
              {error && <div className={styles.errorMessage}><AlertCircle size={14} /> {error}</div>}
              {isLaunching ? <div className={styles.launchingState}><div className={styles.spinner} /><span>Launching...</span></div> : (
                <>
                  <div className={styles.pinDisplayGroup}>
                    {[0, 1, 2, 3].map(i => <div key={i} className={clsx(styles.pinBox, gatekeeperPIN.length === i && styles.pinBoxActive, gatekeeperPIN.length > i && styles.pinBoxFilled)}>{gatekeeperPIN.length > i ? "●" : ""}</div>)}
                  </div>
                  <input
                    id="pin-input"
                    type="password"
                    inputMode="numeric"
                    pattern="\d*"
                    maxLength={4}
                    className={styles.hiddenInput}
                    autoFocus
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
              <p>Are you sure you want to unlock <strong>{appToRemove.name}</strong>? This application will no longer be protected by your vault.</p>
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
              <p>Are you sure you want to reset Guardian? This will remove all your protected apps and security settings.</p>
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
