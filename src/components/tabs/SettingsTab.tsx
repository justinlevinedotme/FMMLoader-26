import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Switch } from '@/components/ui/switch';
import { FolderOpen, Settings } from 'lucide-react';
import { tauriCommands } from '@/hooks/useTauri';

interface SettingsTabProps {
  darkMode: boolean;
  onToggleDarkMode: () => void;
  addLog: (message: string) => void;
}

const formatError = (error: unknown): string => {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return String(error);
};

export function SettingsTab({ darkMode, onToggleDarkMode, addLog }: SettingsTabProps) {
  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Settings className="h-5 w-5" />
          Settings
        </CardTitle>
        <CardDescription>Theme and storage preferences</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto space-y-6">
        <div className="flex items-center justify-between">
          <div className="space-y-1">
            <div className="text-sm font-medium">Dark Mode</div>
            <div className="text-sm text-muted-foreground">
              Toggle dark theme for the app window.
            </div>
          </div>
          <Switch checked={darkMode} onCheckedChange={onToggleDarkMode} />
        </div>

        <div className="space-y-2 border-t pt-4">
          <div className="text-sm font-medium">Application Logs</div>
          <div className="text-sm text-muted-foreground">
            Opens the folder where recent app logs are stored.
          </div>
          <Button
            variant="outline"
            className="w-full mt-2"
            onClick={async () => {
              try {
                await tauriCommands.openLogsFolder();
                addLog('Opened logs folder');
              } catch (error) {
                addLog(`Failed to open logs folder: ${formatError(error)}`);
              }
            }}
          >
            <FolderOpen className="mr-2 h-4 w-4" />
            Open Logs Folder
          </Button>
        </div>

        <div className="space-y-2 border-t pt-4">
          <div className="text-sm font-medium">Mods Storage</div>
          <div className="text-sm text-muted-foreground">
            Opens the folder where imported mods are stored.
          </div>
          <Button
            variant="outline"
            className="w-full mt-2"
            onClick={async () => {
              try {
                await tauriCommands.openModsFolder();
                addLog('Opened mods folder');
              } catch (error) {
                addLog(`Failed to open mods folder: ${formatError(error)}`);
              }
            }}
          >
            <FolderOpen className="mr-2 h-4 w-4" />
            Open Mods Folder
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
