import { useState } from "react";
import { InstalledApp, LockedApp, Tab } from "../types";
import { getDetailedApps, saveSelection } from "../services/apps.service";

interface UseAppsResult {
  lockedApps: LockedApp[];
  setLockedApps: React.Dispatch<React.SetStateAction<LockedApp[]>>;
  allApps: InstalledApp[];
  isScanning: boolean;
  appToRemove: LockedApp | InstalledApp | null;
  setAppToRemove: React.Dispatch<
    React.SetStateAction<LockedApp | InstalledApp | null>
  >;
  appsToBulkUnlock: LockedApp[] | null;
  setAppsToBulkUnlock: React.Dispatch<React.SetStateAction<LockedApp[] | null>>;
  fetchDetailedApps: () => Promise<void>;
  toggleApp: (app: LockedApp | InstalledApp, fromTab?: Tab) => Promise<void>;
  confirmRemoval: () => Promise<void>;
  bulkUnlock: (apps: LockedApp[]) => void;
  confirmBulkUnlock: () => Promise<void>;
}

export function useApps(
  triggerToast: (message: string, type?: "lock" | "unlock" | "success") => void,
  setError: (err: string | null) => void
): UseAppsResult {
  const [lockedApps, setLockedApps] = useState<LockedApp[]>([]);
  const [allApps, setAllApps] = useState<InstalledApp[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [appToRemove, setAppToRemove] = useState<
    LockedApp | InstalledApp | null
  >(null);
  const [appsToBulkUnlock, setAppsToBulkUnlock] = useState<LockedApp[] | null>(
    null
  );

  const fetchDetailedApps = async () => {
    try {
      setIsScanning(true);
      const apps = await getDetailedApps();
      setAllApps(apps);
    } catch (err) {
      console.error("Failed to fetch apps:", err);
    } finally {
      setIsScanning(false);
    }
  };

  const toggleApp = async (app: LockedApp | InstalledApp, fromTab?: Tab) => {
    let snapshot: LockedApp[] = [];
    setLockedApps((prev) => {
      snapshot = prev;
      return prev;
    });

    const isLocked = snapshot.some((la) => la.name === app.name);

    if (isLocked) {
      if (fromTab === "all") {
        setAppToRemove(app);
        return;
      }
      const newLocked = snapshot.filter((la) => la.name !== app.name);
      setLockedApps(newLocked);
      try {
        await saveSelection(newLocked);
        triggerToast(`${app.name} Unlocked Successfully`, "unlock");
      } catch (err) {
        setError(String(err));
      }
      return;
    }

    const newLocked: LockedApp[] = [
      ...snapshot,
      {
        id: Math.random().toString(36).substring(2, 9),
        name: app.name,
        exec_name:
          (app as LockedApp).exec_name || (app as InstalledApp).path || "",
        icon: app.icon,
      },
    ];
    setLockedApps(newLocked);
    try {
      await saveSelection(newLocked);
      triggerToast(`${app.name} Locked Successfully`, "lock");
    } catch (err) {
      setError(String(err));
    }
  };

  const confirmRemoval = async () => {
    if (!appToRemove) return;
    setLockedApps((prev) => {
      const newLocked = prev.filter((la) => la.name !== appToRemove.name);
      saveSelection(newLocked).catch((err) => setError(String(err)));
      return newLocked;
    });
    const name = appToRemove.name;
    setAppToRemove(null);
    triggerToast(`${name} Unlocked Successfully`, "unlock");
  };

  const bulkUnlock = (apps: LockedApp[]) => setAppsToBulkUnlock(apps);

  const confirmBulkUnlock = async () => {
    if (!appsToBulkUnlock) return;
    const namesToUnlock = new Set(appsToBulkUnlock.map((a) => a.name));
    const count = appsToBulkUnlock.length;
    setLockedApps((prev) => {
      const newLocked = prev.filter((la) => !namesToUnlock.has(la.name));
      saveSelection(newLocked).catch((err) => setError(String(err)));
      return newLocked;
    });
    setAppsToBulkUnlock(null);
    triggerToast(`${count} Apps Unlocked Successfully`, "unlock");
  };

  return {
    lockedApps,
    setLockedApps,
    allApps,
    isScanning,
    fetchDetailedApps,
    appToRemove,
    setAppToRemove,
    appsToBulkUnlock,
    setAppsToBulkUnlock,
    toggleApp,
    confirmRemoval,
    bulkUnlock,
    confirmBulkUnlock,
  };
}
