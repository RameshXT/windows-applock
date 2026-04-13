import React, { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Timer, XCircle, RotateCcw, ShieldAlert } from "lucide-react";
import styles from "../../styles/App.module.css";
import { graceSessionService, GraceSessionView } from "../../services/graceSessionService";

export const GraceSessionPanel: React.FC = () => {
    const [sessions, setSessions] = useState<GraceSessionView[]>([]);
    const [maxSecurity, setMaxSecurity] = useState(false);

    const fetchSessions = async () => {
        try {
            const data = await graceSessionService.getAllGraceSessions();
            setSessions(data);
            const ms = await graceSessionService.getMaxSecurityMode();
            setMaxSecurity(ms);
        } catch (e) {
            console.error("Failed to fetch grace sessions", e);
        }
    };

    useEffect(() => {
        fetchSessions();
        const timer = setInterval(() => {
            setSessions(prev => prev.map(s => ({
                ...s,
                seconds_remaining: Math.max(0, s.seconds_remaining - 1)
            })));
        }, 1000);

        const unlistens: Promise<any>[] = [];
        unlistens.push(graceSessionService.onGraceStarted(fetchSessions));
        unlistens.push(graceSessionService.onGraceExpired(fetchSessions));
        unlistens.push(graceSessionService.onAllGraceSessionsReset(fetchSessions));
        unlistens.push(graceSessionService.onMaxSecurityModeChanged((p) => setMaxSecurity(p.enabled)));

        return () => {
            clearInterval(timer);
            unlistens.forEach(u => u.then(fn => fn()));
        };
    }, []);

    const handleReLock = async (appId: string) => {
        await graceSessionService.reLockApp(appId);
        fetchSessions();
    };

    const handleReLockAll = async () => {
        await graceSessionService.reLockAll();
        fetchSessions();
    };

    if (maxSecurity) {
        return (
            <div className={styles.gracePanelDisabled}>
                <ShieldAlert size={24} className={styles.warningIcon} />
                <div className={styles.warningText}>
                    <strong>Maximum Security Mode Active</strong>
                    <span>Grace periods are disabled. Every access requires verification.</span>
                </div>
            </div>
        );
    }

    if (sessions.length === 0) return null;

    return (
        <section className={styles.gracePanel}>
            <div className={styles.graceHeader}>
                <div className={styles.graceTitle}>
                    <Timer size={18} />
                    <h3>Active Grace Periods</h3>
                    <span className={styles.sessionCountBadge}>{sessions.length}</span>
                </div>
                <button className={styles.reLockAllBtn} onClick={handleReLockAll}>
                    <RotateCcw size={14} /> Re-lock All
                </button>
            </div>

            <div className={styles.sessionList}>
                <AnimatePresence mode="popLayout">
                    {sessions.map((session) => (
                        <motion.div
                            key={session.app_id}
                            layout
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, scale: 0.95 }}
                            className={styles.sessionRow}
                        >
                            <div className={styles.sessionInfo}>
                                <span className={styles.sessionAppName}>{session.app_name}</span>
                                <span className={styles.sessionTimeRemaining}>
                                    {Math.floor(session.seconds_remaining / 60)}:
                                    {(session.seconds_remaining % 60).toString().padStart(2, '0')} remaining
                                </span>
                            </div>

                            <div className={styles.progressContainer}>
                                <motion.div 
                                    className={styles.progressBar}
                                    initial={false}
                                    animate={{ 
                                        width: `${(session.seconds_remaining / session.grace_duration_secs) * 100}%` 
                                    }}
                                    transition={{ duration: 1, ease: "linear" }}
                                />
                            </div>

                            <button 
                                className={styles.reLockSingleBtn} 
                                onClick={() => handleReLock(session.app_id)}
                                title="End grace period"
                            >
                                <XCircle size={16} />
                            </button>
                        </motion.div>
                    ))}
                </AnimatePresence>
            </div>
        </section>
    );
};
