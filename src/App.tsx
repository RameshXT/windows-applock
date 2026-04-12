import { useRef, useState, useEffect } from "react";
import { AnimatePresence, motion } from "framer-motion";
import styles from "./styles/App.module.css";
import clsx from "clsx";

import { View } from "./types";
import { APP_NAME, STORAGE_KEYS } from "./constants";

import {
  useAppInit,
  useAuth,
  useApps,
  useConfig,
  useToast,
  useFocusGuard,
  usePlaceholder,
} from "./hooks";

import { Onboarding } from "./pages/Onboarding";
import { Setup } from "./pages/Setup";
import { Unlock } from "./pages/Unlock";
import { Dashboard } from "./pages/Dashboard";
import { Gatekeeper } from "./pages/Gatekeeper";

import { ConfirmModals } from "./components/modals/ConfirmModals";

import { resetApp, lockSession } from "./services";
import { releaseApp } from "./services/system.service";

function App() {
  const [view, setView] = useState<View | null>(null);
  const [isUpdatingFromSettings, setIsUpdatingFromSettings] = useState(false);
  const [showResetConfirm, setShowResetConfirm] = useState(false);
  const [showResetFinal, setShowResetFinal] = useState(false);
  const [isLaunching] = useState(false);
  const [search, setSearch] = useState("");

  const {
    password,
    setPassword,
    confirmPassword,
    setConfirmPassword,
    gatekeeperPIN,
    setGatekeeperPIN,
    error,
    setError,
    isCompleting,
    completingStep,
    handleSetup,
    handleUnlock,
    handleGatekeeperUnlock,
  } = useAuth();

  const { config, setConfig, authMode, setAuthMode, updateConfig } =
    useConfig(setError);

  const { toast, showUpdateSuccess, setShowUpdateSuccess, triggerToast } =
    useToast();

  const {
    lockedApps,
    setLockedApps,
    allApps,
    isScanning,
    fetchDetailedApps,
    appToRemove,
    setAppToRemove,
    appsToBulkUnlock,
    setAppsToBulkUnlock,
    toggleApp,
    confirmRemoval,
    bulkUnlock,
    confirmBulkUnlock,
  } = useApps(triggerToast, setError);

  const { placeholder } = usePlaceholder();

  const { blockedApp, activeTab, setActiveTab, settingsTab, setSettingsTab } =
    useAppInit({
      setConfig,
      setAuthMode,
      setLockedApps,
      fetchDetailedApps,
      setGatekeeperPIN,
      setError,
      onSetView: (v) => setView(v),
    });

  const pinInputRef = useRef<HTMLInputElement>(null);
  const confirmInputRef = useRef<HTMLInputElement>(null);
  const mainInputRef = useRef<HTMLInputElement>(null);
  const gatekeeperInputRef = useRef<HTMLInputElement>(null);

  useFocusGuard({
    view,
    authMode,
    passwordLength: password.length,
    pinInputRef,
    confirmInputRef,
    mainInputRef,
    gatekeeperInputRef,
  });

  useEffect(() => {
    if (
      view &&
      view !== "onboarding" &&
      view !== "unlock" &&
      view !== "gatekeeper"
    ) {
      localStorage.setItem(STORAGE_KEYS.VIEW, view);
    }
    if (activeTab) localStorage.setItem(STORAGE_KEYS.TAB, activeTab);
    if (settingsTab)
      localStorage.setItem(STORAGE_KEYS.SETTINGS_TAB, settingsTab);
  }, [view, activeTab, settingsTab]);

  if (view === null) return null;

  return (
    <div
      className={clsx(
        styles.container,
        view === "gatekeeper" && styles.transparentBg
      )}
    >
      <AnimatePresence>
        {view === "gatekeeper" && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className={styles.gatekeeperOverlay}
          />
        )}
      </AnimatePresence>

      <AnimatePresence mode="wait">
        {view === "onboarding" && (
          <Onboarding appName={APP_NAME} onContinue={() => setView("setup")} />
        )}

        {view === "setup" && (
          <Setup
            authMode={authMode}
            password={password}
            confirmPassword={confirmPassword}
            error={error}
            isCompleting={isCompleting}
            completingStep={completingStep}
            allAppsCount={allApps.length}
            pinInputRef={pinInputRef}
            confirmInputRef={confirmInputRef}
            setAuthMode={setAuthMode}
            setPassword={setPassword}
            setConfirmPassword={setConfirmPassword}
            setError={setError}
            setView={setView}
            handleSetup={(e) =>
              handleSetup(e, authMode, {
                isUpdating: isUpdatingFromSettings,
                setView,
                setIsUpdatingFromSettings,
                setShowUpdateSuccess,
              })
            }
          />
        )}

        {(view === "unlock" || view === "verify") && (
          <Unlock
            appName={APP_NAME}
            authMode={authMode}
            password={password}
            error={error}
            isVerify={view === "verify"}
            mainInputRef={mainInputRef}
            setPassword={setPassword}
            setError={setError}
            handleUnlock={(e, override) =>
              handleUnlock(e, view, setView, override)
            }
            onCancel={() => {
              setView("dashboard");
              setPassword("");
              setError(null);
            }}
          />
        )}

        {view === "dashboard" && (
          <Dashboard
            appName={APP_NAME}
            activeTab={activeTab}
            setActiveTab={setActiveTab}
            showUpdateSuccess={showUpdateSuccess}
            toast={toast}
            search={search}
            setSearch={setSearch}
            placeholder={placeholder}
            handleLockSession={async () => {
              await lockSession();
              setView("unlock");
            }}
            isScanning={isScanning}
            allApps={allApps}
            lockedApps={lockedApps}
            toggleApp={toggleApp}
            refreshApps={fetchDetailedApps}
            bulkUnlock={bulkUnlock}
            settingsTab={settingsTab}
            setSettingsTab={setSettingsTab}
            authMode={authMode}
            setAuthMode={setAuthMode}
            setView={(v: string) => {
              if (v === "setup") {
                setIsUpdatingFromSettings(true);
                setView("verify");
              } else {
                setView(v as View);
              }
            }}
            setIsUpdatingFromSettings={setIsUpdatingFromSettings}
            config={config}
            updateConfig={updateConfig}
            setShowResetConfirm={setShowResetConfirm}
          />
        )}

        {view === "gatekeeper" && (
          <Gatekeeper
            blockedApp={blockedApp}
            authMode={authMode}
            gatekeeperPIN={gatekeeperPIN}
            error={error}
            isLaunching={isLaunching}
            gatekeeperInputRef={gatekeeperInputRef}
            setGatekeeperPIN={setGatekeeperPIN}
            setError={setError}
            config={config}
            handleGatekeeperUnlock={(e, override) =>
              handleGatekeeperUnlock(e, blockedApp, setConfig, override)
            }
            closeWindow={async () => {
              await releaseApp();
            }}
          />
        )}
      </AnimatePresence>

      <ConfirmModals
        appName={APP_NAME}
        appToRemove={appToRemove}
        setAppToRemove={setAppToRemove}
        onConfirmRemoval={confirmRemoval}
        appsToBulkUnlock={appsToBulkUnlock}
        setAppsToBulkUnlock={setAppsToBulkUnlock}
        onConfirmBulkUnlock={confirmBulkUnlock}
        showResetConfirm={showResetConfirm}
        setShowResetConfirm={setShowResetConfirm}
        showResetFinal={showResetFinal}
        setShowResetFinal={setShowResetFinal}
        onReset={async () => {
          await resetApp();
          window.location.reload();
        }}
      />
    </div>
  );
}

export default App;
