
import React, { useState, useEffect } from "react";
import styles from "../../styles/App.module.css";
import { RefreshCw, AlertCircle, Rocket, ArrowUpCircle } from "lucide-react";
import { GithubIcon as GitHubIcon } from "../GithubIcon";
import { check, Update } from "@tauri-apps/plugin-updater";
import { getVersion } from "@tauri-apps/api/app";
import { motion } from "framer-motion";

interface UpdatesPageProps {
  appName: string;
}


export const UpdatesSection: React.FC<UpdatesPageProps> = ({ appName }) => {
  const [currentVersion, setCurrentVersion] = useState<string>("");
  const [latestFromGithub, setLatestFromGithub] = useState<string | null>(null);
  const [update, setUpdate] = useState<Update | null>(null);
  const [status, setStatus] = useState<"idle" | "checking" | "available" | "uptodate" | "downloading" | "error">("idle");
  const [error, setError] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState(0);

  useEffect(() => {
    getVersion().then(setCurrentVersion);
    fetchLatestFromGithub();
  }, []);

  const fetchLatestFromGithub = async () => {
    try {
      const response = await fetch("https://api.github.com/repos/RameshXT/AppLock/releases/latest");
      if (response.ok) {
        const data = await response.json();
        setLatestFromGithub(data.tag_name.replace('v', ''));
      }
    } catch (e) {
      console.error("GitHub API Error:", e);
    }
  };

  const checkForUpdates = async () => {
    try {
      setStatus("checking");
      setError(null);
      const updateResult = await check();
      if (updateResult) {
        setUpdate(updateResult);
        setStatus("available");
      } else {
        setUpdate(null);
        setStatus("uptodate");
      }
    } catch (err: any) {
      setError("Update check failed. Check your internet connection.");
      setStatus("error");
    }
  };

  const installUpdate = async () => {
    if (!update) return;
    try {
      setStatus("downloading");
      await update.downloadAndInstall((event) => {
        if (event.event === 'Progress' && event.data.chunkLength) {
           setDownloadProgress(prev => Math.min(prev + 2, 99));
        }
      });
    } catch (err: any) {
      setError(err.toString());
      setStatus("error");
    }
  };

  const renderStatus = () => {
    switch (status) {
      case "checking": return "Checking for updates...";
      case "available": return `Update v${update?.version} is available.`;
      case "uptodate": return "Your software is up to date.";
      case "downloading": return "Downloading & Installing...";
      case "error": return "Something went wrong.";
      default: return `Running v${currentVersion}`;
    }
  };

  const StatusIcon = () => {
    if (status === "checking" || status === "downloading") return <RefreshCw size={28} className={styles.minimalSpinning} />;
    if (status === "available") return <Rocket size={28} />;
    if (status === "error") return <AlertCircle size={28} />;
    return <ArrowUpCircle size={28} />;
  };

  return (
    <div className={styles.minimalUpdateContainer}>
      <motion.div 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className={styles.minimalUpdateCard}
      >
        <div className={styles.minimalUpdateIcon}>
          <div className={styles.minimalUpdateIconGlow} />
          <StatusIcon />
        </div>

        <div className={styles.minimalUpdateInfo}>
          <h2>{status === 'uptodate' ? 'System Updated' : 'Software Update'}</h2>
          <p>{renderStatus()}</p>
          <div className={styles.minimalVersionBadge}>v{currentVersion}</div>
        </div>

        <div className={styles.minimalUpdateAction}>
          {status === "downloading" ? (
            <div className={styles.minimalUpdateProgress}>
                <div className={styles.minimalProgressBar}>
                    <motion.div className={styles.minimalProgressFill} animate={{ width: `${downloadProgress}%` }} />
                </div>
                <div className={styles.minimalStatusText}>
                    <span>Installing</span>
                    <span>{downloadProgress}%</span>
                </div>
            </div>
          ) : status === "available" && update ? (
            <button className={styles.minimalPrimaryBtn} onClick={installUpdate}>
              Download & Install
            </button>
          ) : (
            <button className={styles.minimalSecondaryBtn} onClick={checkForUpdates} disabled={status === "checking"}>
              {status === "checking" ? "Verifying..." : "Check for Updates"}
            </button>
          )}
        </div>

        {error && (
            <div className={styles.minimalError}>
                <AlertCircle size={16} />
                <span>{error}</span>
            </div>
        )}

        <div className={styles.minimalChannelInfo}>
          Channel: Stable Release
        </div>
      </motion.div>
    </div>
  );
};
