import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { open as openUrl } from "@tauri-apps/plugin-shell";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Switch } from "@/components/ui/switch";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import {
  tauriCommands,
  type Config,
  type ModManifest,
  type ModMetadata,
} from "@/hooks/useTauri";
import {
  FolderOpen,
  RefreshCw,
  Download,
  Trash2,
  Upload,
  AlertTriangle,
  History,
  Settings,
  Wrench,
  CheckCircle2,
  XCircle,
} from "lucide-react";
import { FaDiscord } from "react-icons/fa6";
import { SiKofi } from "react-icons/si";
import { ModMetadataDialog } from "@/components/ModMetadataDialog";
import { ConflictsDialog } from "@/components/ConflictsDialog";
import { RestorePointsDialog } from "@/components/RestorePointsDialog";
import { TitleBar } from "@/components/TitleBar";
import { Toaster } from "@/components/ui/sonner";
import { UpdaterCard } from "@/components/UpdaterCard";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface ModWithInfo extends ModManifest {
  id: string;
  enabled: boolean;
}

// Helper function to safely convert unknown errors to strings
const formatError = (error: unknown): string => {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
};

function App() {
  const [config, setConfig] = useState<Config | null>(null);
  const [mods, setMods] = useState<ModWithInfo[]>([]);
  const [selectedMod, setSelectedMod] = useState<ModWithInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const [appVersion, setAppVersion] = useState("");

  // Editable path states
  const [gameTargetInput, setGameTargetInput] = useState("");
  const [userDirInput, setUserDirInput] = useState("");
  const [darkMode, setDarkMode] = useState(false);

  // Dialog states
  const [metadataDialogOpen, setMetadataDialogOpen] = useState(false);
  const [conflictsDialogOpen, setConflictsDialogOpen] = useState(false);
  const [restoreDialogOpen, setRestoreDialogOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [modDetailsOpen, setModDetailsOpen] = useState(false);
  const [pendingImportPath, setPendingImportPath] = useState<string | null>(
    null
  );

  // FM Name Fix states
  const [nameFixInstalled, setNameFixInstalled] = useState(false);
  const [checkingNameFix, setCheckingNameFix] = useState(false);
  const [installingNameFix, setInstallingNameFix] = useState(false);

  const addLog = (message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setLogs((prev) => [`[${timestamp}] ${message}`, ...prev].slice(0, 100));
  };

  const loadConfig = async () => {
    try {
      const cfg = await tauriCommands.getConfig();
      setConfig(cfg);
      setGameTargetInput(cfg.target_path ?? "");
      setUserDirInput(cfg.user_dir_path ?? "");

      // Load dark mode preference
      const shouldUseDarkMode = cfg.dark_mode ?? false;
      setDarkMode(shouldUseDarkMode);
      if (shouldUseDarkMode) {
        document.documentElement.classList.add("dark");
      } else {
        document.documentElement.classList.remove("dark");
      }

      addLog("Configuration loaded");
    } catch (error) {
      addLog(`Error loading config: ${formatError(error)}`);
    }
  };

  const loadMods = async () => {
    try {
      setLoading(true);
      const modNames = await tauriCommands.getModsList();
      const modsWithInfo: ModWithInfo[] = [];

      for (const name of modNames) {
        try {
          const manifest = await tauriCommands.getModDetails(name);
          modsWithInfo.push({
            ...manifest,
            id: name,
            enabled: config?.enabled_mods?.includes(name) ?? false,
          });
        } catch (error) {
          addLog(`Failed to load mod ${name}: ${formatError(error)}`);
        }
      }

      setMods(modsWithInfo);
      addLog(`Loaded ${modsWithInfo.length} mods`);
    } catch (error) {
      addLog(`Error loading mods: ${formatError(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const detectGamePath = async () => {
    try {
      setLoading(true);
      addLog("Detecting game path...");
      const paths = await tauriCommands.detectGamePath();

      if (paths.length > 0) {
        await tauriCommands.setGameTarget(paths[0]);
        await loadConfig();
        addLog(`Game path detected: ${paths[0]}`);
      } else {
        addLog("No game installation found");
      }
    } catch (error) {
      addLog(`Error detecting game path: ${formatError(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const selectGamePath = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
        title: "Select FM26 Game Target Folder",
      });

      if (selected) {
        await tauriCommands.setGameTarget(selected);
        await loadConfig();
        addLog(`Game target set to: ${selected}`);
      }
    } catch (error) {
      addLog(`Error selecting game path: ${formatError(error)}`);
    }
  };

  const selectUserDirectory = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
        title: "Select FM26 User Directory",
      });

      if (selected) {
        const updatedConfig = {
          ...config!,
          user_dir_path: selected,
        };
        await tauriCommands.updateConfig(updatedConfig);
        await loadConfig();
        addLog(`User directory set to: ${selected}`);
      }
    } catch (error) {
      addLog(`Error selecting user directory: ${formatError(error)}`);
    }
  };

  const handleGameTargetChange = (value: string) => {
    setGameTargetInput(value);
  };

  const handleUserDirChange = (value: string) => {
    setUserDirInput(value);
  };

  const saveGameTarget = async () => {
    if (gameTargetInput !== config?.target_path) {
      try {
        await tauriCommands.setGameTarget(gameTargetInput);
        await loadConfig();
        addLog(`Game target updated to: ${gameTargetInput}`);
      } catch (error) {
        addLog(`Error updating game target: ${formatError(error)}`);
        // Revert on error
        setGameTargetInput(config?.target_path ?? "");
      }
    }
  };

  const saveUserDirectory = async () => {
    if (userDirInput !== config?.user_dir_path) {
      try {
        const updatedConfig = {
          ...config!,
          user_dir_path: userDirInput,
        };
        await tauriCommands.updateConfig(updatedConfig);
        await loadConfig();
        addLog(`User directory updated to: ${userDirInput}`);
      } catch (error) {
        addLog(`Error updating user directory: ${formatError(error)}`);
        // Revert on error
        setUserDirInput(config?.user_dir_path ?? "");
      }
    }
  };

  const toggleMod = async (modId: string, enable: boolean) => {
    try {
      if (enable) {
        await tauriCommands.enableMod(modId);
        addLog(`Enabled ${modId}`);
      } else {
        await tauriCommands.disableMod(modId);
        addLog(`Disabled ${modId}`);
      }
      await loadConfig();
      await loadMods();
    } catch (error) {
      addLog(`Error toggling mod: ${formatError(error)}`);
    }
  };

  const applyMods = async () => {
    if (!config?.target_path) {
      addLog("Please set game target first");
      toast.warning("Please set game target first");
      return;
    }

    try {
      setLoading(true);
      addLog("Applying mods...");
      toast.loading("Applying mods...", { id: "apply-mods" });
      const result = await tauriCommands.applyMods();
      addLog(result);
      addLog("Mods applied successfully");
      toast.success("Mods applied successfully!", { id: "apply-mods" });
    } catch (error) {
      addLog(`Error applying mods: ${formatError(error)}`);
      toast.error(`Failed to apply mods: ${formatError(error)}`, {
        id: "apply-mods",
      });
    } finally {
      setLoading(false);
    }
  };

  const removeMod = async (modId: string) => {
    if (!confirm(`Are you sure you want to remove ${modId}?`)) {
      return;
    }

    try {
      await tauriCommands.removeMod(modId);
      addLog(`Removed ${modId}`);
      toast.success(`Successfully removed ${modId}`);

      // Clear selection if we removed the selected mod
      if (selectedMod?.id === modId) {
        setSelectedMod(null);
        setModDetailsOpen(false);
      }

      await loadMods();
    } catch (error) {
      addLog(`Error removing mod: ${formatError(error)}`);
      toast.error(`Failed to remove mod: ${formatError(error)}`);
    }
  };

  const handleImportClick = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: "Mod Files",
            extensions: ["zip", "bundle", "fmf"],
          },
        ],
      });

      if (selected) {
        await handleImport(selected);
      }
    } catch (error) {
      addLog(`Error selecting file: ${formatError(error)}`);
    }
  };

  const handleImport = async (sourcePath: string) => {
    try {
      addLog(`Importing from: ${sourcePath}`);
      const result = await tauriCommands.importMod(sourcePath);
      addLog(`Successfully imported: ${result}`);
      toast.success(`Successfully imported: ${result}`);
      await loadMods();
    } catch (error) {
      const errorStr = String(error);

      if (errorStr === "NEEDS_METADATA") {
        // Mod needs metadata - show dialog
        setPendingImportPath(sourcePath);
        setMetadataDialogOpen(true);
        toast.info("Please provide mod metadata");
      } else {
        addLog(`Import failed: ${formatError(error)}`);
        toast.error(`Import failed: ${formatError(error)}`);
      }
    }
  };

  const handleMetadataSubmit = async (metadata: ModMetadata) => {
    if (!pendingImportPath) return;

    try {
      addLog(`Importing with metadata...`);
      const result = await tauriCommands.importMod(pendingImportPath, metadata);
      addLog(`Successfully imported: ${result}`);
      setMetadataDialogOpen(false);
      setPendingImportPath(null);
      await loadMods();
    } catch (error) {
      addLog(`Import failed: ${formatError(error)}`);
    }
  };

  const handleConflictDisable = async (modName: string) => {
    await toggleMod(modName, false);
  };

  const detectUserDirectory = async () => {
    try {
      setLoading(true);
      addLog("Detecting user directory...");
      const detectedPath = await tauriCommands.detectUserDir();

      // Update the config with the detected path
      const updatedConfig = {
        ...config!,
        user_dir_path: detectedPath,
      };
      await tauriCommands.updateConfig(updatedConfig);
      await loadConfig();
      addLog(`User directory detected: ${detectedPath}`);
    } catch (error) {
      addLog(`Error detecting user directory: ${formatError(error)}`);
    } finally {
      setLoading(false);
    }
  };

  const toggleDarkMode = async () => {
    const newDarkMode = !darkMode;
    setDarkMode(newDarkMode);
    if (newDarkMode) {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }

    // Save dark mode preference to config
    if (config) {
      const updatedConfig = { ...config, dark_mode: newDarkMode };
      try {
        await tauriCommands.updateConfig(updatedConfig);
        setConfig(updatedConfig);
        addLog(`Dark mode ${newDarkMode ? "enabled" : "disabled"}`);
      } catch (error) {
        addLog(`Error saving dark mode preference: ${formatError(error)}`);
      }
    }
  };

  // FM Name Fix handlers
  const checkNameFixStatus = async () => {
    try {
      setCheckingNameFix(true);
      addLog("Checking FM Name Fix installation status...");
      const isInstalled = await tauriCommands.checkNameFixInstalled();
      setNameFixInstalled(isInstalled);
      addLog(`FM Name Fix is ${isInstalled ? "installed" : "not installed"}`);
    } catch (error) {
      addLog(`Error checking FM Name Fix status: ${formatError(error)}`);
    } finally {
      setCheckingNameFix(false);
    }
  };

  const installNameFix = async () => {
    try {
      setInstallingNameFix(true);
      addLog("Installing FM Name Fix...");
      const result = await tauriCommands.installNameFix();
      addLog(result);
      toast.success("FM Name Fix installed successfully!");
      setNameFixInstalled(true);
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error installing FM Name Fix: ${errorMsg}`);
      toast.error(`Failed to install FM Name Fix: ${errorMsg}`);
    } finally {
      setInstallingNameFix(false);
    }
  };

  const uninstallNameFix = async () => {
    try {
      setInstallingNameFix(true);
      addLog("Uninstalling FM Name Fix...");
      const result = await tauriCommands.uninstallNameFix();
      addLog(result);
      toast.success("FM Name Fix uninstalled successfully!");
      setNameFixInstalled(false);
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error uninstalling FM Name Fix: ${errorMsg}`);
      toast.error(`Failed to uninstall FM Name Fix: ${errorMsg}`);
    } finally {
      setInstallingNameFix(false);
    }
  };

  // Drag and drop visual feedback
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    const init = async () => {
      try {
        await tauriCommands.initApp();
        const version = await tauriCommands.getAppVersion();
        setAppVersion(version);
        await loadConfig();
        await loadMods();
        addLog("FMMLoader26 initialized");

        // Set up Tauri drag and drop event listeners
        const unlistenDrop = await listen<string[]>(
          "tauri://file-drop",
          (event) => {
            const files = event.payload;
            if (files && files.length > 0) {
              void handleImport(files[0]);
            }
            setIsDragging(false);
          }
        );

        const unlistenDragOver = await listen("tauri://drag-over", () => {
          setIsDragging(true);
        });

        const unlistenDragDrop = await listen<{ paths: string[] }>(
          "tauri://drag-drop",
          (event) => {
            // In Tauri v2, drag-drop contains the file paths
            const paths = event.payload?.paths;
            if (paths && paths.length > 0) {
              void handleImport(paths[0]);
            }
            setIsDragging(false);
          }
        );

        const unlistenDragLeave = await listen("tauri://drag-leave", () => {
          setIsDragging(false);
        });

        // Check FM Name Fix installation status
        try {
          const isInstalled = await tauriCommands.checkNameFixInstalled();
          setNameFixInstalled(isInstalled);
        } catch (error) {
          // Silently fail - not critical
          console.error("Failed to check FM Name Fix status:", error);
        }

        return () => {
          unlistenDrop();
          unlistenDragOver();
          unlistenDragDrop();
          unlistenDragLeave();
        };
      } catch (error) {
        addLog(`Initialization error: ${formatError(error)}`);
      }
    };

    void init();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (config) {
      void loadMods();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [config]);

  return (
    <TooltipProvider>
      <div className="h-screen flex flex-col bg-background">
        {/* Custom TitleBar */}
        <TitleBar />

        {/* File Drop Zone - covers everything below titlebar */}
        {/* This invisible overlay catches file drops without blocking interactions */}
        <div className="fixed top-12 left-0 right-0 bottom-0 z-[1] pointer-events-none">
          {/* Drag overlay visual feedback */}
          {isDragging && (
            <div className="absolute inset-0 bg-primary/10 border-4 border-dashed border-primary flex items-center justify-center z-40 pointer-events-none">
              <div className="bg-background/95 p-8 rounded-lg shadow-lg">
                <Upload className="h-16 w-16 mx-auto mb-4 text-primary" />
                <p className="text-xl font-semibold">Drop mod file to import</p>
              </div>
            </div>
          )}
        </div>

        {/* Header */}
        <div className="border-b pt-6">
          <div className="flex items-center justify-between p-4">
            <div className="flex items-center gap-4">
              <svg
                className="h-14 w-auto fill-foreground"
                viewBox="0 0 800 600"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path d="M190.4,348.5l-5.3,24.4h60.4l-7.6,35.4h-60.2l-10.5,49.3h-48.9l31.4-147.4h118.4l-4,38.3h-73.7Z" />
                <path d="M264.1,457.5l31.4-147.4h53.7l20.6,62.8,45.3-62.8h58.3l-31.4,147.4h-47l17.3-82.5-55,72.9h-2.3l-26.5-72.6-17.5,82.3h-47Z" />
                <path d="M473.7,457.5l31.4-147.4h53.7l20.6,62.8,45.3-62.8h58.3l-31.4,147.4h-47l17.3-82.5-55,72.9h-2.3l-26.5-72.6-17.5,82.3h-47Z" />
                <path d="M117,559l18.1-85.1h28.2l-13.1,61.4h40.9l-7.2,23.7h-66.9Z" />
                <path d="M197.4,522.1c0-14.1,4.4-25.9,13.1-35.5,9.6-10.5,22.6-15.7,39.2-15.7,24.6,0,42.2,14.4,42.2,39.8s-4.4,25.9-13.1,35.4c-9.6,10.6-22.6,15.8-39.2,15.8-24.6,0-42.2-14.3-42.2-39.8h0ZM264.5,512.4c0-11.2-6.1-17.9-17.4-17.9s-22.4,12.4-22.4,25.8,6.1,18.1,17.4,18.1,22.4-12.7,22.4-26Z" />
                <path d="M288.2,559l43.9-85.1h46l7.7,85.1h-28.7l-1.3-16.5h-29.2l-8.4,16.5h-29.9,0ZM348.7,497l-12.6,25.7h18.9l-1.5-25.7h-4.7Z" />
                <path d="M395.6,559l18.1-85.1h32.5c30.3,0,45.4,11.9,45.4,35.6s-3.9,24.3-11.8,33.2c-9.6,10.8-24,16.3-42.9,16.3h-41.2,0ZM437.2,496.5l-8.5,39.9h10.2c15.1,0,24.3-10,24.3-24.8s-5.8-15.1-17.6-15.1h-8.4Z" />
                <path d="M537.3,495.9l-2.3,10.6h34.7l-4.1,19.1h-34.7l-2.4,11.4h46.2l-6.7,22h-72.2l18.1-85.1h70l-2.1,22h-44.5Z" />
                <path d="M670.9,559h-30.3l-11.8-21.5h-11.8l-4.6,21.5h-28.2l18.1-85.1h41.2c18.8,0,34.8,7,34.8,28.1s-7.5,27.2-22.5,32.1l15.1,24.9h0ZM638,516.4c7.3,0,12.5-4.6,12.5-12s-3.4-8.8-10.3-8.8h-14.3l-4.4,20.8h16.5Z" />
              </svg>
            </div>
            <div className="flex items-center gap-2">
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleImportClick}
                    disabled={loading}
                  >
                    <Upload className="mr-2 h-4 w-4" />
                    Import
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Import mod from ZIP, folder, or file</p>
                </TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setConflictsDialogOpen(true)}
                    disabled={loading || !config?.target_path}
                  >
                    <AlertTriangle className="mr-2 h-4 w-4" />
                    Conflicts
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Check for file conflicts between mods</p>
                </TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setRestoreDialogOpen(true)}
                    disabled={loading}
                  >
                    <History className="mr-2 h-4 w-4" />
                    Restore
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Rollback to a previous backup</p>
                </TooltipContent>
              </Tooltip>

              <Button
                variant="outline"
                size="sm"
                onClick={loadMods}
                disabled={loading}
              >
                <RefreshCw
                  className={`mr-2 h-4 w-4 ${loading ? "animate-spin" : ""}`}
                />
                Refresh
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setSettingsOpen(true)}
              >
                <Settings className="h-4 w-4" />
              </Button>
            </div>
          </div>

          {/* Game Target and User Directory */}
          <div className="px-4 pb-4 space-y-2">
            <div className="flex items-center gap-2">
              <div className="flex items-center gap-2 flex-1">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <span className="text-sm text-muted-foreground whitespace-nowrap">
                      Game Directory:
                    </span>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>
                      The FM26 installation folder containing the .bundle files
                    </p>
                  </TooltipContent>
                </Tooltip>
                <input
                  type="text"
                  value={gameTargetInput}
                  onChange={(e) => handleGameTargetChange(e.target.value)}
                  onBlur={saveGameTarget}
                  onKeyDown={(e) => e.key === "Enter" && saveGameTarget()}
                  className="flex-1 px-2 py-1 text-sm font-mono bg-background rounded border border-input focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                  placeholder="Not set - click 'Select' or 'Detect Game'"
                  disabled={loading}
                />
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={detectGamePath}
                disabled={loading}
              >
                Detect
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={selectGamePath}
                disabled={loading}
              >
                <FolderOpen className="h-4 w-4 text-foreground flex-shrink-0" />
              </Button>
            </div>

            <div className="flex items-center gap-2">
              <div className="flex items-center gap-2 flex-1">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <span className="text-sm text-muted-foreground whitespace-nowrap">
                      User Directory:
                    </span>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>
                      The FM26 User Directory where saves and settings are
                      stored
                    </p>
                  </TooltipContent>
                </Tooltip>
                <input
                  type="text"
                  value={userDirInput}
                  onChange={(e) => handleUserDirChange(e.target.value)}
                  onBlur={saveUserDirectory}
                  onKeyDown={(e) => e.key === "Enter" && saveUserDirectory()}
                  className="flex-1 px-2 py-1 text-sm font-mono bg-background rounded border border-input focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                  placeholder="Auto-detected from system"
                  disabled={loading}
                />
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={detectUserDirectory}
                disabled={loading}
              >
                Detect
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={selectUserDirectory}
                disabled={loading}
              >
                <FolderOpen className="h-4 w-4 text-foreground flex-shrink-0" />
              </Button>
            </div>
          </div>
        </div>

        {/* Main Content */}
        <div className="flex-1 overflow-hidden">
          <Tabs defaultValue="mods" className="h-full flex flex-col">
            <TabsList className="mx-4 mt-4">
              <TabsTrigger value="mods">Mods</TabsTrigger>
              <TabsTrigger value="utilities">Utilities</TabsTrigger>
              <TabsTrigger value="logs">Logs</TabsTrigger>
            </TabsList>

            <TabsContent
              value="mods"
              className="flex-1 overflow-hidden m-4 mt-2"
            >
              <div className="h-full">
                {/* Mods List */}
                <Card className="flex flex-col h-full">
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <div>
                        <CardTitle>Installed Mods</CardTitle>
                        <CardDescription>
                          {mods.length} mods installed
                        </CardDescription>
                      </div>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            onClick={applyMods}
                            disabled={loading || !config?.target_path}
                          >
                            <Download className="mr-2 h-4 w-4" />
                            Apply Mods
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          <p>Apply enabled mods to game (creates backup)</p>
                        </TooltipContent>
                      </Tooltip>
                    </div>
                  </CardHeader>
                  <CardContent className="flex-1 overflow-auto">
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead className="w-12">Status</TableHead>
                          <TableHead>Name</TableHead>
                          <TableHead>Type</TableHead>
                          <TableHead>Version</TableHead>
                          <TableHead>Author</TableHead>
                          <TableHead className="w-24">Actions</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {mods.map((mod) => (
                          <TableRow
                            key={mod.id}
                            className="cursor-pointer hover:bg-muted/50"
                            onClick={() => {
                              setSelectedMod(mod);
                              setModDetailsOpen(true);
                            }}
                          >
                            <TableCell>
                              <Switch
                                checked={mod.enabled}
                                onCheckedChange={(checked: boolean) => {
                                  void toggleMod(mod.id, checked);
                                }}
                                onClick={(e: React.MouseEvent) =>
                                  e.stopPropagation()
                                }
                              />
                            </TableCell>
                            <TableCell className="font-medium">
                              {mod.name}
                            </TableCell>
                            <TableCell>
                              <span className="inline-flex items-center rounded-md bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700 ring-1 ring-inset ring-blue-700/10">
                                {mod.mod_type}
                              </span>
                            </TableCell>
                            <TableCell className="text-muted-foreground">
                              {mod.version}
                            </TableCell>
                            <TableCell className="text-muted-foreground">
                              {mod.author}
                            </TableCell>
                            <TableCell>
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  void removeMod(mod.id);
                                }}
                              >
                                <Trash2 className="h-4 w-4 text-destructive" />
                              </Button>
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </CardContent>
                </Card>
              </div>
            </TabsContent>

            <TabsContent
              value="utilities"
              className="flex-1 overflow-hidden m-4 mt-2"
            >
              <Card className="h-full flex flex-col">
                <CardHeader>
                  <CardTitle className="flex items-center gap-2">
                    <Wrench className="h-5 w-5" />
                    FM Utilities
                  </CardTitle>
                  <CardDescription>
                    Additional tools and utilities for Football Manager
                  </CardDescription>
                </CardHeader>
                <CardContent className="flex-1 overflow-auto space-y-4">
                  {/* FM Name Fix */}
                  <Card>
                    <CardHeader>
                      <div className="flex items-center justify-between">
                        <div className="flex-1">
                          <CardTitle className="text-lg">FM Name Fix</CardTitle>
                          <CardDescription className="mt-1">
                            Fixes licensing issues and unlocks real names for
                            clubs, players, and competitions
                          </CardDescription>
                        </div>
                        <div className="flex items-center gap-2">
                          {checkingNameFix ? (
                            <div className="flex items-center gap-2 text-sm text-muted-foreground">
                              <RefreshCw className="h-4 w-4 animate-spin" />
                              Checking...
                            </div>
                          ) : nameFixInstalled ? (
                            <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
                              <CheckCircle2 className="h-4 w-4" />
                              Installed
                            </div>
                          ) : (
                            <div className="flex items-center gap-2 text-sm text-muted-foreground">
                              <XCircle className="h-4 w-4" />
                              Not installed
                            </div>
                          )}
                        </div>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="text-sm text-muted-foreground space-y-2">
                        <p>
                          <strong>What it does:</strong>
                        </p>
                        <ul className="list-disc list-inside space-y-1 ml-2">
                          <li>
                            Unlocks real names for clubs like AC Milan, Inter,
                            and Lazio
                          </li>
                          <li>Fixes Japanese player names</li>
                          <li>Removes fake/unlicensed content</li>
                          <li>Works with all leagues and competitions</li>
                        </ul>
                        <p className="text-xs mt-2">
                          Source:{" "}
                          <a
                            href="https://github.com/jo13310/NameFixFM26"
                            onClick={(e) => {
                              e.preventDefault();
                              void openUrl(
                                "https://github.com/jo13310/NameFixFM26"
                              );
                            }}
                            className="text-blue-600 dark:text-blue-400 hover:underline cursor-pointer"
                          >
                            github.com/jo13310/NameFixFM26
                          </a>
                        </p>
                      </div>
                      <div className="flex gap-2">
                        {!nameFixInstalled ? (
                          <Button
                            onClick={() => void installNameFix()}
                            disabled={installingNameFix || !config?.target_path}
                          >
                            {installingNameFix ? (
                              <>
                                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                                Installing...
                              </>
                            ) : (
                              <>
                                <Download className="mr-2 h-4 w-4" />
                                Install Name Fix
                              </>
                            )}
                          </Button>
                        ) : (
                          <Button
                            variant="destructive"
                            onClick={() => void uninstallNameFix()}
                            disabled={installingNameFix}
                          >
                            <Trash2 className="mr-2 h-4 w-4" />
                            Uninstall
                          </Button>
                        )}
                        <Button
                          variant="outline"
                          onClick={() => void checkNameFixStatus()}
                          disabled={checkingNameFix}
                        >
                          <RefreshCw className="mr-2 h-4 w-4" />
                          Check Status
                        </Button>
                      </div>
                      {!config?.target_path && (
                        <p className="text-sm text-amber-600 dark:text-amber-400">
                          Please set your Game Directory first
                        </p>
                      )}
                    </CardContent>
                  </Card>

                  {/* App Updater */}
                  <UpdaterCard />
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent
              value="logs"
              className="flex-1 overflow-hidden m-4 mt-2"
            >
              <Card className="h-full flex flex-col">
                <CardHeader>
                  <CardTitle>Activity Logs</CardTitle>
                  <CardDescription>
                    Recent activity and operations
                  </CardDescription>
                </CardHeader>
                <CardContent className="flex-1 overflow-auto">
                  <div className="font-mono text-xs space-y-1">
                    {logs.map((log, i) => (
                      <div key={i} className="text-muted-foreground">
                        {log}
                      </div>
                    ))}
                    {logs.length === 0 && (
                      <p className="text-sm text-muted-foreground">
                        No logs yet
                      </p>
                    )}
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </div>

        {/* Footer */}
        <div className="border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 p-3 flex items-center justify-between">
          <div className="text-xs text-muted-foreground font-medium">
            FMMLoader26 v{appVersion} | Created by JALCO / Justin Levine
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => openUrl("https://ko-fi.com/jalco")}
              className="hover:bg-[#FF5E5B] hover:text-white hover:border-[#FF5E5B] transition-colors"
            >
              <SiKofi className="mr-2 h-4 w-4" />
              Support on Ko-Fi
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => openUrl("https://discord.gg/AspRvTTAch")}
              className="hover:bg-[#5865F2] hover:text-white hover:border-[#5865F2] transition-colors"
            >
              <FaDiscord className="mr-2 h-4 w-4" />
              Discord
            </Button>
          </div>
        </div>

        {/* Dialogs */}
        <ModMetadataDialog
          open={metadataDialogOpen}
          onOpenChange={setMetadataDialogOpen}
          sourcePath={pendingImportPath ?? ""}
          onSubmit={handleMetadataSubmit}
        />

        <ConflictsDialog
          open={conflictsDialogOpen}
          onOpenChange={setConflictsDialogOpen}
          onDisableMod={handleConflictDisable}
        />

        <RestorePointsDialog
          open={restoreDialogOpen}
          onOpenChange={setRestoreDialogOpen}
          onRestore={() => {
            void loadMods();
            addLog("Restored from backup");
          }}
        />

        {/* Mod Details Sheet */}
        <Sheet open={modDetailsOpen} onOpenChange={setModDetailsOpen}>
          <SheetContent className="w-[400px] sm:w-[540px]">
            <SheetHeader>
              <SheetTitle>{selectedMod?.name ?? "Mod Details"}</SheetTitle>
              <SheetDescription>
                {selectedMod
                  ? `Version ${selectedMod.version}`
                  : "Select a mod to view details"}
              </SheetDescription>
            </SheetHeader>
            {selectedMod && (
              <div className="mt-6 space-y-4">
                <div className="space-y-2">
                  <div>
                    <span className="text-sm font-medium">Author:</span>
                    <p className="text-sm text-muted-foreground">
                      {selectedMod.author || "Unknown"}
                    </p>
                  </div>

                  <div>
                    <span className="text-sm font-medium">Type:</span>
                    <p className="text-sm text-muted-foreground">
                      {selectedMod.mod_type}
                    </p>
                  </div>

                  <div>
                    <span className="text-sm font-medium">Description:</span>
                    <p className="text-sm text-muted-foreground">
                      {selectedMod.description || "No description available"}
                    </p>
                  </div>

                  {selectedMod.license && (
                    <div>
                      <span className="text-sm font-medium">License:</span>
                      <p className="text-sm text-muted-foreground">
                        {selectedMod.license}
                      </p>
                    </div>
                  )}

                  {selectedMod.files && selectedMod.files.length > 0 && (
                    <div>
                      <span className="text-sm font-medium">
                        Files ({selectedMod.files.length}):
                      </span>
                      <ul className="text-sm text-muted-foreground list-disc list-inside max-h-60 overflow-y-auto">
                        {selectedMod.files.map((file, i) => (
                          <li key={i} className="truncate">
                            {file.source}
                          </li>
                        ))}
                      </ul>
                    </div>
                  )}
                </div>

                <div className="pt-4 space-y-2">
                  <Button
                    className="w-full"
                    variant={selectedMod.enabled ? "destructive" : "default"}
                    onClick={() =>
                      toggleMod(selectedMod.id, !selectedMod.enabled)
                    }
                  >
                    {selectedMod.enabled ? "Disable Mod" : "Enable Mod"}
                  </Button>
                  <Button
                    className="w-full"
                    variant="outline"
                    onClick={() => removeMod(selectedMod.id)}
                  >
                    <Trash2 className="mr-2 h-4 w-4" />
                    Remove Mod
                  </Button>
                </div>
              </div>
            )}
          </SheetContent>
        </Sheet>

        {/* Settings Sheet */}
        <Sheet open={settingsOpen} onOpenChange={setSettingsOpen}>
          <SheetContent>
            <SheetHeader>
              <SheetTitle>Settings</SheetTitle>
              <SheetDescription>
                Configure FMMLoader26 preferences
              </SheetDescription>
            </SheetHeader>
            <div className="mt-6 space-y-6">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <div className="text-sm font-medium">Dark Mode</div>
                  <div className="text-sm text-muted-foreground">
                    Toggle dark mode theme
                  </div>
                </div>
                <Switch checked={darkMode} onCheckedChange={toggleDarkMode} />
              </div>

              <div className="border-t pt-4">
                <div className="space-y-2">
                  <div className="text-sm font-medium">Application Logs</div>
                  <div className="text-sm text-muted-foreground">
                    View application logs for troubleshooting. Logs from the
                    last 10 sessions are kept.
                  </div>
                  <Button
                    variant="outline"
                    className="w-full mt-2"
                    onClick={async () => {
                      try {
                        await tauriCommands.openLogsFolder();
                        addLog("Opened logs folder");
                      } catch (error) {
                        addLog(
                          `Failed to open logs folder: ${formatError(error)}`
                        );
                      }
                    }}
                  >
                    <FolderOpen className="mr-2 h-4 w-4" />
                    Open Logs Folder
                  </Button>
                </div>
              </div>
            </div>
          </SheetContent>
        </Sheet>
        <Toaster />
      </div>
    </TooltipProvider>
  );
}

export default App;
