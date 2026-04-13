import React, { useState } from 'react';
import { Copy, Download, Eye, CheckCircle } from 'lucide-react';

interface RecoveryKeyDisplayProps {
  recoveryKey: string;
  onConfirm: () => void;
}

export const RecoveryKeyDisplay: React.FC<RecoveryKeyDisplayProps> = ({ recoveryKey, onConfirm }) => {
  const [revealed, setRevealed] = useState(false);
  const [confirmed, setConfirmed] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(recoveryKey);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleDownload = () => {
    const element = document.createElement("a");
    const file = new Blob([`AppLock Recovery Key: ${recoveryKey}\nKeep this safe!`], {type: 'text/plain'});
    element.href = URL.createObjectURL(file);
    element.download = "recovery-key.txt";
    document.body.appendChild(element);
    element.click();
  };

  return (
    <div className="flex flex-col gap-6 max-w-md w-full bg-[#111] p-8 rounded-2xl border border-white/10 shadow-2xl">
      <div className="text-center">
        <h2 className="text-2xl font-bold text-white mb-2">Save Recovery Key</h2>
        <p className="text-white/60 text-sm">
          If you forget your master password or get locked out, this key is the ONLY way to regain access.
        </p>
      </div>

      <div className="relative group">
        <div className={`
          p-6 bg-white/5 rounded-xl border border-white/5 font-mono text-xl tracking-[0.2em] text-center transition-all duration-500
          ${!revealed ? 'blur-md select-none opacity-50' : 'blur-0 opacity-100'}
        `}>
          {recoveryKey}
        </div>
        
        {!revealed && (
          <button 
            onClick={() => setRevealed(true)}
            className="absolute inset-0 flex items-center justify-center bg-black/40 rounded-xl hover:bg-black/20 transition-all group-hover:scale-105"
          >
            <div className="flex items-center gap-2 px-4 py-2 bg-white/10 rounded-full border border-white/20 backdrop-blur-md text-white text-sm font-medium">
              <Eye size={16} />
              Reveal Recovery Key
            </div>
          </button>
        )}
      </div>

      <div className="flex gap-4">
        <button 
          onClick={handleCopy}
          className="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-white/5 hover:bg-white/10 rounded-xl border border-white/5 text-white/80 transition-all"
        >
          {copied ? <CheckCircle size={18} className="text-green-500" /> : <Copy size={18} />}
          {copied ? 'Copied' : 'Copy'}
        </button>
        <button 
          onClick={handleDownload}
          className="flex-1 flex items-center justify-center gap-2 px-4 py-3 bg-white/5 hover:bg-white/10 rounded-xl border border-white/5 text-white/80 transition-all"
        >
          <Download size={18} />
          Download
        </button>
      </div>

      <div className="flex items-start gap-3 p-4 bg-orange-500/10 rounded-xl border border-orange-500/20">
        <input 
          type="checkbox" 
          id="confirm-save"
          checked={confirmed}
          onChange={(e) => setConfirmed(e.target.checked)}
          className="mt-1"
        />
        <label htmlFor="confirm-save" className="text-sm text-orange-200/80 leading-snug">
          I have safely stored my recovery key. I understand that if I lose it, my data cannot be recovered.
        </label>
      </div>

      <button
        disabled={!confirmed}
        onClick={onConfirm}
        className={`
          w-full py-4 rounded-xl font-bold transition-all
          ${confirmed 
            ? 'bg-white text-black hover:scale-[1.02] active:scale-[0.98]' 
            : 'bg-white/10 text-white/30 cursor-not-allowed'}
        `}
      >
        Proceed
      </button>
    </div>
  );
};
