import { useEffect, useRef, useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { open as openUrl } from '@tauri-apps/plugin-shell';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet';
import { tauriCommands } from '@/hooks/useTauri';
import type {
  Config,
  ModMetadata,
  NameFixSource,
  ExtractionProgress,
  GraphicsPackMetadata,
  GraphicsPackAnalysis,
  GraphicsPackIssue,
  GraphicsConflictInfo,
} from '@/types';
import {
  FolderOpen,
  RefreshCw,
  Trash2,
  Upload,
  AlertTriangle,
  History,
  Ellipsis,
} from 'lucide-react';
import { FaDiscord } from 'react-icons/fa6';
import { SiKofi } from 'react-icons/si';
import { ModMetadataDialog } from '@/components/ModMetadataDialog';
import { ConflictsDialog } from '@/components/ConflictsDialog';
import { RestorePointsDialog } from '@/components/RestorePointsDialog';
import { GraphicsPackConfirmDialog } from '@/components/GraphicsPackConfirmDialog';
import { TitleBar } from '@/components/TitleBar';
import { Toaster } from '@/components/ui/sonner';
import { UpdateBanner } from '@/components/UpdateBanner';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { ModsTab, GraphicsTab, NameFixTab, SettingsTab, type ModWithInfo } from '@/components/tabs';
import {
  I18nProvider,
  detectSystemLocale,
  ensureSupportedLocale,
  useI18n,
  type SupportedLocale,
} from '@/lib/i18n';

type DebugUIProps = {
  metadataDialogOpen: boolean;
  conflictsDialogOpen: boolean;
  restoreDialogOpen: boolean;
  settingsOpen: boolean;
  modDetailsOpen: boolean;
  setMetadataDialogOpen: (open: boolean) => void;
  setConflictsDialogOpen: (open: boolean) => void;
  setRestoreDialogOpen: (open: boolean) => void;
  setSettingsOpen: (open: boolean) => void;
  setModDetailsOpen: (open: boolean) => void;
  setSelectedMod: (mod: ModWithInfo | null) => void;
  setPendingGraphicsAnalysis: (analysis: GraphicsPackAnalysis | null) => void;
  setPendingGraphicsPath: (path: string | null) => void;
};

function DebugUI({
  metadataDialogOpen,
  conflictsDialogOpen,
  restoreDialogOpen,
  settingsOpen,
  modDetailsOpen,
  setMetadataDialogOpen,
  setConflictsDialogOpen,
  setRestoreDialogOpen,
  setSettingsOpen,
  setModDetailsOpen,
  setSelectedMod,
  setPendingGraphicsAnalysis,
  setPendingGraphicsPath,
}: DebugUIProps) {
  const sampleMod: ModWithInfo = {
    id: 'debug-mod',
    name: 'Debug Mod',
    version: '0.0.0',
    mod_type: 'graphics',
    author: 'Debug',
    homepage: '',
    description: 'Sample mod for UI debug',
    license: 'MIT',
    compatibility: { fm_version: '26' },
    dependencies: [],
    conflicts: [],
    load_after: [],
    files: [
      { source: 'sample/face.png', target_subpath: 'graphics/faces/face.png' },
      { source: 'sample/config.xml', target_subpath: 'graphics/faces/config.xml' },
    ],
    enabled: false,
  };

  const sampleAnalysis: GraphicsPackAnalysis = {
    pack_type: 'Faces',
    confidence: 0.82,
    suggested_paths: ['faces/sample-pack', 'faces/alt-path'],
    file_count: 2,
    total_size_bytes: 2048,
    has_config_xml: true,
    subdirectory_breakdown: { faces: 2 },
    is_flat_pack: true,
  };

  const openGraphicsDialog = () => {
    setPendingGraphicsPath('debug/sample/pack.zip');
    setPendingGraphicsAnalysis(sampleAnalysis);
  };

  const openModDetails = () => {
    setSelectedMod(sampleMod);
    setModDetailsOpen(true);
  };

  const openStates = [
    metadataDialogOpen ? 'Metadata' : null,
    conflictsDialogOpen ? 'Conflicts' : null,
    restoreDialogOpen ? 'Restore Points' : null,
    settingsOpen ? 'Settings' : null,
    modDetailsOpen ? 'Mod Details' : null,
  ].filter(Boolean);

  return (
    <Card>
      <CardHeader>
        <CardTitle>UI Debug / Playground (dev only)</CardTitle>
        <CardDescription>Quick toggles to open dialogs without full flows.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
          <Button variant="secondary" onClick={() => setMetadataDialogOpen(true)}>
            Open Metadata Dialog
          </Button>
          <Button variant="secondary" onClick={() => setConflictsDialogOpen(true)}>
            Open Conflicts Dialog
          </Button>
          <Button variant="secondary" onClick={() => setRestoreDialogOpen(true)}>
            Open Restore Points
          </Button>
          <Button variant="secondary" onClick={() => setSettingsOpen(true)}>
            Open Settings Sheet
          </Button>
          <Button variant="secondary" onClick={openModDetails}>
            Open Mod Details
          </Button>
          <Button variant="secondary" onClick={openGraphicsDialog}>
            Open Graphics Confirm
          </Button>
        </div>
        <div className="text-xs text-muted-foreground">
          Open modals:{' '}
          {openStates.length > 0 ? openStates.join(', ') : 'None – toggle above to inspect.'}
        </div>
      </CardContent>
    </Card>
  );
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
  const [, setLogs] = useState<string[]>([]);
  const [appVersion, setAppVersion] = useState('');
  const [blockingMessage, setBlockingMessage] = useState<string | null>(null);

  // Import name fix dialog state
  const [importNameFixDialogOpen, setImportNameFixDialogOpen] = useState(false);
  const [pendingImportFilePath, setPendingImportFilePath] = useState<string | null>(null);
  const [importNameFixName, setImportNameFixName] = useState('');

  // Editable path states
  const [gameTargetInput, setGameTargetInput] = useState('');
  const [userDirInput, setUserDirInput] = useState('');
  const [darkMode, setDarkMode] = useState(false);
  const [locale, setLocale] = useState<SupportedLocale>('en');
  const [localeReady, setLocaleReady] = useState(false);
  const localeInitialized = useRef(false);
  const localeOptions: { value: SupportedLocale; emoji: string; label: string }[] = [
    { value: 'en', emoji: '🇺🇸', label: 'English', contributor: 'Justin Levine' },
    { value: 'ko', emoji: '🇰🇷', label: '한국어', contributor: 'AI' },
    { value: 'tr', emoji: '🇹🇷', label: 'Türkçe', contributor: 'AI' },
    { value: 'pt-PT', emoji: '🇵🇹', label: 'Português (Portugal)', contributor: 'AI' },
    { value: 'de', emoji: '🇩🇪', label: 'Deutsch', contributor: 'AI' },
  ];

  // Log toast emissions to help debug visibility issues in dev
  useEffect(() => {
    const kinds: Array<'success' | 'error' | 'info' | 'warning' | 'loading'> = [
      'success',
      'error',
      'info',
      'warning',
      'loading',
    ];

    kinds.forEach((kind) => {
      const original = (toast as { [key: string]: ((...args: unknown[]) => unknown) | undefined })[
        kind
      ];
      if (!original || (original as { __instrumented?: boolean }).__instrumented) return;
      const wrapped = (message: unknown, options?: { id?: string }) => {
        console.log(`[toast:${kind}]`, { id: options?.id, message });
        return original(message, options);
      };
      (wrapped as { __instrumented?: boolean }).__instrumented = true;
      (toast as { [key: string]: unknown })[kind] = wrapped;
    });
  }, []);

  const handleLocaleChange = async (nextLocale: SupportedLocale) => {
    setLocale(nextLocale);
    if (!config) return;
    const updatedConfig = { ...config, language: nextLocale };
    try {
      await tauriCommands.updateConfig(updatedConfig);
      setConfig(updatedConfig);
      addLog(`Language set to ${nextLocale}`);
    } catch (error) {
      addLog(`Error saving locale preference: ${formatError(error)}`);
    }
  };

  const triggerToastTests = () => {
    toast.success(txRef.current('toasts.validateGraphics.success'), { id: 'test-validate' });
    toast.info(txRef.current('toasts.validateGraphics.issues', { count: 2 }));
    toast.loading(txRef.current('toasts.migrateGraphics.loading', { name: 'Test Pack' }), {
      id: 'test-migrate',
    });
    toast.success(txRef.current('toasts.migrateGraphics.success'), { id: 'test-migrate' });
    toast.error(txRef.current('toasts.migrateGraphics.error', { message: 'Example error' }));
    toast.success(txRef.current('toasts.nameFixUninstallSuccess'), { id: 'test-namefix' });
  };

  // Dialog states
  const [metadataDialogOpen, setMetadataDialogOpen] = useState(false);
  const [conflictsDialogOpen, setConflictsDialogOpen] = useState(false);
  const [restoreDialogOpen, setRestoreDialogOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [modDetailsOpen, setModDetailsOpen] = useState(false);
  const [pendingImportPath, setPendingImportPath] = useState<string | null>(null);

  // FM Name Fix states
  const [nameFixInstalled, setNameFixInstalled] = useState(false);
  const [checkingNameFix, setCheckingNameFix] = useState(false);
  const [installingNameFix, setInstallingNameFix] = useState(false);
  const [nameFixSources, setNameFixSources] = useState<NameFixSource[]>([]);
  const [activeNameFixId, setActiveNameFixId] = useState<string | null>(null);
  const [selectedNameFixId, setSelectedNameFixId] = useState<string>('');

  // Graphics pack states
  const [graphicsProgress, setGraphicsProgress] = useState<ExtractionProgress | null>(null);
  const [importingGraphics, setImportingGraphics] = useState(false);
  const [graphicsPacks, setGraphicsPacks] = useState<GraphicsPackMetadata[]>([]);
  const [graphicsIssues, setGraphicsIssues] = useState<GraphicsPackIssue[]>([]);
  const [showValidationDialog, setShowValidationDialog] = useState(false);
  const [validatingGraphics, setValidatingGraphics] = useState(false);
  const [pendingGraphicsAnalysis, setPendingGraphicsAnalysis] =
    useState<GraphicsPackAnalysis | null>(null);
  const [pendingGraphicsPath, setPendingGraphicsPath] = useState<string | null>(null);
  const [migrationProgress, setMigrationProgress] = useState<ExtractionProgress | null>(null);
  const [migratingPack, setMigratingPack] = useState(false);
  const [graphicsConflict, setGraphicsConflict] = useState<GraphicsConflictInfo | null>(null);
  const [showConflictDialog, setShowConflictDialog] = useState(false);
  const [pendingInstall, setPendingInstall] = useState<{
    path: string;
    shouldSplit: boolean;
  } | null>(null);

  // Confirmation dialog states
  const [confirmDeleteMod, setConfirmDeleteMod] = useState<string | null>(null);
  const [confirmDeleteNameFix, setConfirmDeleteNameFix] = useState<NameFixSource | null>(null);

  const addLog = (message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setLogs((prev) => [`[${timestamp}] ${message}`, ...prev].slice(0, 100));
  };

  const resolveAndApplyLocale = async (cfg: Config) => {
    const configured = ensureSupportedLocale(cfg.language);
    const detected = await detectSystemLocale();
    const resolved = configured ?? ensureSupportedLocale(detected ?? undefined);
    setLocale(resolved);
    setLocaleReady(true);

    if (!localeInitialized.current && cfg.language != resolved) {
      const updatedConfig = { ...cfg, language: resolved };
      try {
        await tauriCommands.updateConfig(updatedConfig);
        setConfig(updatedConfig);
      } catch (error) {
        // If persistence fails, keep runtime locale and log
        addLog(`Error saving locale preference: ${formatError(error)}`);
        setConfig(cfg);
      }
    } else {
      setConfig(cfg);
    }

    localeInitialized.current = true;
  };

  const runWithBlockingMessage = async <T,>(
    message: string,
    action: () => Promise<T>,
    delayMs = 250
  ) => {
    let overlayShown = false;
    let timer: number | undefined;

    if (delayMs === 0) {
      overlayShown = true;
      setBlockingMessage(message);
      await new Promise((resolve) => requestAnimationFrame(() => resolve(undefined)));
    } else {
      timer = window.setTimeout(() => {
        overlayShown = true;
        setBlockingMessage(message);
      }, delayMs);
    }

    try {
      // If the overlay is already visible, let it paint once more before heavy work
      if (overlayShown) {
        await new Promise((resolve) => requestAnimationFrame(() => resolve(undefined)));
      }
      return await action();
    } finally {
      if (timer !== undefined) {
        window.clearTimeout(timer);
      }
      if (overlayShown) {
        await new Promise((resolve) => requestAnimationFrame(() => resolve(undefined)));
      }
      setBlockingMessage(null);
    }
  };

  const loadConfig = async () => {
    try {
      const cfg = await tauriCommands.getConfig();
      setGameTargetInput(cfg.target_path ?? '');
      setUserDirInput(cfg.user_dir_path ?? '');

      // Load dark mode preference
      const shouldUseDarkMode = cfg.dark_mode ?? false;
      setDarkMode(shouldUseDarkMode);
      if (shouldUseDarkMode) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }

      await resolveAndApplyLocale(cfg);
      addLog('Configuration loaded');
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

  const loadGraphicsPacks = async () => {
    try {
      const packs = await tauriCommands.listGraphicsPacks();
      setGraphicsPacks(packs);
      if (packs.length > 0) {
        addLog(`Loaded ${packs.length} graphics packs`);
      }
    } catch (error) {
      addLog(`Error loading graphics packs: ${formatError(error)}`);
    }
  };

  const detectGamePath = async () => {
    try {
      setLoading(true);
      addLog('Detecting game path...');
      const paths = await tauriCommands.detectGamePath();

      if (paths.length > 0) {
        await tauriCommands.setGameTarget(paths[0]);
        await loadConfig();
        addLog(`Game path detected: ${paths[0]}`);
      } else {
        addLog('No game installation found');
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
        title: 'Select FM26 Game Target Folder',
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
        title: 'Select FM26 User Directory',
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
        setGameTargetInput(config?.target_path ?? '');
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
        setUserDirInput(config?.user_dir_path ?? '');
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
      addLog('Please set game target first');
      toast.warning('Please set game target first');
      return;
    }

    try {
      setLoading(true);
      await runWithBlockingMessage('Applying mods...', async () => {
        addLog('Applying mods...');
        toast.loading('Applying mods...', { id: 'apply-mods' });
        const result = await tauriCommands.applyMods();
        addLog(result);
        addLog('Mods applied successfully');
        toast.success('Mods applied successfully!', { id: 'apply-mods' });
      });
    } catch (error) {
      addLog(`Error applying mods: ${formatError(error)}`);
      toast.error(`Failed to apply mods: ${formatError(error)}`, {
        id: 'apply-mods',
      });
    } finally {
      setLoading(false);
    }
  };

  const removeMod = async (modId: string) => {
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
    } finally {
      setConfirmDeleteMod(null);
    }
  };

  const handleImportClick = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'Mod Files',
            extensions: ['zip', 'bundle', 'fmf'],
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
      // Check if this is a graphics pack (large ZIP file)
      const isZip = sourcePath.toLowerCase().endsWith('.zip');
      const fileName = sourcePath.split('/').pop()?.toLowerCase() || '';

      // Heuristic: if filename contains graphics-related keywords, treat as graphics pack
      const graphicsKeywords = ['faces', 'logos', 'kits', 'graphics', 'megapack', 'facepack'];
      const isLikelyGraphicsPack = graphicsKeywords.some((keyword) => fileName.includes(keyword));

      if (isZip && isLikelyGraphicsPack) {
        // Route to graphics pack analysis and confirmation
        addLog(`Detected graphics pack: ${sourcePath}`);
        await runWithBlockingMessage(
          'Analyzing graphics pack (this may take a few minutes)...',
          async () => {
            try {
              const analysis = await tauriCommands.analyzeGraphicsPack(sourcePath);
              addLog(`Detected pack type: ${JSON.stringify(analysis.pack_type)}`);
              setPendingGraphicsAnalysis(analysis);
              setPendingGraphicsPath(sourcePath);
            } catch (error) {
              const errorMsg = formatError(error);
              addLog(`Error analyzing graphics pack: ${errorMsg}`);
              toast.error(`Failed to analyze graphics pack: ${errorMsg}`, {
                id: 'analyze-graphics',
              });
            }
          },
          0
        );
        return;
      }

      // Otherwise, import as regular mod
      addLog(`Importing from: ${sourcePath}`);
      await runWithBlockingMessage(
        'Importing mod...',
        async () => {
          toast.loading('Importing mod...', { id: 'import-mod' });
          const result = await tauriCommands.importMod(sourcePath);
          addLog(`Successfully imported: ${result}`);
          toast.success(`Successfully imported: ${result}`, { id: 'import-mod' });
          await loadMods();
        },
        0
      );
    } catch (error) {
      const errorStr = String(error);

      if (errorStr === 'NEEDS_METADATA') {
        toast.dismiss('import-mod');
        // Mod needs metadata - show dialog
        setPendingImportPath(sourcePath);
        setMetadataDialogOpen(true);
        toast.info('Please provide mod metadata');
      } else {
        addLog(`Import failed: ${formatError(error)}`);
        toast.error(`Import failed: ${formatError(error)}`, { id: 'import-mod' });
      }
    }
  };

  const handleMetadataSubmit = async (metadata: ModMetadata) => {
    if (!pendingImportPath) return;

    try {
      addLog(`Importing with metadata...`);
      await runWithBlockingMessage(
        'Importing mod with metadata...',
        async () => {
          toast.loading('Importing mod...', { id: 'import-mod' });
          const result = await tauriCommands.importMod(pendingImportPath, metadata);
          addLog(`Successfully imported: ${result}`);
          toast.success(`Successfully imported: ${result}`, { id: 'import-mod' });
          setMetadataDialogOpen(false);
          setPendingImportPath(null);
          await loadMods();
        },
        0
      );
    } catch (error) {
      addLog(`Import failed: ${formatError(error)}`);
      toast.error(`Import failed: ${formatError(error)}`, { id: 'import-mod' });
    }
  };

  const handleConflictDisable = async (modName: string) => {
    await toggleMod(modName, false);
  };

  const detectUserDirectory = async () => {
    try {
      setLoading(true);
      addLog('Detecting user directory...');
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
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }

    // Save dark mode preference to config
    if (config) {
      const updatedConfig = { ...config, dark_mode: newDarkMode };
      try {
        await tauriCommands.updateConfig(updatedConfig);
        setConfig(updatedConfig);
        addLog(`Dark mode ${newDarkMode ? 'enabled' : 'disabled'}`);
      } catch (error) {
        addLog(`Error saving dark mode preference: ${formatError(error)}`);
      }
    }
  };

  // FM Name Fix handlers
  const checkNameFixStatus = async () => {
    try {
      setCheckingNameFix(true);
      addLog('Checking FM Name Fix installation status...');
      const isInstalled = await tauriCommands.checkNameFixInstalled();
      setNameFixInstalled(isInstalled);

      // Also load the active name fix ID
      const activeId = await tauriCommands.getActiveNameFix();
      setActiveNameFixId(activeId);

      addLog(`FM Name Fix is ${isInstalled ? 'installed' : 'not installed'}`);
    } catch (error) {
      addLog(`Error checking FM Name Fix status: ${formatError(error)}`);
    } finally {
      setCheckingNameFix(false);
    }
  };

  const loadNameFixSources = async () => {
    try {
      const sources = await tauriCommands.listNameFixes();
      setNameFixSources(sources);

      // Set default selection to GitHub if nothing selected
      if (!selectedNameFixId && sources.length > 0) {
        setSelectedNameFixId(sources[0].id);
      }
    } catch (error) {
      addLog(`Error loading name fix sources: ${formatError(error)}`);
    }
  };

  const installSelectedNameFix = async () => {
    if (!selectedNameFixId) {
      toast.error('Please select a name fix to install');
      return;
    }

    try {
      setInstallingNameFix(true);
      const selectedSource = nameFixSources.find((s) => s.id === selectedNameFixId);
      addLog(`Installing ${selectedSource?.name || 'name fix'}...`);
      const result = await tauriCommands.installNameFixById(selectedNameFixId);
      addLog(result);
      toast.success('Name fix installed successfully!');
      setNameFixInstalled(true);
      await checkNameFixStatus();
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error installing name fix: ${errorMsg}`);
      toast.error(`Failed to install name fix: ${errorMsg}`);
    } finally {
      setInstallingNameFix(false);
    }
  };

  const handleImportNameFix = async () => {
    console.log('=== handleImportNameFix START ===');
    try {
      console.log('Opening file picker dialog...');
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'Name Fix Archives',
            extensions: ['zip'],
          },
        ],
      });

      console.log('File picker closed. Selected file:', selected);

      if (!selected) {
        console.log('No file selected, user cancelled file picker');
        return;
      }

      console.log('File was selected, opening name dialog...');

      // Open dialog for user to enter name
      setPendingImportFilePath(selected);
      setImportNameFixName('Custom Name Fix');
      setImportNameFixDialogOpen(true);
    } catch (error) {
      console.error('=== ERROR in handleImportNameFix ===');
      console.error('Error type:', typeof error);
      console.error('Error object:', error);
      const errorMsg = formatError(error);
      console.error('Formatted error message:', errorMsg);
      addLog(`Error importing name fix: ${errorMsg}`);
      toast.error(`Failed to import name fix: ${errorMsg}`);
    }
    console.log('=== handleImportNameFix END ===');
  };

  const confirmImportNameFix = async () => {
    if (!pendingImportFilePath || !importNameFixName.trim()) {
      toast.error('Please enter a name for the name fix');
      return;
    }

    try {
      setImportNameFixDialogOpen(false);
      const name = importNameFixName.trim();

      console.log('Name validated, proceeding with import...');
      addLog(`Importing name fix: ${name}`);
      console.log(
        'Calling tauriCommands.importNameFix with path:',
        pendingImportFilePath,
        'and name:',
        name
      );

      const result = await tauriCommands.importNameFix(pendingImportFilePath, name);
      console.log('Import completed successfully. Result:', result);

      addLog(result);
      toast.success(result);

      // Reload sources
      console.log('Reloading name fix sources...');
      await loadNameFixSources();
      console.log('Sources reloaded successfully');

      // Check installation status in case the imported fix matches what's in the game
      await checkNameFixStatus();

      // Clear state
      setPendingImportFilePath(null);
      setImportNameFixName('');
    } catch (error) {
      console.error('=== ERROR in confirmImportNameFix ===');
      console.error('Error object:', error);
      const errorMsg = formatError(error);
      console.error('Formatted error message:', errorMsg);
      addLog(`Error importing name fix: ${errorMsg}`);
      toast.error(`Failed to import name fix: ${errorMsg}`);
    }
  };

  const handleImportGraphicsPack = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'Graphics Pack Archives',
            extensions: ['zip'],
          },
        ],
      });

      if (!selected) {
        return;
      }

      // Analyze the pack first
      addLog('Analyzing graphics pack...');
      await runWithBlockingMessage(
        'Analyzing graphics pack (this may take a few minutes)...',
        async () => {
          const analysis = await tauriCommands.analyzeGraphicsPack(selected);
          addLog(`Detected pack type: ${JSON.stringify(analysis.pack_type)}`);
          setPendingGraphicsAnalysis(analysis);
          setPendingGraphicsPath(selected);
        },
        0
      );
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error analyzing graphics pack: ${errorMsg}`);
      toast.error(`Failed to analyze graphics pack: ${errorMsg}`, { id: 'analyze-graphics' });
    }
  };

  const handleGraphicsConfirm = async (installPath: string, shouldSplit: boolean) => {
    if (!pendingGraphicsPath || !pendingGraphicsAnalysis) return;

    try {
      await runWithBlockingMessage('Checking graphics pack conflicts...', async () => {
        // Check for conflicts before installing
        const packName = pendingGraphicsPath.split('/').pop()?.replace('.zip', '') || 'Unknown';
        const conflict = await tauriCommands.checkGraphicsConflicts(
          installPath,
          packName,
          pendingGraphicsAnalysis.is_flat_pack
        );

        if (conflict) {
          // Show conflict dialog
          setGraphicsConflict(conflict);
          setPendingInstall({ path: installPath, shouldSplit });
          setShowConflictDialog(true);
          return;
        }

        // No conflict, proceed with installation
        await performGraphicsInstall(installPath, shouldSplit, false);
      });
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error checking conflicts: ${errorMsg}`);
      toast.error(`Failed to check conflicts: ${errorMsg}`);
    }
  };

  const performGraphicsInstall = async (
    installPath: string,
    shouldSplit: boolean,
    force: boolean
  ) => {
    if (!pendingGraphicsPath) return;

    try {
      setImportingGraphics(true);
      addLog(`Installing graphics pack to: ${installPath}`);

      const result = await tauriCommands.importGraphicsPackWithType(
        pendingGraphicsPath,
        installPath,
        shouldSplit,
        force
      );

      addLog(result);
      await loadGraphicsPacks();
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error importing graphics pack: ${errorMsg}`);
      toast.error(`Failed to import graphics pack: ${errorMsg}`, { id: 'graphics-import' });
      setGraphicsProgress(null);
    } finally {
      setImportingGraphics(false);
      setPendingGraphicsAnalysis(null);
      setPendingGraphicsPath(null);
    }
  };

  const handleConflictConfirm = async () => {
    if (!pendingInstall) return;

    setShowConflictDialog(false);
    await performGraphicsInstall(pendingInstall.path, pendingInstall.shouldSplit, true);
    setGraphicsConflict(null);
    setPendingInstall(null);
  };

  const handleConflictCancel = () => {
    setShowConflictDialog(false);
    setGraphicsConflict(null);
    setPendingInstall(null);
    setPendingGraphicsAnalysis(null);
    setPendingGraphicsPath(null);
    addLog('Graphics pack installation cancelled due to conflicts');
  };

  const handleGraphicsCancel = () => {
    setPendingGraphicsAnalysis(null);
    setPendingGraphicsPath(null);
    addLog('Graphics pack import cancelled');
  };

  const handleValidateGraphics = async () => {
    try {
      setValidatingGraphics(true);
      addLog('Validating installed graphics packs...');
      toast.loading('Validating graphics...', { id: 'validate-graphics' });

      const issues = await tauriCommands.validateGraphics();
      setGraphicsIssues(issues);

      if (issues.length === 0) {
        addLog('No issues found - all graphics packs are correctly placed');
        toast.success('All graphics packs are correctly placed!', { id: 'validate-graphics' });
      } else {
        addLog(`Found ${issues.length} graphics pack issue(s)`);
        toast.info(`Found ${issues.length} issue(s). Click to review.`, {
          id: 'validate-graphics',
        });
        setShowValidationDialog(true);
      }
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error validating graphics: ${errorMsg}`);
      toast.error(`Failed to validate graphics: ${errorMsg}`, { id: 'validate-graphics' });
    } finally {
      setValidatingGraphics(false);
    }
  };

  const handleMigrateGraphicsPack = async (packName: string, targetSubdir: string) => {
    try {
      setMigratingPack(true);
      addLog(`Migrating ${packName} to ${targetSubdir}/...`);
      toast.loading(`Moving ${packName}...`, { id: 'migrate-graphics' });

      const result = await tauriCommands.migrateGraphicsPack(packName, targetSubdir);
      addLog(result);
      toast.success('Graphics pack moved successfully!', { id: 'migrate-graphics' });

      // Clear progress and reload validation
      setMigrationProgress(null);
      await handleValidateGraphics();
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error migrating graphics pack: ${errorMsg}`);
      toast.error(`Failed to migrate pack: ${errorMsg}`, { id: 'migrate-graphics' });
      setMigrationProgress(null);
    } finally {
      setMigratingPack(false);
    }
  };

  const handleMigrateAll = async () => {
    if (graphicsIssues.length === 0) return;

    try {
      addLog(`Migrating ${graphicsIssues.length} graphics pack(s)...`);
      toast.loading('Migrating all packs...', { id: 'migrate-all' });

      for (const issue of graphicsIssues) {
        await tauriCommands.migrateGraphicsPack(issue.pack_name, issue.pack_type);
        addLog(`Migrated ${issue.pack_name}`);
      }

      toast.success('All packs migrated successfully!', { id: 'migrate-all' });
      setShowValidationDialog(false);

      // Reload validation
      await handleValidateGraphics();
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error migrating packs: ${errorMsg}`);
      toast.error(`Failed to migrate packs: ${errorMsg}`, { id: 'migrate-all' });
    }
  };

  const handleDeleteNameFix = async (source: NameFixSource) => {
    try {
      addLog(`Deleting ${source.name}...`);
      const result = await tauriCommands.deleteNameFix(source.id);
      addLog(result);
      toast.success(result);

      // Reload sources
      await loadNameFixSources();
      await checkNameFixStatus();

      // Reset selection if deleted source was selected
      if (selectedNameFixId === source.id) {
        setSelectedNameFixId(nameFixSources[0]?.id || '');
      }
    } catch (error) {
      const errorMsg = formatError(error);
      addLog(`Error deleting name fix: ${errorMsg}`);
      toast.error(`Failed to delete name fix: ${errorMsg}`);
    } finally {
      setConfirmDeleteNameFix(null);
    }
  };

  const uninstallNameFix = async () => {
    try {
      setInstallingNameFix(true);
      addLog('Uninstalling FM Name Fix...');
      const result = await tauriCommands.uninstallNameFix();
      addLog(result);
      toast.success('FM Name Fix uninstalled successfully!');
      setNameFixInstalled(false);
      setActiveNameFixId(null);
      await checkNameFixStatus();
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
        await loadGraphicsPacks();
        addLog('FMMLoader26 initialized');

        // Set up Tauri drag and drop event listeners
        const unlistenDrop = await listen<string[]>('tauri://file-drop', (event) => {
          const files = event.payload;
          if (files && files.length > 0) {
            void runWithBlockingMessage('Importing mod...', () => handleImport(files[0]), 0);
          }
          setIsDragging(false);
        });

        const unlistenDragOver = await listen('tauri://drag-over', () => {
          setIsDragging(true);
        });

        const unlistenDragDrop = await listen<{ paths: string[] }>('tauri://drag-drop', (event) => {
          // In Tauri v2, drag-drop contains the file paths
          const paths = event.payload?.paths;
          if (paths && paths.length > 0) {
            void runWithBlockingMessage('Importing mod...', () => handleImport(paths[0]), 0);
          }
          setIsDragging(false);
        });

        const unlistenDragLeave = await listen('tauri://drag-leave', () => {
          setIsDragging(false);
        });

        // Listen for graphics pack extraction progress
        const unlistenGraphicsProgress = await listen<ExtractionProgress>(
          'graphics-extraction-progress',
          (event) => {
            setGraphicsProgress(event.payload);

            if (event.payload.phase === 'complete') {
              setGraphicsProgress(null);
            }
          }
        );

        // Listen for migration progress
        const unlistenMigrationProgress = await listen<ExtractionProgress>(
          'migration-progress',
          (event) => {
            setMigrationProgress(event.payload);

            // Check if migration is complete
            if (event.payload.phase === 'complete') {
              setMigrationProgress(null);
              setMigratingPack(false);
            } else {
              // Update progress state
              const percent = Math.round((event.payload.current / event.payload.total) * 100);
              toast.loading(
                `Migrating: ${event.payload.current}/${event.payload.total} files (${percent}%)`,
                { id: 'migrate-graphics' }
              );
            }
          }
        );

        // Check FM Name Fix installation status and load sources
        try {
          const isInstalled = await tauriCommands.checkNameFixInstalled();
          setNameFixInstalled(isInstalled);

          const activeId = await tauriCommands.getActiveNameFix();
          setActiveNameFixId(activeId);

          await loadNameFixSources();
        } catch (error) {
          // Silently fail - not critical
          console.error('Failed to check FM Name Fix status:', error);
        }

        return () => {
          unlistenDrop();
          unlistenDragOver();
          unlistenDragDrop();
          unlistenDragLeave();
          unlistenGraphicsProgress();
          unlistenMigrationProgress();
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

  // Debug function to preview graphics pack dialog
  useEffect(() => {
    (
      window as unknown as Window & {
        previewGraphicsDialog: (confidenceLevel?: 'high' | 'low' | 'mixed') => void;
      }
    ).previewGraphicsDialog = (confidenceLevel: 'high' | 'low' | 'mixed' = 'high') => {
      const mockData: Record<string, GraphicsPackAnalysis> = {
        high: {
          pack_type: 'Logos',
          confidence: 0.95,
          suggested_paths: ['logos', 'faces', 'kits'],
          file_count: 15420,
          total_size_bytes: 2147483648,
          has_config_xml: true,
          subdirectory_breakdown: {},
          is_flat_pack: false,
        },
        low: {
          pack_type: 'Unknown',
          confidence: 0.35,
          suggested_paths: ['logos', 'faces', 'kits'],
          file_count: 8500,
          total_size_bytes: 1073741824,
          has_config_xml: false,
          subdirectory_breakdown: {},
          is_flat_pack: true,
        },
        mixed: {
          pack_type: { Mixed: ['Faces', 'Logos', 'Kits'] },
          confidence: 0.85,
          suggested_paths: ['faces', 'logos', 'kits'],
          file_count: 25000,
          total_size_bytes: 3221225472,
          has_config_xml: true,
          subdirectory_breakdown: { faces: 12000, logos: 8000, kits: 5000 },
          is_flat_pack: false,
        },
      };
      setPendingGraphicsAnalysis(mockData[confidenceLevel]);
    };
  }, []);

  const graphicsPhaseLabel = graphicsProgress
    ? graphicsProgress.phase === 'copying'
      ? 'Installing graphics pack'
      : graphicsProgress.phase === 'indexing'
        ? 'Indexing files'
        : 'Extracting graphics pack'
    : importingGraphics
      ? 'Finalizing graphics install'
      : null;

  const graphicsPercent =
    graphicsProgress && graphicsProgress.total > 0
      ? Math.round((graphicsProgress.current / graphicsProgress.total) * 100)
      : null;

  const TabsHeader = () => {
    const { t } = useI18n();
    return (
      <div className="border-b px-4 pb-2 flex justify-center">
        <TabsList className="mx-0 mt-0 gap-2">
          <TabsTrigger value="mods">{t('nav.mods')}</TabsTrigger>
          <TabsTrigger value="graphics">{t('nav.graphics')}</TabsTrigger>
          <TabsTrigger value="namefix">{t('nav.nameFix')}</TabsTrigger>
          <TabsTrigger value="settings">{t('nav.settings')}</TabsTrigger>
        </TabsList>
      </div>
    );
  };

  const TranslatedUI = () => {
    const { t } = useI18n();
    const contributors = [
      { name: 'Justin Levine', role: 'Lead / Creator' },
      { name: 'Tom (LotsGon)', role: 'Advice and Expertise' },
      { name: 'Gerko', role: 'Testing and Feedback' },
      { name: 'BassyBoy', role: 'Community Expert / Feedback' },
      { name: 'FM Modders Community', role: 'Inspiration and Support' },
    ];
    // CrowdIn section removed
    return (
      <TooltipProvider>
        <div className="h-screen flex flex-col bg-background" data-locale-ready={localeReady}>
          {/* Custom TitleBar */}
          <TitleBar />

          {/* Update Banner */}
          <UpdateBanner />

          {/* File Drop Zone - covers everything below titlebar */}
          {/* This invisible overlay catches file drops without blocking interactions */}
          <div className="fixed top-12 left-0 right-0 bottom-0 z-[1] pointer-events-none">
            {/* Drag overlay visual feedback */}
            {isDragging && (
              <div className="absolute inset-0 bg-primary/10 border-4 border-dashed border-primary flex items-center justify-center z-40 pointer-events-none">
                <div className="bg-background/95 p-8 rounded-lg shadow-lg">
                  <Upload className="h-16 w-16 mx-auto mb-4 text-primary" />
                  <p className="text-xl font-semibold">{t('overlay.dropTitle')}</p>
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
                      {t('toolbar.import')}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>{t('toolbar.tooltips.import')}</p>
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
                      {t('toolbar.conflicts')}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>{t('toolbar.tooltips.conflicts')}</p>
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
                      {t('toolbar.restore')}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>{t('toolbar.tooltips.restore')}</p>
                  </TooltipContent>
                </Tooltip>

                <Button variant="outline" size="sm" onClick={loadMods} disabled={loading}>
                  <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
                  {t('toolbar.refresh')}
                </Button>
                {process.env.NODE_ENV !== 'production' && (
                  <Button variant="ghost" size="sm" onClick={triggerToastTests}>
                    🔔 Toast test
                  </Button>
                )}
                <Select
                  value={locale}
                  onValueChange={(val) => handleLocaleChange(val as SupportedLocale)}
                >
                  <SelectTrigger className="w-[80px] h-9 justify-center">
                    <div className="w-full text-center" aria-hidden>
                      {localeOptions.find((o) => o.value === locale)?.emoji ?? locale}
                    </div>
                    <SelectValue className="sr-only" />
                  </SelectTrigger>
                  <SelectContent className="w-[40px]">
                    {localeOptions.map((option) => (
                      <SelectItem
                        key={option.value}
                        value={option.value}
                        className="flex items-center justify-center gap-2"
                      >
                        <span className="text-lg">{option.emoji}</span>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setSettingsOpen(true)}
                  aria-label="Credits"
                >
                  <Ellipsis className="h-4 w-4" />
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
                        {t('paths.gameDir.label')}
                      </span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>{t('paths.gameDir.tooltip')}</p>
                    </TooltipContent>
                  </Tooltip>
                  <input
                    type="text"
                    value={gameTargetInput}
                    onChange={(e) => handleGameTargetChange(e.target.value)}
                    onBlur={saveGameTarget}
                    onKeyDown={(e) => e.key === 'Enter' && saveGameTarget()}
                    className="flex-1 px-2 py-1 text-sm font-mono bg-background rounded border border-input focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                    placeholder={t('paths.gameDir.placeholder')}
                    disabled={loading}
                  />
                </div>
                <Button variant="outline" size="sm" onClick={detectGamePath} disabled={loading}>
                  {t('paths.gameDir.detect')}
                </Button>
                <Button variant="outline" size="sm" onClick={selectGamePath} disabled={loading}>
                  <FolderOpen className="h-4 w-4 text-foreground flex-shrink-0" />
                </Button>
              </div>

              <div className="flex items-center gap-2">
                <div className="flex items-center gap-2 flex-1">
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <span className="text-sm text-muted-foreground whitespace-nowrap">
                        {t('paths.userDir.label')}
                      </span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <p>{t('paths.userDir.tooltip')}</p>
                    </TooltipContent>
                  </Tooltip>
                  <input
                    type="text"
                    value={userDirInput}
                    onChange={(e) => handleUserDirChange(e.target.value)}
                    onBlur={saveUserDirectory}
                    onKeyDown={(e) => e.key === 'Enter' && saveUserDirectory()}
                    className="flex-1 px-2 py-1 text-sm font-mono bg-background rounded border border-input focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
                    placeholder={t('paths.userDir.placeholder')}
                    disabled={loading}
                  />
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={detectUserDirectory}
                  disabled={loading}
                >
                  {t('paths.userDir.detect')}
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

          {import.meta.env.VITE_ENABLE_DEBUG_UI === 'true' && (
            <div className="mx-4 mt-4">
              <DebugUI
                metadataDialogOpen={metadataDialogOpen}
                conflictsDialogOpen={conflictsDialogOpen}
                restoreDialogOpen={restoreDialogOpen}
                settingsOpen={settingsOpen}
                modDetailsOpen={modDetailsOpen}
                setMetadataDialogOpen={setMetadataDialogOpen}
                setConflictsDialogOpen={setConflictsDialogOpen}
                setRestoreDialogOpen={setRestoreDialogOpen}
                setSettingsOpen={setSettingsOpen}
                setModDetailsOpen={setModDetailsOpen}
                setSelectedMod={setSelectedMod}
                setPendingGraphicsAnalysis={setPendingGraphicsAnalysis}
                setPendingGraphicsPath={setPendingGraphicsPath}
              />
            </div>
          )}

          {/* Main Content */}
          <div className="flex-1 overflow-hidden">
            <Tabs defaultValue="mods" className="h-full flex flex-col">
              <TabsHeader />

              <TabsContent value="mods" className="flex-1 overflow-hidden m-4 mt-2">
                <ModsTab
                  mods={mods}
                  config={config}
                  loading={loading}
                  onApplyMods={applyMods}
                  onToggleMod={toggleMod}
                  onSelectMod={(mod) => {
                    setSelectedMod(mod);
                    setModDetailsOpen(true);
                  }}
                  onDeleteMod={(modId) => setConfirmDeleteMod(modId)}
                />
              </TabsContent>

              <TabsContent value="graphics" className="flex-1 overflow-hidden m-4 mt-2">
                <GraphicsTab
                  config={config}
                  graphicsPacks={graphicsPacks}
                  importingGraphics={importingGraphics}
                  graphicsProgress={graphicsProgress}
                  validatingGraphics={validatingGraphics}
                  onImportGraphicsPack={handleImportGraphicsPack}
                  onValidateGraphics={handleValidateGraphics}
                />
              </TabsContent>

              <TabsContent value="namefix" className="flex-1 overflow-hidden m-4 mt-2">
                <NameFixTab
                  config={config}
                  nameFixInstalled={nameFixInstalled}
                  checkingNameFix={checkingNameFix}
                  installingNameFix={installingNameFix}
                  nameFixSources={nameFixSources}
                  activeNameFixId={activeNameFixId}
                  selectedNameFixId={selectedNameFixId}
                  onSelectNameFix={setSelectedNameFixId}
                  onInstall={installSelectedNameFix}
                  onUninstall={uninstallNameFix}
                  onImport={handleImportNameFix}
                  onCheckStatus={checkNameFixStatus}
                  onDeleteSource={(source) => setConfirmDeleteNameFix(source)}
                />
              </TabsContent>

              <TabsContent value="settings" className="flex-1 overflow-hidden m-4 mt-2">
                <SettingsTab
                  darkMode={darkMode}
                  onToggleDarkMode={toggleDarkMode}
                  addLog={addLog}
                  locale={locale}
                  onLocaleChange={handleLocaleChange}
                />
              </TabsContent>
            </Tabs>
          </div>

          {/* Footer */}
          <div className="border-t bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 p-3 flex items-center justify-between">
            <div className="text-xs text-muted-foreground font-medium">
              {t('footer.createdBy', { version: appVersion })}
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => openUrl('https://ko-fi.com/jalco')}
                className="hover:bg-[#FF5E5B] hover:text-white hover:border-[#FF5E5B] transition-colors"
              >
                <SiKofi className="mr-2 h-4 w-4" />
                {t('footer.supportKofi')}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => openUrl('https://discord.gg/AspRvTTAch')}
                className="hover:bg-[#5865F2] hover:text-white hover:border-[#5865F2] transition-colors"
              >
                <FaDiscord className="mr-2 h-4 w-4" />
                {t('footer.joinDiscord')}
              </Button>
            </div>
          </div>

          {/* Dialogs */}
          <ModMetadataDialog
            open={metadataDialogOpen}
            onOpenChange={setMetadataDialogOpen}
            sourcePath={pendingImportPath ?? ''}
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
              addLog('Restored from backup');
            }}
          />

          {/* Graphics Pack Confirmation Dialog */}
          <GraphicsPackConfirmDialog
            key={
              pendingGraphicsPath ??
              pendingGraphicsAnalysis?.suggested_paths.join('|') ??
              'graphics-dialog'
            }
            analysis={pendingGraphicsAnalysis}
            onConfirm={handleGraphicsConfirm}
            onCancel={handleGraphicsCancel}
            userDirPath={config?.user_dir_path}
          />

          {/* Graphics Conflict Confirmation Dialog */}
          <Dialog open={showConflictDialog} onOpenChange={setShowConflictDialog}>
            <DialogContent>
              <DialogHeader>
                <DialogTitle className="flex items-center gap-2">
                  <AlertTriangle className="h-5 w-5 text-amber-500" />
                  Confirm Overwrite
                </DialogTitle>
                <DialogDescription>
                  Files exist in the target directory. This action may overwrite existing graphics.
                </DialogDescription>
              </DialogHeader>

              {graphicsConflict && (
                <div className="space-y-3 my-4">
                  <div className="text-sm">
                    <p className="mb-2">
                      There are currently <strong>{graphicsConflict.existing_file_count}</strong>{' '}
                      file(s) in:
                    </p>
                    <div className="bg-muted p-2 rounded text-muted-foreground font-mono text-xs">
                      {graphicsConflict.target_directory}
                    </div>
                  </div>

                  <div className="bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-800 p-3 rounded-md">
                    <p className="text-sm text-amber-900 dark:text-amber-200">
                      Installing <strong>{graphicsConflict.pack_name}</strong> may replace or merge
                      with existing graphics files.
                    </p>
                  </div>

                  <p className="text-sm text-muted-foreground">
                    Are you sure you want to continue?
                  </p>
                </div>
              )}

              <DialogFooter>
                <Button variant="outline" onClick={handleConflictCancel}>
                  Cancel
                </Button>
                <Button onClick={handleConflictConfirm} variant="default">
                  Continue Installation
                </Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>

          {/* Graphics Validation Dialog */}
          <Dialog open={showValidationDialog} onOpenChange={setShowValidationDialog}>
            <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
              <DialogHeader>
                <DialogTitle>Graphics Pack Validation Results</DialogTitle>
                <DialogDescription>
                  Found {graphicsIssues.length} pack(s) that may need to be moved
                </DialogDescription>
              </DialogHeader>

              {migrationProgress && (
                <div className="text-sm bg-muted p-3 rounded-md space-y-2 mb-4">
                  <div className="flex justify-between">
                    <strong>Migration Progress:</strong>
                    <span>
                      {Math.round((migrationProgress.current / migrationProgress.total) * 100)}%
                    </span>
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {migrationProgress.current} / {migrationProgress.total} files
                  </div>
                  <div className="w-full bg-secondary rounded-full h-2">
                    <div
                      className="bg-primary h-2 rounded-full transition-all"
                      style={{
                        width: `${(migrationProgress.current / migrationProgress.total) * 100}%`,
                      }}
                    />
                  </div>
                </div>
              )}

              <div className="space-y-4 my-4">
                {graphicsIssues.map((issue, index) => (
                  <Card key={index} className="border-amber-200 dark:border-amber-800">
                    <CardContent className="pt-4">
                      <div className="space-y-3">
                        <div className="flex items-start gap-2">
                          <AlertTriangle className="h-5 w-5 text-amber-500 mt-0.5" />
                          <div className="flex-1">
                            <div className="font-semibold">{issue.pack_name}</div>
                            <div className="text-sm text-muted-foreground mt-1">{issue.reason}</div>
                          </div>
                        </div>

                        <div className="grid grid-cols-2 gap-3 text-sm bg-muted p-3 rounded">
                          <div>
                            <div className="font-medium">Current:</div>
                            <div className="text-muted-foreground">{issue.current_path}</div>
                          </div>
                          <div>
                            <div className="font-medium">Suggested:</div>
                            <div className="text-green-600 dark:text-green-400">
                              {issue.suggested_path}
                            </div>
                          </div>
                        </div>

                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() =>
                            void handleMigrateGraphicsPack(issue.pack_name, issue.pack_type)
                          }
                          disabled={migratingPack}
                          className="w-full"
                        >
                          {migratingPack ? (
                            <>
                              <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                              Moving...
                            </>
                          ) : (
                            'Move to Correct Location'
                          )}
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>

              <DialogFooter>
                <Button
                  variant="outline"
                  onClick={() => setShowValidationDialog(false)}
                  disabled={migratingPack}
                >
                  Close
                </Button>
                {graphicsIssues.length > 0 && (
                  <Button onClick={() => void handleMigrateAll()} disabled={migratingPack}>
                    Fix All ({graphicsIssues.length})
                  </Button>
                )}
              </DialogFooter>
            </DialogContent>
          </Dialog>

          {/* Mod Details Sheet */}
          <Sheet open={modDetailsOpen} onOpenChange={setModDetailsOpen}>
            <SheetContent className="w-[400px] sm:w-[540px]">
              <SheetHeader>
                <SheetTitle>{selectedMod?.name ?? 'Mod Details'}</SheetTitle>
                <SheetDescription>
                  {selectedMod ? `Version ${selectedMod.version}` : 'Select a mod to view details'}
                </SheetDescription>
              </SheetHeader>
              {selectedMod && (
                <div className="mt-6 space-y-4">
                  <div className="space-y-2">
                    <div>
                      <span className="text-sm font-medium">Author:</span>
                      <p className="text-sm text-muted-foreground">
                        {selectedMod.author || 'Unknown'}
                      </p>
                    </div>

                    <div>
                      <span className="text-sm font-medium">Type:</span>
                      <p className="text-sm text-muted-foreground">{selectedMod.mod_type}</p>
                    </div>

                    <div>
                      <span className="text-sm font-medium">Description:</span>
                      <p className="text-sm text-muted-foreground">
                        {selectedMod.description || 'No description available'}
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
                      variant={selectedMod.enabled ? 'destructive' : 'default'}
                      onClick={() => toggleMod(selectedMod.id, !selectedMod.enabled)}
                    >
                      {selectedMod.enabled ? 'Disable Mod' : 'Enable Mod'}
                    </Button>
                    <Button
                      className="w-full"
                      variant="outline"
                      onClick={() => setConfirmDeleteMod(selectedMod.id)}
                    >
                      <Trash2 className="mr-2 h-4 w-4" />
                      Remove Mod
                    </Button>
                  </div>
                </div>
              )}
            </SheetContent>
          </Sheet>

          {/* Credits Sheet (repurposed from Settings) */}
          <Sheet open={settingsOpen} onOpenChange={setSettingsOpen}>
            <SheetContent>
              <SheetHeader>
                <SheetTitle>{t('credits.title')}</SheetTitle>
                <SheetDescription>{t('credits.description')}</SheetDescription>
              </SheetHeader>
              <div className="mt-6 space-y-6">
                <div className="space-y-2">
                  <div className="text-sm font-semibold">
                    {t('credits.sections.contributors.title')}
                  </div>
                  <div className="space-y-1 text-sm text-muted-foreground">
                    {contributors.map((person) => (
                      <div key={person.name} className="flex justify-between">
                        <span>{person.name}</span>
                        <span className="text-xs uppercase tracking-wide text-muted-foreground/80">
                          {person.role}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>

                <div className="space-y-2 border-t pt-4">
                  <div className="text-sm font-semibold">
                    {t('credits.sections.translators.title')}
                  </div>
                  <div className="text-sm text-muted-foreground flex flex-wrap gap-2">
                    {localeOptions.map((opt) => (
                      <span
                        key={opt.value}
                        className="rounded-full border px-2 py-1 text-xs bg-muted/40 border-border"
                      >
                        {opt.emoji} {opt.contributor ?? opt.label}
                      </span>
                    ))}
                  </div>
                </div>
              </div>
            </SheetContent>
          </Sheet>

          {/* Keep graphics pack progress visible; only show blocker for mod-oriented flows */}
          {blockingMessage && !importingGraphics && !graphicsProgress && (
            <div className="fixed inset-0 z-[9999] flex flex-col items-center justify-center gap-4 bg-background/70 p-6 text-center backdrop-blur-sm">
              <div className="flex flex-col items-center gap-6">
                <div
                  className="h-12 w-12 animate-spin rounded-full border-4 border-muted-foreground/30 border-t-primary"
                  aria-hidden="true"
                />
                <div className="space-y-2">
                  <p className="text-xl font-semibold">{blockingMessage}</p>
                  <p className="text-sm text-muted-foreground">
                    Large imports can take a moment. Keep the window open until it finishes.
                  </p>
                </div>
              </div>
            </div>
          )}

          {/* Graphics pack overlay with inline progress (replaces repeated toasts) */}
          {(importingGraphics || graphicsProgress) && (
            <div className="fixed inset-0 z-[9998] flex flex-col items-center justify-center gap-4 bg-background/70 p-6 text-center backdrop-blur-sm">
              <div className="flex flex-col items-center gap-6">
                <div
                  className="h-12 w-12 animate-spin rounded-full border-4 border-muted-foreground/30 border-t-primary"
                  aria-hidden="true"
                />
                <div className="space-y-2">
                  <p className="text-xl font-semibold">
                    {graphicsPhaseLabel ?? 'Processing graphics pack'}
                    {graphicsPercent !== null ? ` • ${graphicsPercent}%` : ''}
                  </p>
                  {graphicsProgress && (
                    <p className="text-sm text-muted-foreground">
                      {graphicsProgress.current} / {graphicsProgress.total} files
                    </p>
                  )}
                  <p className="text-sm text-muted-foreground">
                    Large graphics packs can take a few minutes. Keep the window open until it
                    finishes.
                  </p>
                </div>
              </div>
              {graphicsPercent !== null && (
                <div className="w-80 max-w-[90vw]">
                  <div className="w-full bg-secondary rounded-full h-2 overflow-hidden">
                    <div
                      className="bg-primary h-2 rounded-full transition-all"
                      style={{ width: `${graphicsPercent}%` }}
                    />
                  </div>
                </div>
              )}
            </div>
          )}
          <Toaster />

          {/* Import Name Fix Dialog */}
          <Dialog open={importNameFixDialogOpen} onOpenChange={setImportNameFixDialogOpen}>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Import Name Fix</DialogTitle>
                <DialogDescription>Enter a name for this name fix package</DialogDescription>
              </DialogHeader>
              <div className="grid gap-4 py-4">
                <div className="grid gap-2">
                  <Label htmlFor="namefix-name">Name</Label>
                  <Input
                    id="namefix-name"
                    value={importNameFixName}
                    onChange={(e) => setImportNameFixName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        void confirmImportNameFix();
                      }
                    }}
                    placeholder="e.g., Real Names Fix v1.0"
                    autoFocus
                  />
                </div>
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={() => setImportNameFixDialogOpen(false)}>
                  Cancel
                </Button>
                <Button onClick={() => void confirmImportNameFix()}>Import</Button>
              </DialogFooter>
            </DialogContent>
          </Dialog>

          {/* Delete Mod Confirmation Dialog */}
          <AlertDialog
            open={confirmDeleteMod !== null}
            onOpenChange={(open) => !open && setConfirmDeleteMod(null)}
          >
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Delete Mod?</AlertDialogTitle>
                <AlertDialogDescription>
                  This will permanently remove the mod "
                  {mods.find((m) => m.id === confirmDeleteMod)?.name || confirmDeleteMod}" from your
                  library. This action cannot be undone.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction
                  onClick={() => {
                    if (confirmDeleteMod) {
                      void removeMod(confirmDeleteMod);
                    }
                    setConfirmDeleteMod(null);
                  }}
                >
                  Delete
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>

          {/* Delete Name Fix Confirmation Dialog */}
          <AlertDialog
            open={confirmDeleteNameFix !== null}
            onOpenChange={(open) => !open && setConfirmDeleteNameFix(null)}
          >
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Delete Name Fix Source?</AlertDialogTitle>
                <AlertDialogDescription>
                  This will permanently remove "{confirmDeleteNameFix?.name}" from your available
                  name fix sources. This action cannot be undone.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction
                  onClick={() => {
                    if (confirmDeleteNameFix) {
                      void handleDeleteNameFix(confirmDeleteNameFix);
                    }
                    setConfirmDeleteNameFix(null);
                  }}
                >
                  Delete
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </TooltipProvider>
    );
  };

  return (
    <I18nProvider locale={locale} fallbackLocale="en" onLocaleChange={setLocale}>
      <TranslatedUI />
    </I18nProvider>
  );
}

export default App;
