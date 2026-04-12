import { motion, AnimatePresence } from "framer-motion";
import { Lock, Unlock, Search, LogOut, Settings, RotateCcw, Home, CheckSquare, Square, Trash2, X, MousePointer2, Monitor, Shield } from "lucide-react";
import { useState, useMemo } from "react";
import clsx from "clsx";
import styles from "../styles/App.module.css";
import logo from "../assets/logo.png";
import { Tab, InstalledApp, LockedApp, AppConfig, AuthMode } from "../types";
import { SettingsPage } from "../components/settings/settings";

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
              {toast.type === 'success' && <img src={logo} style={{ width: 16, height: 16, objectFit: 'contain' }} alt="" />}
              <span>{toast.visible ? toast.message : "Credentials Updated Successfully"}</span>
            </motion.div>
          )}
        </AnimatePresence>
        <div className={styles.headerTitleGroup}>
          <img src={logo} className={styles.headerLogo} alt={`${appName} Logo`} />
          <span className={styles.navBrandName}>AppLock</span>
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
          <div className={styles.combinedActions}>
            <button 
              className={clsx(styles.actionBtn, isScanning && styles.refreshBtnRotating)} 
              onClick={refreshApps} 
              title="Refresh App List"
              disabled={isScanning}
            >
              <RotateCcw size={18} />
            </button>
            <div className={styles.actionDivider} />
            <button className={styles.actionBtn} onClick={handleLockSession} title="Lock Session">
              <LogOut size={18} />
            </button>
          </div>
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
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.5 }}
            className={styles.homeMinimal}
          >
            <div className={styles.homeStatusSection}>
              <div className={styles.statusShield}>
                <Shield className={styles.shieldBaseIcon} size={220} />
                <div className={styles.statusLogoGlow} />
                <img src={logo} className={styles.statusLogoImage} alt="" />
                <div className={styles.statusLogoShine} />

              </div>
                <div className={styles.statusInfo}>
                  <motion.h2
                    initial={{ y: 40, opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    transition={{ duration: 1, ease: [0.19, 1, 0.22, 1], delay: 0.2 }}
                    className={styles.statusTitle}
                  >
                    {appName}
                  </motion.h2>
                  <motion.p 
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: 0.8, duration: 0.8, ease: "easeOut" }}
                    className={styles.statusSubtitle}
                  >
                    System perimeter is currently secured
                  </motion.p>
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
              Start Protection
            </button>
          </motion.div>
        ) : activeTab === "settings" ? (
          <SettingsPage 
            appName={appName} config={config} updateConfig={updateConfig} 
            settingsTab={settingsTab} setSettingsTab={setSettingsTab} 
            authMode={authMode} setAuthMode={setAuthMode} setView={setView} 
            setIsUpdatingFromSettings={setIsUpdatingFromSettings} 
            setShowResetConfirm={setShowResetConfirm}
          />
        ) : (
          <motion.div key={activeTab} initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.4 }} className={styles.appListWrapper}>
            {isScanning ? (
              <div className={styles.emptyState}>
                <div className={styles.premiumLoader}>
                  <motion.div className={styles.loaderRing} animate={{ rotate: 360 }} transition={{ duration: 1.5, repeat: Infinity, ease: "linear" }} />
                  <img src={logo} style={{ width: 28, height: 28, objectFit: 'contain' }} className={styles.loaderIcon} alt="" />
                </div>
                <div className={styles.loaderContent}>
                  <span className={styles.loaderText}>Scanning workspace..</span>
                  <p className={styles.loaderSubtext}>This may take a few seconds.</p>
                </div>
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
