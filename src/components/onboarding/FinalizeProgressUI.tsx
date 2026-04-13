import React, { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Check, X, Loader2, AlertCircle, RotateCcw } from "lucide-react";
import styles from "../../styles/Onboarding.module.css";
import { 
  onboardingFinalizerService, 
  OnboardingPayload, 
  OnboardingStepProgress 
} from "../../services/onboardingFinalizerService";

export interface FinalizeProgressUIProps {
  payload: OnboardingPayload;
  onSuccess: () => void;
  onCancel: () => void;
}

interface StepState {
  id: string;
  label: string;
  status: "pending" | "in_progress" | "done" | "failed";
}

const STEPS: StepState[] = [
  { id: "Securing credential", label: "Securing credential", status: "pending" },
  { id: "Saving apps", label: "Saving apps", status: "pending" },
  { id: "Saving settings", label: "Saving settings", status: "pending" },
  { id: "Registering autostart", label: "Registering autostart", status: "pending" },
  { id: "Finalizing", label: "Finalizing", status: "pending" },
];

export const FinalizeProgressUI: React.FC<FinalizeProgressUIProps> = ({ payload, onSuccess, onCancel }) => {
  const [steps, setSteps] = useState<StepState[]>(STEPS);
  const [error, setError] = useState<{ step: string; reason: string; rollback_ok: boolean } | null>(null);
  const [isFinalizing, setIsFinalizing] = useState(false);

  useEffect(() => {
    let unlistenProgress: any;
    let unlistenComplete: any;
    let unlistenFailure: any;

    const setupListeners = async () => {
      unlistenProgress = await onboardingFinalizerService.onProgress((p: OnboardingStepProgress) => {
        setSteps(prev => prev.map(s => s.id === p.step ? { ...s, status: p.status } : s));
      });

      unlistenComplete = await onboardingFinalizerService.onComplete(() => {
        setIsFinalizing(false);
        setTimeout(onSuccess, 800);
      });

      unlistenFailure = await onboardingFinalizerService.onFailure((f) => {
        setError(f);
        setSteps(prev => prev.map(s => s.id === f.step ? { ...s, status: "failed" } : s));
        setIsFinalizing(false);
      });
    };

    setupListeners();
    startFinalization();

    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenComplete) unlistenComplete();
      if (unlistenFailure) unlistenFailure();
    };
  }, []);

  const startFinalization = async () => {
    if (isFinalizing) return;
    setError(null);
    setSteps(STEPS.map(s => ({ ...s, status: "pending" })));
    setIsFinalizing(true);
    try {
      await onboardingFinalizerService.finalize(payload);
    } catch (e: any) {
      console.error("Invocation error:", e);
      setError({ step: "System", reason: "Failed to communicate with the secure engine.", rollback_ok: true });
      setIsFinalizing(false);
    }
  };

  return (
    <div className={styles.finalizeContainer}>
      <motion.div 
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className={styles.finalizeCard}
      >
        <h2 className={styles.finalizeTitle}>Finalizing Setup</h2>
        <p className={styles.finalizeSubtitle}>Applying security protocols and persisting configurations.</p>

        <div className={styles.stepList}>
          {steps.map((step) => (
            <div key={step.id} className={styles.stepItem}>
              <div className={styles.stepIconWrapper}>
                <AnimatePresence mode="wait">
                  {step.status === "pending" && (
                    <motion.div 
                      key="pending" 
                      initial={{ scale: 0 }} 
                      animate={{ scale: 1 }} 
                      exit={{ scale: 0 }}
                      className={styles.stepDot} 
                    />
                  )}
                  {step.status === "in_progress" && (
                    <motion.div 
                      key="spinner" 
                      initial={{ rotate: 0 }} 
                      animate={{ rotate: 360 }} 
                      transition={{ repeat: Infinity, duration: 1, ease: "linear" }}
                      className={styles.stepSpinner}
                    >
                      <Loader2 size={18} />
                    </motion.div>
                  )}
                  {step.status === "done" && (
                    <motion.div 
                      key="check" 
                      initial={{ scale: 0 }} 
                      animate={{ scale: 1 }} 
                      className={styles.stepCheck}
                    >
                      <Check size={18} />
                    </motion.div>
                  )}
                  {step.status === "failed" && (
                    <motion.div 
                      key="fail" 
                      initial={{ scale: 0 }} 
                      animate={{ scale: 1 }} 
                      className={styles.stepFail}
                    >
                      <X size={18} />
                    </motion.div>
                  )}
                </AnimatePresence>
              </div>
              <span className={`${styles.stepLabel} ${step.status === "failed" ? styles.stepLabelFailed : ""}`}>
                {step.label}
              </span>
            </div>
          ))}
        </div>

        {error && (
          <motion.div 
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            className={styles.finalizeError}
          >
            <div className={styles.errorHeader}>
              <AlertCircle size={20} />
              <span>Finalization Failed</span>
            </div>
            <p className={styles.errorReason}>{error.reason}</p>
            <div className={styles.errorFooter}>
               {error.rollback_ok ? "Rollback successful - System clean." : "Rollback incomplete - Manual cleanup suggested."}
            </div>
            <div className={styles.finalizeActions}>
               <button className={styles.retryBtn} onClick={startFinalization}>
                 <RotateCcw size={16} />
                 Try Again
               </button>
               <button className={styles.cancelBtn} onClick={onCancel}>
                 Cancel
               </button>
            </div>
          </motion.div>
        )}
      </motion.div>
    </div>
  );
};
