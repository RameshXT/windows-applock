import React, { useState, useEffect } from 'react';
import { ShieldAlert, RefreshCw } from 'lucide-react';
import { recoveryService, RecoveryResult } from '../../services/recoveryService';

interface RecoveryKeyInputProps {
  appId: string;
  onSuccess: () => void;
  onCancel?: () => void;
}

export const RecoveryKeyInput: React.FC<RecoveryKeyInputProps> = ({ appId, onSuccess, onCancel }) => {
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [lockoutSecs, setLockoutSecs] = useState<number>(0);

  const formatKey = (val: string) => {
    const alphanumeric = val.replace(/[^a-zA-Z0-9]/g, '').toUpperCase();
    const groups = alphanumeric.match(/.{1,4}/g) || [];
    return groups.slice(0, 5).join('-');
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const formatted = formatKey(e.target.value);
    setInput(formatted);
    setError(null);
  };

  const handleVerify = async () => {
    if (input.length < 24) return;
    
    setLoading(true);
    setError(null);
    try {
      const res = await recoveryService.verifyRecoveryKey(input, appId);
      if (res.success) {
        onSuccess();
      } else {
        setError(res.failure_reason);
        if (res.failure_reason?.includes('1 hour lockout')) {
            setLockoutSecs(3600);
        }
      }
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (lockoutSecs > 0) {
      const timer = setInterval(() => {
        setLockoutSecs(s => s - 1);
      }, 1000);
      return () => clearInterval(timer);
    }
  }, [lockoutSecs]);

  const formatTime = (seconds: number) => {
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  };

  return (
    <div className="flex flex-col gap-6 w-full max-w-sm bg-[#111] p-8 rounded-2xl border border-white/10 shadow-2xl animate-in fade-in zoom-in duration-300">
      <div className="flex flex-col items-center text-center">
        <div className="w-12 h-12 bg-white/5 rounded-full flex items-center justify-center mb-4">
          <ShieldAlert className="text-white/60" size={24} />
        </div>
        <h3 className="text-xl font-bold text-white">Recovery Access</h3>
        <p className="text-white/40 text-sm mt-1">Enter your 20-character recovery key</p>
      </div>

      <div className="relative">
        <input 
          type="text"
          value={input}
          onChange={handleChange}
          disabled={loading || lockoutSecs > 0}
          placeholder="XXXX-XXXX-XXXX-XXXX-XXXX"
          className={`
            w-full bg-white/5 border rounded-xl p-4 text-center font-mono text-lg tracking-widest text-white transition-all
            focus:outline-none focus:ring-2 placeholder:text-white/10
            ${error ? 'border-red-500/50 bg-red-500/5 focus:ring-red-500/20' : 'border-white/10 focus:ring-white/20'}
            ${lockoutSecs > 0 ? 'opacity-50 grayscale' : ''}
          `}
        />
        {loading && (
          <div className="absolute right-4 top-1/2 -translate-y-1/2">
            <RefreshCw size={18} className="text-white/40 animate-spin" />
          </div>
        )}
      </div>

      {error && !lockoutSecs && (
        <div className="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-red-400 text-xs text-center animate-in slide-in-from-top-2">
          {error}
        </div>
      )}

      {lockoutSecs > 0 && (
        <div className="flex flex-col items-center gap-2">
           <div className="text-red-400 font-bold text-xl">{formatTime(lockoutSecs)}</div>
           <p className="text-white/30 text-xs">Recovery attempts exhausted</p>
        </div>
      )}

      <div className="flex flex-col gap-3">
        <button
          onClick={handleVerify}
          disabled={loading || input.length < 24 || lockoutSecs > 0}
          className="w-full py-4 bg-white text-black rounded-xl font-bold hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-30 disabled:hover:scale-100"
        >
          Verify & Restore
        </button>
        {onCancel && (
          <button
            onClick={onCancel}
            className="w-full py-3 text-white/40 text-sm hover:text-white/60 transition-colors"
          >
            Cancel
          </button>
        )}
      </div>
    </div>
  );
};
