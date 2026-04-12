import { AnimatePresence, motion } from "framer-motion";
import { AlertCircle, AlertTriangle } from "lucide-react";
import styles from "../../styles/App.module.css";
import { LockedApp, InstalledApp } from "../../types";

interface ConfirmModalsProps {
  appName: string;
  appToRemove: LockedApp | InstalledApp | null;
  setAppToRemove: (val: null) => void;
  onConfirmRemoval: () => void;
  appsToBulkUnlock: LockedApp[] | null;
  setAppsToBulkUnlock: (val: null) => void;
  onConfirmBulkUnlock: () => void;
  showResetConfirm: boolean;
  setShowResetConfirm: (val: boolean) => void;
  showResetFinal: boolean;
  setShowResetFinal: (val: boolean) => void;
  onReset: () => void;
}

export function ConfirmModals({
  appName,
  appToRemove,
  setAppToRemove,
  onConfirmRemoval,
  appsToBulkUnlock,
  setAppsToBulkUnlock,
  onConfirmBulkUnlock,
  showResetConfirm,
  setShowResetConfirm,
  showResetFinal,
  setShowResetFinal,
  onReset,
}: ConfirmModalsProps) {
  return (
    <AnimatePresence>
      {appToRemove && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className={styles.modalOverlay}
        >
          <motion.div
            initial={{ scale: 0.95 }}
            animate={{ scale: 1 }}
            className={styles.modalCard}
          >
            <div className={styles.modalIcon}>
              <AlertCircle size={40} color="var(--error-color)" />
            </div>
            <h3>Remove Protection?</h3>
            <p>
              Are you sure you want to unlock{" "}
              <strong>{appToRemove.name}</strong>? This application will no
              longer be protected by {appName}.
            </p>
            <div className={styles.modalActions}>
              <button
                className={styles.modalCancel}
                onClick={() => setAppToRemove(null)}
              >
                Cancel
              </button>
              <button
                className={styles.modalConfirm}
                onClick={onConfirmRemoval}
              >
                Remove Lock
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}

      {appsToBulkUnlock && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className={styles.modalOverlay}
        >
          <motion.div
            initial={{ scale: 0.95 }}
            animate={{ scale: 1 }}
            className={styles.modalCard}
          >
            <div className={styles.modalIcon}>
              <AlertCircle size={40} color="var(--error-color)" />
            </div>
            <h3>Unlock Multiple?</h3>
            <p>
              Are you sure you want to remove protection from{" "}
              <strong>{appsToBulkUnlock.length}</strong> applications?
            </p>
            <div className={styles.modalActions}>
              <button
                className={styles.modalCancel}
                onClick={() => setAppsToBulkUnlock(null)}
              >
                Cancel
              </button>
              <button
                className={styles.modalConfirm}
                onClick={onConfirmBulkUnlock}
              >
                Remove Locks
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}

      {showResetConfirm && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className={styles.modalOverlay}
        >
          <motion.div
            initial={{ scale: 0.95 }}
            animate={{ scale: 1 }}
            className={styles.modalCard}
          >
            <div className={styles.modalIcon}>
              <AlertTriangle size={40} color="#f59e0b" />
            </div>
            <h3>Wipe All Data?</h3>
            <p>
              Are you sure you want to reset {appName}? This will remove all
              your protected apps and security settings.
            </p>
            <div className={styles.modalActions}>
              <button
                className={styles.modalCancel}
                onClick={() => setShowResetConfirm(false)}
              >
                Cancel
              </button>
              <button
                className={styles.modalConfirm}
                style={{ background: "#f59e0b" }}
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
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className={styles.modalOverlay}
        >
          <motion.div
            initial={{ scale: 0.95 }}
            animate={{ scale: 1 }}
            className={styles.modalCard}
          >
            <div className={styles.modalIcon}>
              <AlertCircle size={40} color="var(--error-color)" />
            </div>
            <h3>Final Warning</h3>
            <p>
              This action is <strong>irreversible</strong>. All your
              configurations will be permanently deleted and cannot be
              recovered.
            </p>
            <div className={styles.modalActions}>
              <button
                className={styles.modalCancel}
                onClick={() => setShowResetFinal(false)}
              >
                Abort
              </button>
              <button className={styles.modalConfirm} onClick={onReset}>
                Reset Everything
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
