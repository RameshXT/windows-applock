import React, { useState } from 'react';
import { Lock, AlertCircle, Key, Info } from 'lucide-react';
import { RecoveryKeyInput } from './RecoveryKeyInput';

interface HardLockScreenProps {
  appId: string;
  lockedAt: string | null;
  onUnlocked: () => void;
}

export const HardLockScreen: React.FC<HardLockScreenProps> = ({ appId, lockedAt, onUnlocked }) => {
  const [showRecovery, setShowRecovery] = useState(false);

  // Parse and format lockedAt timestamp
  const formatLockedAt = (iso: string | null) => {
    if (!iso) return "recently";
    try {
      const date = new Date(iso);
      return date.toLocaleString(undefined, {
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
      });
    } catch {
      return "recently";
    }
  };

  return (
    <div className="fixed inset-0 z-[9999] bg-black flex items-center justify-center p-6 select-none animate-in fade-in duration-700">
      {/* Background patterns */}
      <div className="absolute inset-0 opacity-[0.03] pointer-events-none">
        <div className="absolute top-0 left-0 w-full h-full bg-[radial-gradient(#fff_1px,transparent_1px)] [background-size:40px_40px]"></div>
      </div>

      {!showRecovery ? (
        <div className="max-w-md w-full flex flex-col items-center text-center gap-8 relative">
          <div className="relative">
            <div className="absolute inset-0 bg-red-500/20 blur-3xl rounded-full scale-150"></div>
            <div className="w-24 h-24 bg-[#111] border-2 border-red-500/30 rounded-3xl flex items-center justify-center text-red-500 shadow-[0_0_50px_-10px_rgba(239,68,68,0.3)] relative group transition-all duration-500 hover:scale-110">
              <Lock size={44} className="animate-pulse" />
            </div>
          </div>

          <div className="space-y-4">
            <div className="space-y-2">
              <h1 className="text-4xl font-black text-white tracking-tight uppercase italic">Hard Locked</h1>
              <div className="flex items-center justify-center gap-2 text-red-400/80 font-medium">
                <AlertCircle size={16} />
                <span>Security Protocol Active</span>
              </div>
            </div>
            
            <p className="text-white/40 leading-relaxed text-sm">
              This {appId === 'dashboard_lock' ? 'application dashboard' : 'protected application'} has been locked after 10 failed access attempts to prevent brute-force attacks.
            </p>
          </div>

          <div className="w-full space-y-3 pt-4">
             <div className="p-4 bg-white/5 rounded-2xl border border-white/10 flex items-center gap-4 text-left">
                <div className="p-2 bg-white/5 rounded-lg text-white/40">
                    <Info size={18} />
                </div>
                <div>
                    <p className="text-[10px] uppercase tracking-widest text-white/30 font-bold">Lock Timestamp</p>
                    <p className="text-sm text-white/70 font-mono">{formatLockedAt(lockedAt)}</p>
                </div>
             </div>

             <button 
              onClick={() => setShowRecovery(true)}
              className="w-full group flex items-center justify-between p-5 bg-white text-black rounded-2xl font-bold hover:bg-neutral-200 transition-all active:scale-95"
             >
                <div className="flex items-center gap-3">
                  <Key size={20} />
                  <span>Use Recovery Key</span>
                </div>
                <div className="w-8 h-8 rounded-full bg-black/5 flex items-center justify-center group-hover:translate-x-1 transition-transform">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12h14m-7-7 7 7-7 7"/></svg>
                </div>
             </button>
          </div>

          <footer className="pt-8 flex flex-col items-center gap-4">
               <p className="text-[10px] text-white/20 uppercase tracking-[0.3em] font-black">Hardware Identity Verified</p>
               <div className="flex gap-2">
                  <div className="w-1 h-1 rounded-full bg-red-500/50"></div>
                  <div className="w-1 h-1 rounded-full bg-red-500/50"></div>
                  <div className="w-1 h-1 rounded-full bg-red-500"></div>
               </div>
          </footer>
        </div>
      ) : (
        <RecoveryKeyInput 
          appId={appId} 
          onSuccess={onUnlocked}
          onCancel={() => setShowRecovery(false)}
        />
      )}
    </div>
  );
};
