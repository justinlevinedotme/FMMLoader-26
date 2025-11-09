import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { open as openUrl } from "@tauri-apps/plugin-shell";
import { listen } from '@tauri-apps/api/event';
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Switch } from "@/components/ui/switch";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { tauriCommands, type Config, type ModManifest, type ModMetadata, type UpdateInfo } from "@/hooks/useTauri";
import { Folder, FolderOpen, RefreshCw, Download, Trash2, Upload, AlertTriangle, History, Settings, DollarSign, MessageCircle } from "lucide-react";
import { ModMetadataDialog } from "@/components/ModMetadataDialog";
import { ConflictsDialog } from "@/components/ConflictsDialog";
import { RestorePointsDialog } from "@/components/RestorePointsDialog";
import { UpdateBanner } from "@/components/UpdateBanner";

interface ModWithInfo extends ModManifest {
  id: string;
  enabled: boolean;
}

function App() {
  const [config, setConfig] = useState<Config | null>(null);
  const [mods, setMods] = useState<ModWithInfo[]>([]);
  const [selectedMod, setSelectedMod] = useState<ModWithInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [logs, setLogs] = useState<string[]>([]);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);

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
  const [pendingImportPath, setPendingImportPath] = useState<string | null>(null);

  const addLog = (message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setLogs((prev) => [`[${timestamp}] ${message}`, ...prev].slice(0, 100));
  };

  const loadConfig = async () => {
    try {
      const cfg = await tauriCommands.getConfig();
      setConfig(cfg);
      setGameTargetInput(cfg.target_path || "");
      setUserDirInput(cfg.user_dir_path || "");
      addLog("Configuration loaded");
    } catch (error) {
      addLog(`Error loading config: ${error}`);
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
            enabled: config?.enabled_mods?.includes(name) || false,
          });
        } catch (error) {
          addLog(`Failed to load mod ${name}: ${error}`);
        }
      }

      setMods(modsWithInfo);
      addLog(`Loaded ${modsWithInfo.length} mods`);
    } catch (error) {
      addLog(`Error loading mods: ${error}`);
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
      addLog(`Error detecting game path: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const selectGamePath = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
        title: "Select FM26 Game Target Folder"
      });

      if (selected) {
        await tauriCommands.setGameTarget(selected as string);
        await loadConfig();
        addLog(`Game target set to: ${selected}`);
      }
    } catch (error) {
      addLog(`Error selecting game path: ${error}`);
    }
  };

  const selectUserDirectory = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
        title: "Select FM26 User Directory"
      });

      if (selected) {
        const updatedConfig = {
          ...config!,
          user_dir_path: selected as string
        };
        await tauriCommands.updateConfig(updatedConfig);
        await loadConfig();
        addLog(`User directory set to: ${selected}`);
      }
    } catch (error) {
      addLog(`Error selecting user directory: ${error}`);
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
        addLog(`Error updating game target: ${error}`);
        // Revert on error
        setGameTargetInput(config?.target_path || "");
      }
    }
  };

  const saveUserDirectory = async () => {
    if (userDirInput !== config?.user_dir_path) {
      try {
        const updatedConfig = {
          ...config!,
          user_dir_path: userDirInput
        };
        await tauriCommands.updateConfig(updatedConfig);
        await loadConfig();
        addLog(`User directory updated to: ${userDirInput}`);
      } catch (error) {
        addLog(`Error updating user directory: ${error}`);
        // Revert on error
        setUserDirInput(config?.user_dir_path || "");
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
      addLog(`Error toggling mod: ${error}`);
    }
  };

  const applyMods = async () => {
    if (!config?.target_path) {
      addLog("Please set game target first");
      return;
    }

    try {
      setLoading(true);
      addLog("Applying mods...");
      const result = await tauriCommands.applyMods();
      addLog(result);
      addLog("Mods applied successfully");
    } catch (error) {
      addLog(`Error applying mods: ${error}`);
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

      // Clear selection if we removed the selected mod
      if (selectedMod?.id === modId) {
        setSelectedMod(null);
        setModDetailsOpen(false);
      }

      await loadMods();
    } catch (error) {
      addLog(`Error removing mod: ${error}`);
    }
  };

  const handleImportClick = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{
          name: 'Mod Files',
          extensions: ['zip', 'bundle', 'fmf']
        }]
      });

      if (selected) {
        await handleImport(selected as string);
      }
    } catch (error) {
      addLog(`Error selecting file: ${error}`);
    }
  };

  const handleImport = async (sourcePath: string) => {
    try {
      addLog(`Importing from: ${sourcePath}`);
      const result = await tauriCommands.importMod(sourcePath);
      addLog(`Successfully imported: ${result}`);
      await loadMods();
    } catch (error) {
      const errorStr = String(error);

      if (errorStr === "NEEDS_METADATA") {
        // Mod needs metadata - show dialog
        setPendingImportPath(sourcePath);
        setMetadataDialogOpen(true);
      } else {
        addLog(`Import failed: ${error}`);
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
      addLog(`Import failed: ${error}`);
    }
  };

  const handleConflictDisable = async (modName: string) => {
    await toggleMod(modName, false);
  };

  const detectUserDirectory = async () => {
    try {
      setLoading(true);
      addLog("Detecting user directory...");
      // The backend should auto-detect this, but we can trigger a refresh
      await loadConfig();
      if (config?.user_dir_path) {
        addLog(`User directory detected: ${config.user_dir_path}`);
      } else {
        addLog("Could not auto-detect user directory");
      }
    } catch (error) {
      addLog(`Error detecting user directory: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const toggleDarkMode = () => {
    setDarkMode(!darkMode);
    if (!darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
  };

  // Drag and drop handlers for the entire window
  const [isDragging, setIsDragging] = useState(false);

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    // Only set to false if leaving the window entirely
    if (e.currentTarget === e.target) {
      setIsDragging(false);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);

    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0];
      // In Tauri, we get the file path from the path property
      const path = (file as any).path || file.name;
      await handleImport(path);
    }
  };

  useEffect(() => {
    const init = async () => {
      try {
        await tauriCommands.initApp();
        await loadConfig();
        await loadMods();
        addLog("FMMLoader26 initialized");

        // Set up file drop listener
        const unlisten = await listen('tauri://file-drop', (event: any) => {
          const files = event.payload as string[];
          if (files && files.length > 0) {
            handleImport(files[0]);
          }
        });

        // Check for updates
        try {
          const updates = await tauriCommands.checkUpdates();
          setUpdateInfo(updates);
          if (updates.has_update) {
            addLog(`Update available: ${updates.latest_version}`);
          }
        } catch (error) {
          // Silently fail update check - not critical
          console.error("Failed to check for updates:", error);
        }

        return () => {
          unlisten();
        };
      } catch (error) {
        addLog(`Initialization error: ${error}`);
      }
    };

    init();
  }, []);

  useEffect(() => {
    if (config) {
      loadMods();
    }
  }, [config]);

  return (
    <div
      className="h-screen flex flex-col bg-background"
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {/* Drag overlay */}
      {isDragging && (
        <div className="fixed inset-0 bg-primary/10 border-4 border-dashed border-primary z-50 flex items-center justify-center pointer-events-none">
          <div className="bg-background/95 p-8 rounded-lg shadow-lg">
            <Upload className="h-16 w-16 mx-auto mb-4 text-primary" />
            <p className="text-xl font-semibold">Drop mod file to import</p>
          </div>
        </div>
      )}
      {/* Update Banner */}
      {updateInfo && updateInfo.has_update && (
        <UpdateBanner
          updateInfo={updateInfo}
          onDismiss={() => setUpdateInfo(null)}
        />
      )}

      {/* Header */}
      <div className="border-b">
        <div className="flex items-center justify-between p-4">
          <div>
            <h1 className="text-2xl font-bold">FMMLoader26</h1>
            <p className="text-sm text-muted-foreground">Football Manager 2026 Mod Manager</p>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleImportClick}
              disabled={loading}
            >
              <Upload className="mr-2 h-4 w-4" />
              Import
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setConflictsDialogOpen(true)}
              disabled={loading || !config?.target_path}
            >
              <AlertTriangle className="mr-2 h-4 w-4" />
              Conflicts
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setRestoreDialogOpen(true)}
              disabled={loading}
            >
              <History className="mr-2 h-4 w-4" />
              Restore
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={detectGamePath}
              disabled={loading}
            >
              <Folder className="mr-2 h-4 w-4" />
              Detect Game
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={loadMods}
              disabled={loading}
            >
              <RefreshCw className={`mr-2 h-4 w-4 ${loading ? "animate-spin" : ""}`} />
              Refresh
            </Button>
            <div className="border-l pl-2 ml-2 flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => openUrl('https://ko-fi.com/jalco')}
              >
                <DollarSign className="mr-2 h-4 w-4" />
                Support
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => openUrl('https://discord.gg/AspRvTTAch')}
              >
                <MessageCircle className="mr-2 h-4 w-4" />
                Discord
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
        </div>

        {/* Game Target and User Directory */}
        <div className="px-4 pb-4 space-y-2">
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-2 flex-1">
              <FolderOpen className="h-4 w-4 text-muted-foreground flex-shrink-0" />
              <span className="text-sm text-muted-foreground whitespace-nowrap">Game Target:</span>
              <input
                type="text"
                value={gameTargetInput}
                onChange={(e) => handleGameTargetChange(e.target.value)}
                onBlur={saveGameTarget}
                onKeyDown={(e) => e.key === 'Enter' && saveGameTarget()}
                className="flex-1 px-2 py-1 text-sm font-mono bg-background rounded border border-input focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                placeholder="Not set - click 'Select' or 'Detect Game'"
                disabled={loading}
              />
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={selectGamePath}
              disabled={loading}
            >
              Select...
            </Button>
          </div>

          <div className="flex items-center gap-2">
            <div className="flex items-center gap-2 flex-1">
              <Folder className="h-4 w-4 text-muted-foreground flex-shrink-0" />
              <span className="text-sm text-muted-foreground whitespace-nowrap">User Directory:</span>
              <input
                type="text"
                value={userDirInput}
                onChange={(e) => handleUserDirChange(e.target.value)}
                onBlur={saveUserDirectory}
                onKeyDown={(e) => e.key === 'Enter' && saveUserDirectory()}
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
              Select...
            </Button>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        <Tabs defaultValue="mods" className="h-full flex flex-col">
          <TabsList className="mx-4 mt-4">
            <TabsTrigger value="mods">Mods</TabsTrigger>
            <TabsTrigger value="logs">Logs</TabsTrigger>
          </TabsList>

          <TabsContent value="mods" className="flex-1 overflow-hidden m-4 mt-2">
            <div className="h-full">
              {/* Mods List */}
              <Card className="flex flex-col h-full">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>Installed Mods</CardTitle>
                      <CardDescription>{mods.length} mods installed</CardDescription>
                    </div>
                    <Button onClick={applyMods} disabled={loading || !config?.target_path}>
                      <Download className="mr-2 h-4 w-4" />
                      Apply Mods
                    </Button>
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
                                toggleMod(mod.id, checked);
                              }}
                              onClick={(e: React.MouseEvent) => e.stopPropagation()}
                            />
                          </TableCell>
                          <TableCell className="font-medium">{mod.name}</TableCell>
                          <TableCell>
                            <span className="inline-flex items-center rounded-md bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700 ring-1 ring-inset ring-blue-700/10">
                              {mod.mod_type}
                            </span>
                          </TableCell>
                          <TableCell className="text-muted-foreground">{mod.version}</TableCell>
                          <TableCell className="text-muted-foreground">{mod.author}</TableCell>
                          <TableCell>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={(e) => {
                                e.stopPropagation();
                                removeMod(mod.id);
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

          <TabsContent value="logs" className="flex-1 overflow-hidden m-4 mt-2">
            <Card className="h-full flex flex-col">
              <CardHeader>
                <CardTitle>Activity Logs</CardTitle>
                <CardDescription>Recent activity and operations</CardDescription>
              </CardHeader>
              <CardContent className="flex-1 overflow-auto">
                <div className="font-mono text-xs space-y-1">
                  {logs.map((log, i) => (
                    <div key={i} className="text-muted-foreground">
                      {log}
                    </div>
                  ))}
                  {logs.length === 0 && (
                    <p className="text-sm text-muted-foreground">No logs yet</p>
                  )}
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>

      {/* Footer */}
      <div className="border-t p-2 text-center text-xs text-muted-foreground">
        FMMLoader26 v0.1.0 | Created by JALCO / Justin Levine
      </div>

      {/* Dialogs */}
      <ModMetadataDialog
        open={metadataDialogOpen}
        onOpenChange={setMetadataDialogOpen}
        sourcePath={pendingImportPath || ""}
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
          loadMods();
          addLog("Restored from backup");
        }}
      />

      {/* Mod Details Sheet */}
      <Sheet open={modDetailsOpen} onOpenChange={setModDetailsOpen}>
        <SheetContent className="w-[400px] sm:w-[540px]">
          <SheetHeader>
            <SheetTitle>{selectedMod?.name || "Mod Details"}</SheetTitle>
            <SheetDescription>
              {selectedMod ? `Version ${selectedMod.version}` : "Select a mod to view details"}
            </SheetDescription>
          </SheetHeader>
          {selectedMod && (
            <div className="mt-6 space-y-4">
              <div className="space-y-2">
                <div>
                  <span className="text-sm font-medium">Author:</span>
                  <p className="text-sm text-muted-foreground">{selectedMod.author || "Unknown"}</p>
                </div>

                <div>
                  <span className="text-sm font-medium">Type:</span>
                  <p className="text-sm text-muted-foreground">{selectedMod.mod_type}</p>
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
                    <p className="text-sm text-muted-foreground">{selectedMod.license}</p>
                  </div>
                )}

                {selectedMod.files && selectedMod.files.length > 0 && (
                  <div>
                    <span className="text-sm font-medium">Files ({selectedMod.files.length}):</span>
                    <ul className="text-sm text-muted-foreground list-disc list-inside max-h-60 overflow-y-auto">
                      {selectedMod.files.map((file, i) => (
                        <li key={i} className="truncate">{file.source}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>

              <div className="pt-4 space-y-2">
                <Button
                  className="w-full"
                  variant={selectedMod.enabled ? "destructive" : "default"}
                  onClick={() => toggleMod(selectedMod.id, !selectedMod.enabled)}
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
          <div className="mt-6 space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <div className="text-sm font-medium">Dark Mode</div>
                <div className="text-sm text-muted-foreground">
                  Toggle dark mode theme
                </div>
              </div>
              <Switch
                checked={darkMode}
                onCheckedChange={toggleDarkMode}
              />
            </div>
          </div>
        </SheetContent>
      </Sheet>
    </div>
  );
}

export default App;
