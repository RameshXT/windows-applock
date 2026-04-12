import { useState, useEffect, Dispatch, SetStateAction } from "react";
import { View } from "../types";

interface UseToastResult {
  toast: {
    message: string;
    visible: boolean;
    type: "lock" | "unlock" | "success";
  };
  showUpdateSuccess: boolean;
  setShowUpdateSuccess: Dispatch<SetStateAction<boolean>>;
  triggerToast: (message: string, type?: "lock" | "unlock" | "success") => void;
}

export function useToast(): UseToastResult {
  const [toast, setToast] = useState<{
    message: string;
    visible: boolean;
    type: "lock" | "unlock" | "success";
  }>({
    message: "",
    visible: false,
    type: "success",
  });
  const [showUpdateSuccess, setShowUpdateSuccess] = useState(false);

  const triggerToast = (
    message: string,
    type: "lock" | "unlock" | "success" = "success"
  ) => {
    setToast({ message, visible: true, type });
    setTimeout(
      () => setToast({ message: "", visible: false, type: "success" }),
      3000
    );
  };

  return { toast, showUpdateSuccess, setShowUpdateSuccess, triggerToast };
}

interface UseFocusGuardProps {
  view: View | null;
  authMode: string;
  passwordLength: number;
  pinInputRef: React.RefObject<HTMLInputElement | null>;
  confirmInputRef: React.RefObject<HTMLInputElement | null>;
  mainInputRef: React.RefObject<HTMLInputElement | null>;
  gatekeeperInputRef: React.RefObject<HTMLInputElement | null>;
}

export function useFocusGuard({
  view,
  authMode,
  passwordLength,
  pinInputRef,
  confirmInputRef,
  mainInputRef,
  gatekeeperInputRef,
}: UseFocusGuardProps) {
  useEffect(() => {
    const handleFocus = () => {
      let target: HTMLInputElement | null = null;
      if (view === "unlock" || view === "verify") target = mainInputRef.current;
      else if (view === "gatekeeper") target = gatekeeperInputRef.current;
      else if (view === "setup") {
        if (authMode === "PIN")
          target =
            passwordLength < 4 ? pinInputRef.current : confirmInputRef.current;
        else if (
          document.activeElement !== pinInputRef.current &&
          document.activeElement !== confirmInputRef.current
        )
          target = pinInputRef.current;
      }
      if (target && document.activeElement !== target) target.focus();
    };

    handleFocus();
    const interval = setInterval(handleFocus, 100);
    window.addEventListener("focus", handleFocus);
    window.addEventListener("click", handleFocus);
    return () => {
      clearInterval(interval);
      window.removeEventListener("focus", handleFocus);
      window.removeEventListener("click", handleFocus);
    };
  }, [view, authMode, passwordLength]);
}
