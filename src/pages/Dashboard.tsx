import { motion, AnimatePresence } from "framer-motion";
import { Lock, Unlock, Shield, Search, ShieldCheck, ArrowRight, LogOut, Settings, User, Monitor, RotateCcw, Home, CheckSquare, Square, Trash2, X, MousePointer2 } from "lucide-react";
import { useState, useMemo } from "react";
import clsx from "clsx";
import styles from "../styles/App.module.css";
import logo from "../assets/logo.png";
import { Tab, InstalledApp, LockedApp, AppConfig, AuthMode } from "../types";
import { ModernSelect } from "../components/ModernSelect";
import { GithubIcon } from "../components/GithubIcon";

interface DashboardProps {
  appName: string;
  activeTab: Tab;
  setActiveTab: (tab: Tab) => void;
  showUpdateSuccess: boolean;
  toast: { message: string, visible: boolean, type: 'lock' | 'unlock' | 'success' };
  search: string;
  setSearch: (val: string) => void;
  placeholder: string;
  handleLockSession: () => void;
  isScanning: boolean;
  allApps: InstalledApp[];
  lockedApps: LockedApp[];
  toggleApp: (app: LockedApp | InstalledApp, fromTab?: Tab) => void;
  settingsTab: string;
  setSettingsTab: (tab: string) => void;
  authMode: AuthMode;
  setAuthMode: (mode: AuthMode) => void;
  setView: (view: any) => void;
  setIsUpdatingFromSettings: (val: boolean) => void;
  config: AppConfig;
  updateConfig: (updates: Partial<AppConfig>) => void;
  setShowResetConfirm: (val: boolean) => void;
  bulkUnlock: (apps: LockedApp[]) => void;
  refreshApps: () => void;
}

export const Dashboard = ({
  appName,
  activeTab,
  setActiveTab,
  showUpdateSuccess,
  toast,
  search,
  setSearch,
  placeholder,
  handleLockSession,
  isScanning,
  allApps,
  lockedApps,
  toggleApp,
  settingsTab,
  setSettingsTab,
  authMode,
  setAuthMode,
  setView,
  setIsUpdatingFromSettings,
  config,
  updateConfig,
  setShowResetConfirm,
  bulkUnlock,
  refreshApps
}: DashboardProps) => {
  const [selectionMode, setSelectionMode] = useState(false);
  const [selectedNames, setSelectedNames] = useState<Set<string>>(new Set());

  const toggleSelection = (name: string) => {
    const newSelected = new Set(selectedNames);
    if (newSelected.has(name)) newSelected.delete(name);
    else newSelected.add(name);
    setSelectedNames(newSelected);
  };

  const handleBulkUnlock = () => {
    const appsToUnlock = lockedApps.filter(app => selectedNames.has(app.name));
    bulkUnlock(appsToUnlock);
    setSelectionMode(false);
    setSelectedNames(new Set());
  };

  const appsToShow = useMemo(() => {
    const list = activeTab === "all" ? lockedApps : allApps;
    return list.filter(app => app.name.toLowerCase().includes(search.toLowerCase()));
  }, [activeTab, lockedApps, allApps, search]);
  return (
    <motion.div
      key="dashboard"
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
      className={styles.dashboard}
    >
      <header className={styles.header}>
        <AnimatePresence>
          {(showUpdateSuccess || toast.visible) && (
            <motion.div
              initial={{ opacity: 0, y: -20, x: '-50%' }}
              animate={{ opacity: 1, y: 0, x: '-50%' }}
              exit={{ opacity: 0, y: -20, x: '-50%' }}
              className={clsx(styles.successToast, toast.type === 'lock' && styles.toastLock, toast.type === 'unlock' && styles.toastUnlock)}
            >
              {toast.type === 'lock' && <Lock size={16} />}
              {toast.type === 'unlock' && <Unlock size={16} />}
              {toast.type === 'success' && <ShieldCheck size={16} />}
              <span>{toast.visible ? toast.message : "Credentials Updated Successfully"}</span>
            </motion.div>
          )}
        </AnimatePresence>
        <div className={styles.headerTitleGroup}>
          <img src={logo} className={styles.headerLogo} alt={`${appName} Logo`} />
        </div>

        <div className={styles.tabs}>
          <button className={clsx(styles.tab, activeTab === "home" && styles.tabActive)} onClick={() => setActiveTab("home")}>
            <Home size={18} /> <span>Home</span>
          </button>
          <button className={clsx(styles.tab, activeTab === "system" && styles.tabActive)} onClick={() => setActiveTab("system")}>
            <Unlock size={18} /> <span>Unlocked Apps</span>
          </button>
          <button className={clsx(styles.tab, activeTab === "all" && styles.tabActive)} onClick={() => setActiveTab("all")}>
            <Lock size={18} /> <span>Locked Apps</span>
          </button>
          <button className={clsx(styles.tab, activeTab === "settings" && styles.tabActive)} onClick={() => setActiveTab("settings")}>
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
          {activeTab !== "settings" && activeTab !== "home" && (
            <button 
              className={clsx(styles.refreshBtn, isScanning && styles.refreshBtnRotating)} 
              onClick={refreshApps} 
              title="Refresh App List"
              disabled={isScanning}
            >
              <RotateCcw size={18} />
            </button>
          )}

          <button className={styles.logoutBtn} onClick={handleLockSession} title="Lock Session">
            <LogOut size={20} />
          </button>
        </div>
      </header>

      <div className={styles.listDivider}>
        <div className={styles.dividerLine} />
        {(activeTab === "all" || activeTab === "system") && !isScanning && (() => {
          const count = appsToShow.length;
          return (
            <div className={styles.dividerControls}>
              <span className={styles.dividerText}>
                {count === 0 ? "No Apps Found" : `${count} ${count === 1 ? "App" : "Apps"} Found`}
              </span>
              {activeTab === "all" && count > 0 && (
                <button
                  className={clsx(styles.selectionToggle, selectionMode && styles.selectionToggleActive)}
                  onClick={() => {
                    setSelectionMode(!selectionMode);
                    setSelectedNames(new Set());
                  }}
                >
                  {selectionMode ? <X size={14} /> : <MousePointer2 size={14} />}
                  {selectionMode ? "Cancel" : "Select"}
                </button>
              )}
            </div>
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
                <img src={logo} style={{ width: 64, height: 64, objectFit: 'contain' }} alt="" />
              </div>
              <div className={styles.statusInfo}>
                <h2 className={styles.statusTitle}>{appName} Active</h2>
                <p className={styles.statusSubtitle}>System perimeter is currently secured</p>
              </div>
            </div>

            <div className={styles.minimalStats}>
              <div className={styles.minStat}>
                {isScanning ? <div className={styles.skeletonValue} /> : <span className={styles.minStatValue}>{allApps.length}</span>}
                <span className={styles.minStatLabel}>Total Apps</span>
              </div>
              <div className={styles.minStatDivider} />
              <div className={styles.minStat}>
                {isScanning ? <div className={styles.skeletonValue} /> : <span className={styles.minStatValue} style={{ color: "var(--accent-color)" }}>{lockedApps.length}</span>}
                <span className={styles.minStatLabel}>Locked</span>
              </div>
            </div>

            <button className={styles.minimalAction} onClick={() => setActiveTab("system")}>
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
              <button className={styles.dangerBtnMinimal} onClick={() => setShowResetConfirm(true)}>
                <RotateCcw size={18} /> Factory Reset
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
                      <span>Update your {authMode} to keep your {appName} secure.</span>
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
                    <p>Configure how {appName} responds to intrusions.</p>
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
                        options={[{ label: "3 Attempts", value: "3" }, { label: "5 Attempts", value: "5" }, { label: "10 Attempts", value: "10" }]}
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
                        options={[{ label: "30 Seconds", value: "30" }, { label: "1 Minute", value: "60" }, { label: "5 Minutes", value: "300" }]}
                      />
                    </div>
                  </div>
                </section>
              )}

              {settingsTab === "system" && (
                <section className={styles.settingsGroup}>
                  <div className={styles.settingsHeader}>
                    <h2>System & Appearance</h2>
                    <p>Personalize your workspace and {appName} behavior.</p>
                  </div>
                  <div className={styles.settingRow}>
                    <div className={styles.settingLabel}>
                      <span>Launch at Startup</span>
                      <span>Automatically wake {appName} when Windows starts.</span>
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
                    <p>{appName} is open source. Help us shape the future of privacy.</p>
                  </div>
                  <div className={styles.settingRow}>
                    <div className={styles.settingLabel}>
                      <span>Open Source</span>
                      <span>Explore the source code, report bugs, or submit features.</span>
                    </div>
                    <div className={styles.settingControl}>
                      <a href="https://github.com/RameshXT/windows-applock" target="_blank" rel="noopener noreferrer" className={styles.iconBtn} style={{ textDecoration: 'none', display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <GithubIcon size={16} /> Repository
                      </a>
                    </div>
                  </div>
                </section>
              )}

              <footer className={styles.settingsFooter}>
                <div style={{ flex: 1, display: 'flex', alignItems: 'center', gap: '0.6rem' }}>
                  <span style={{ fontSize: '0.65rem', fontWeight: 800, color: 'var(--accent-color)', letterSpacing: '0.1em' }}>{appName.toUpperCase()}</span>
                  <div style={{ width: '3px', height: '3px', borderRadius: '50%', background: 'rgba(255,255,255,0.2)' }} />
                  <span style={{ fontSize: '0.7rem', fontWeight: 500, color: '#fff', opacity: 0.3 }}>V1.0.4</span>
                </div>
                <div style={{ flex: 1, display: 'flex', justifyContent: 'center', alignItems: 'center', gap: '0.5rem' }}>
                  <ShieldCheck size={12} color="var(--accent-color)" style={{ opacity: 0.5 }} />
                  <span style={{ fontSize: '0.6rem', fontWeight: 700, letterSpacing: '0.1em', color: '#fff', opacity: 0.3 }}>VERIFIED</span>
                </div>
                <div style={{ flex: 1, display: 'flex', justifyContent: 'flex-end', alignItems: 'center', gap: '0.4rem', fontSize: '0.7rem' }}>
                  <span style={{ opacity: 0.4 }}>Designed & Developed by</span>
                  <a href="https://rameshxt.pages.dev/" target="_blank" rel="noopener noreferrer" className={styles.developerLink}>Ramesh XT</a>
                </div>
              </footer>
            </div>
          </div>
        ) : (
          <motion.div key={activeTab} initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.4 }} className={styles.appListWrapper}>
            {isScanning ? (
              <div className={styles.emptyState}>
                <div className={styles.premiumLoader}>
                  <motion.div className={styles.loaderRing} animate={{ rotate: 360 }} transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }} />
                  <Shield size={20} className={styles.loaderIcon} />
                </div>
                <span className={styles.loaderText}>Scanning Workspace</span>
              </div>
            ) : (
              <div className={styles.appList}>
                {appsToShow.map(app => {
                  const isLocked = lockedApps.some(la => la.name === app.name);
                  const isSelected = selectedNames.has(app.name);

                  return (
                    <motion.div
                      layout
                      key={app.name}
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      transition={{ duration: 0.4 }}
                      whileHover={{ y: -2 }}
                      className={clsx(
                        styles.appCard,
                        isLocked && styles.appCardLocked,
                        isSelected && styles.appCardSelected,
                        selectionMode && activeTab === "all" && styles.appCardSelectable
                      )}
                      onClick={() => {
                        if (selectionMode && activeTab === "all") {
                          toggleSelection(app.name);
                        } else {
                          toggleApp(app, activeTab);
                        }
                      }}
                    >
                      {selectionMode && activeTab === "all" && (
                        <div className={styles.selectionIndicator}>
                          {isSelected ? <CheckSquare size={18} color="var(--accent-color)" /> : <Square size={18} opacity={0.3} />}
                        </div>
                      )}
                      {isLocked && !selectionMode && <div className={styles.lockedBadge}><Lock size={8} /> LOCKED</div>}
                      <div className={styles.appIconContainer}>
                        {app.icon
                          ? <img src={app.icon} className={styles.appIconImg} alt="" />
                          : <div className={styles.appIconFallback}><Monitor size={20} opacity={isLocked ? 0.9 : 0.3} /></div>
                        }
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

      <AnimatePresence>
        {selectionMode && selectedNames.size > 0 && (
          <motion.div
            initial={{ opacity: 0, y: 50, x: "-50%" }}
            animate={{ opacity: 1, y: 0, x: "-50%" }}
            exit={{ opacity: 0, y: 50, x: "-50%" }}
            className={styles.bulkActionBar}
          >
            <div className={styles.bulkActionInfo}>
              <span className={styles.selectionCount}>{selectedNames.size}</span>
              <span className={styles.selectionLabel}>Apps Selected</span>
            </div>
            <div className={styles.bulkActionButtons}>
              <button className={styles.bulkUnlockBtn} onClick={handleBulkUnlock}>
                <Trash2 size={16} /> Unlock Selection
              </button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
};
