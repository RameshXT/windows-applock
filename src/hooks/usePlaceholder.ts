import { useState, useEffect, useRef } from "react";

const SENSITIVE_APPS = [
  "WhatsApp",
  "Slack",
  "Teams",
  "Outlook",
  "Chrome",
  "AnyDesk",
];

interface UsePlaceholderResult {
  placeholder: string;
}

export function usePlaceholder(): UsePlaceholderResult {
  const [placeholder, setPlaceholder] = useState("");
  const [appIndex, setAppIndex] = useState(0);
  const [isDeleting, setIsDeleting] = useState(false);
  const [charIndex, setCharIndex] = useState(0);

  const appIndexRef = useRef(appIndex);
  appIndexRef.current = appIndex;

  useEffect(() => {
    const typingSpeed = isDeleting ? 40 : 100;
    const currentApp = SENSITIVE_APPS[appIndex];
    const timeout = setTimeout(() => {
      if (!isDeleting && charIndex < currentApp.length) {
        setPlaceholder(currentApp.substring(0, charIndex + 1));
        setCharIndex(charIndex + 1);
      } else if (isDeleting && charIndex > 0) {
        setPlaceholder(currentApp.substring(0, charIndex - 1));
        setCharIndex(charIndex - 1);
      } else if (!isDeleting && charIndex === currentApp.length) {
        setTimeout(() => setIsDeleting(true), 2000);
      } else if (isDeleting && charIndex === 0) {
        setIsDeleting(false);
        setAppIndex((appIndexRef.current + 1) % SENSITIVE_APPS.length);
      }
    }, typingSpeed);
    return () => clearTimeout(timeout);
  }, [charIndex, isDeleting, appIndex]);

  return { placeholder };
}
