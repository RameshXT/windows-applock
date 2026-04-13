import React, { useState } from 'react';
import { Trash2, ShieldCheck, Key, AlertTriangle, ArrowRight, Check } from 'lucide-react';
import { recoveryService, ResetResult } from '../../services/recoveryService';

export const FullResetFlow: React.FC = () => {
  const [step, setStep] = useState<1 | 2 | 3>(1);
  const [method, setMethod] = useState<'credential' | 'recovery_key'>('credential');
  const [input, setInput] = useState('');
  const [token, setToken] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmed, setConfirmed] = useState(false);
  const [resetResult, setResetResult] = useState<ResetResult | null>(null);

  const handleVerify = async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await recoveryService.initiateFullReset(method, input);
      if (res.verified && res.reset_token) {
        setToken(res.reset_token);
        setStep(2);
      } else {
        setError(res.failure_reason || "Verification failed");
      }
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const handleReset = async () => {
    if (!token) return;
    setLoading(true);
    try {
      const res = await recoveryService.performFullReset(token);
      setResetResult(res);
      setStep(3);
      // Wait a bit then reload or redirect
      setTimeout(() => {
        window.location.reload();
      }, 3000);
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-md w-full animate-in fade-in slide-in-from-bottom-4 duration-500">
      {step === 1 && (
        <div className="bg-[#111] border border-white/10 rounded-2xl p-8 shadow-2xl">
          <div className="flex items-center gap-3 mb-6">
            <div className="p-3 bg-red-500/10 rounded-xl text-red-500">
              <Trash2 size={24} />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">Full Factory Reset</h2>
              <p className="text-white/40 text-sm">Wipe all data and settings</p>
            </div>
          </div>

          <div className="space-y-4 mb-8">
            <p className="text-sm text-white/60">Choose verification method:</p>
            <div className="grid grid-cols-2 gap-3">
              <button
                onClick={() => setMethod('credential')}
                className={`flex flex-col items-center gap-2 p-4 rounded-xl border transition-all ${method === 'credential' ? 'bg-white/10 border-white/20 text-white' : 'bg-transparent border-white/5 text-white/40 hover:border-white/10'}`}
              >
                <ShieldCheck size={20} />
                <span className="text-xs font-medium">Credential</span>
              </button>
              <button
                onClick={() => setMethod('recovery_key')}
                className={`flex flex-col items-center gap-2 p-4 rounded-xl border transition-all ${method === 'recovery_key' ? 'bg-white/10 border-white/20 text-white' : 'bg-transparent border-white/5 text-white/40 hover:border-white/10'}`}
              >
                <Key size={20} />
                <span className="text-xs font-medium">Recovery Key</span>
              </button>
            </div>
          </div>

          <div className="space-y-6">
            <div className="space-y-2">
              <label className="text-xs text-white/40 font-medium uppercase tracking-wider">
                {method === 'credential' ? 'Enter Password / PIN' : 'Enter Recovery Key'}
              </label>
              <input
                type={method === 'credential' ? 'password' : 'text'}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder={method === 'credential' ? '••••••••' : 'XXXX-XXXX-XXXX-XXXX-XXXX'}
                className="w-full bg-white/5 border border-white/10 rounded-xl p-4 text-white placeholder:text-white/10 focus:outline-none focus:border-white/20"
              />
            </div>

            {error && <p className="text-red-400 text-xs text-center">{error}</p>}

            <button
              onClick={handleVerify}
              disabled={loading || !input}
              className="w-full py-4 bg-white text-black rounded-xl font-bold flex items-center justify-center gap-2 hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-50"
            >
              Verify Identity
              <ArrowRight size={18} />
            </button>
          </div>
        </div>
      )}

      {step === 2 && (
        <div className="bg-[#111] border border-red-500/20 rounded-2xl p-8 shadow-2xl overflow-hidden relative">
          <div className="absolute top-0 left-0 w-full h-1 bg-red-500/20">
             <div className="h-full bg-red-500 animate-progress origin-left" style={{animationDuration: '60s'}}></div>
          </div>
          
          <div className="flex flex-col items-center text-center gap-6">
            <div className="w-16 h-16 bg-red-500/10 rounded-full flex items-center justify-center text-red-500 animate-pulse">
              <AlertTriangle size={32} />
            </div>
            
            <div className="space-y-2">
              <h2 className="text-2xl font-black text-white uppercase tracking-tight">Are you absolutely sure?</h2>
              <p className="text-white/60 leading-relaxed">
                This will <span className="text-white font-bold">permanently delete</span> all your credentials, locked application history, settings, and logs. This action <span className="text-red-400 underline decoration-red-500/30">cannot</span> be undone.
              </p>
            </div>

            <div className="w-full flex items-start gap-4 p-5 bg-white/5 rounded-2xl border border-white/10 text-left cursor-pointer group" onClick={() => setConfirmed(!confirmed)}>
                <div className={`mt-1 w-5 h-5 rounded-md border flex items-center justify-center transition-all ${confirmed ? 'bg-red-500 border-red-500' : 'border-white/20 group-hover:border-white/40'}`}>
                    {confirmed && <Check size={14} className="text-white" />}
                </div>
                <p className="text-sm text-white/50 leading-snug group-hover:text-white/70 transition-colors">
                    I understand that performing a full reset will wipe all data and I will be redirected to the initial setup.
                </p>
            </div>

            <div className="w-full grid grid-cols-2 gap-4">
               <button 
                onClick={() => setStep(1)}
                className="py-4 rounded-xl font-bold bg-white/5 text-white/40 hover:bg-white/10 transition-all"
               >
                Nevermind
               </button>
               <button 
                onClick={handleReset}
                disabled={!confirmed || loading}
                className="py-4 rounded-xl font-bold bg-red-500 text-white hover:bg-red-600 hover:scale-[1.02] active:scale-[0.98] transition-all disabled:opacity-20 shadow-[0_0_30px_-5px_rgba(239,68,68,0.4)]"
               >
                {loading ? 'Wiping...' : 'Destroy All Data'}
               </button>
            </div>
          </div>
        </div>
      )}

      {step === 3 && resetResult && (
        <div className="bg-[#111] border border-green-500/20 rounded-2xl p-12 shadow-2xl text-center">
            <div className="w-20 h-20 bg-green-500/10 rounded-full flex items-center justify-center text-green-500 mx-auto mb-6">
                <Check size={40} />
            </div>
            <h2 className="text-2xl font-bold text-white mb-2">System Reset Complete</h2>
            <p className="text-white/40 mb-8">
                {resetResult.files_deleted.length} protected volumes purged.
                <br />
                Cleaning up remaining artifacts...
            </p>
            <div className="flex items-center justify-center gap-2 text-white/20 text-xs font-mono">
                <RefreshCw size={14} className="animate-spin" />
                REBOOTING_ONBOARDING_CORE
            </div>
        </div>
      )}
    </div>
  );
};

const RefreshCw = ({ size, className }: any) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.85.83 6.72 2.24L21 8" />
    <path d="M21 3v5h-5" />
  </svg>
);
