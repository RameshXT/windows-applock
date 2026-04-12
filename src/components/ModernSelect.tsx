import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronDown } from "lucide-react";
import clsx from "clsx";
import styles from "../styles/App.module.css";

interface Option {
  label: string;
  value: string;
}

interface ModernSelectProps {
  value: string;
  options: Option[];
  onChange: (val: string) => void;
}

export const ModernSelect = ({
  value,
  options,
  onChange,
}: ModernSelectProps) => {
  const [isOpen, setIsOpen] = useState(false);
  const selectedLabel =
    options.find((o) => o.value === value)?.label || "Select...";

  return (
    <div className={styles.selectWrapper}>
      <button
        className={styles.modernSelectBtn}
        onFocus={() => setIsOpen(true)}
        onBlur={() => setTimeout(() => setIsOpen(false), 200)}
      >
        <span>{selectedLabel}</span>
        <motion.div animate={{ rotate: isOpen ? 180 : 0 }}>
          <ChevronDown size={14} className={styles.selectIcon} />
        </motion.div>
      </button>

      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, y: 5 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 5 }}
            className={styles.selectMenu}
          >
            {options.map((opt) => (
              <div
                key={opt.value}
                className={clsx(
                  styles.selectOption,
                  opt.value === value && styles.selectOptionActive
                )}
                onClick={() => {
                  onChange(opt.value);
                  setIsOpen(false);
                }}
              >
                {opt.label}
              </div>
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};
