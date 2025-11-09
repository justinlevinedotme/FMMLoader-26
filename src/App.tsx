import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { tauriCommands, type Config, type ModManifest, type ModMetadata } from "@/hooks/useTauri";
import { Folder, FolderOpen, RefreshCw, Download, Trash2, Power, PowerOff, Upload, AlertTriangle, History } from "lucide-react";
import { ModMetadataDialog } from "@/components/ModMetadataDialog";
import { ConflictsDialog } from "@/components/ConflictsDialog";
import { RestorePointsDialog } from "@/components/RestorePointsDialog";
import { DropZone } from "@/components/DropZone";

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

  // Dialog states
  const [metadataDialogOpen, setMetadataDialogOpen] = useState(false);
  const [conflictsDialogOpen, setConflictsDialogOpen] = useState(false);
  const [restoreDialogOpen, setRestoreDialogOpen] = useState(false);
  const [pendingImportPath, setPendingImportPath] = useState<string | null>(null);

  const addLog = (message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setLogs((prev) => [`[${timestamp}] ${message}`, ...prev].slice(0, 100));
  };

  const loadConfig = async () => {
    try {
      const cfg = await tauriCommands.getConfig();
      setConfig(cfg);
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

  const handleFileDrop = async (files: FileList) => {
    if (files.length > 0) {
      const file = files[0];
      // On web we get a File object, but Tauri needs a path
      // In Tauri, drag & drop gives us the file path directly
      const path = (file as any).path || file.name;
      await handleImport(path);
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

  useEffect(() => {
    const init = async () => {
      try {
        await tauriCommands.initApp();
        await loadConfig();
        await loadMods();
        addLog("FMMLoader26 initialized");
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
    <div className="h-screen flex flex-col bg-background">
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
          </div>
        </div>

        {/* Game Target Display */}
        {config?.target_path && (
          <div className="px-4 pb-4">
            <div className="flex items-center gap-2 text-sm">
              <FolderOpen className="h-4 w-4 text-muted-foreground" />
              <span className="text-muted-foreground">Game Target:</span>
              <span className="font-mono">{config.target_path}</span>
            </div>
          </div>
        )}
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-hidden">
        <Tabs defaultValue="mods" className="h-full flex flex-col">
          <TabsList className="mx-4 mt-4">
            <TabsTrigger value="mods">Mods</TabsTrigger>
            <TabsTrigger value="logs">Logs</TabsTrigger>
          </TabsList>

          <TabsContent value="mods" className="flex-1 overflow-hidden m-4 mt-2 space-y-4">
            {/* Drop Zone */}
            <DropZone onDrop={handleFileDrop} />

            <div className="grid grid-cols-3 gap-4 h-full">
              {/* Mods List */}
              <Card className="col-span-2 flex flex-col">
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
                          className={`cursor-pointer ${selectedMod?.id === mod.id ? "bg-muted" : ""}`}
                          onClick={() => setSelectedMod(mod)}
                        >
                          <TableCell>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={(e) => {
                                e.stopPropagation();
                                toggleMod(mod.id, !mod.enabled);
                              }}
                            >
                              {mod.enabled ? (
                                <Power className="h-4 w-4 text-green-500" />
                              ) : (
                                <PowerOff className="h-4 w-4 text-muted-foreground" />
                              )}
                            </Button>
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

              {/* Mod Details */}
              <Card className="flex flex-col">
                <CardHeader>
                  <CardTitle>Mod Details</CardTitle>
                </CardHeader>
                <CardContent className="flex-1 overflow-auto">
                  {selectedMod ? (
                    <div className="space-y-4">
                      <div>
                        <h3 className="font-semibold text-lg">{selectedMod.name}</h3>
                        <p className="text-sm text-muted-foreground">v{selectedMod.version}</p>
                      </div>

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
                            <span className="text-sm font-medium">Files:</span>
                            <ul className="text-sm text-muted-foreground list-disc list-inside">
                              {selectedMod.files.slice(0, 5).map((file, i) => (
                                <li key={i} className="truncate">{file.source}</li>
                              ))}
                              {selectedMod.files.length > 5 && (
                                <li>... and {selectedMod.files.length - 5} more</li>
                              )}
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
                  ) : (
                    <p className="text-sm text-muted-foreground">Select a mod to view details</p>
                  )}
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
    </div>
  );
}

export default App;
