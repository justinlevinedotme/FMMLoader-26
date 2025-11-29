import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { FolderOpen, Settings } from 'lucide-react';
import { useI18n, type SupportedLocale } from '@/lib/i18n';
import { tauriCommands } from '@/hooks/useTauri';

interface SettingsTabProps {
  darkMode: boolean;
  onToggleDarkMode: () => void;
  addLog: (message: string) => void;
  locale: SupportedLocale;
  onLocaleChange: (locale: SupportedLocale) => void;
  updateStatus: {
    checking: boolean;
    available: boolean;
    currentVersion: string | null;
    latestVersion: string | null;
  };
  onCheckForUpdates: () => void;
}

const formatError = (error: unknown): string => {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return String(error);
};

export function SettingsTab({
  darkMode,
  onToggleDarkMode,
  addLog,
  locale,
  onLocaleChange,
  updateStatus,
  onCheckForUpdates,
}: SettingsTabProps) {
  const { t } = useI18n();
  const localeOptions: { value: SupportedLocale; label: string }[] = [
    { value: 'en', label: 'English (US)' },
    { value: 'en-GB', label: 'English (UK)' },
    { value: 'ko', label: '한국어' },
    { value: 'tr', label: 'Türkçe' },
    { value: 'pt-PT', label: 'Português (Portugal)' },
    { value: 'de', label: 'Deutsch' },
    { value: 'it', label: 'Italiano' },
    { value: 'nl', label: 'Nederlands' },
  ];

  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Settings className="h-5 w-5" />
          {t('settings.title')}
        </CardTitle>
        <CardDescription>{t('settings.description')}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto space-y-6">
        <div className="flex items-center justify-between">
          <div className="space-y-1">
            <div className="text-sm font-medium">{t('settings.darkMode')}</div>
            <div className="text-sm text-muted-foreground">{t('settings.darkModeDescription')}</div>
          </div>
          <Switch checked={darkMode} onCheckedChange={onToggleDarkMode} />
        </div>

        <div className="space-y-2 border-t pt-4">
          <div className="text-sm font-medium">{t('settings.logs.title')}</div>
          <div className="text-sm text-muted-foreground">{t('settings.logs.description')}</div>
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
            {t('settings.logs.open')}
          </Button>
        </div>

        <div className="space-y-2 border-t pt-4">
          <div className="text-sm font-medium">{t('settings.modsStorage.title')}</div>
          <div className="text-sm text-muted-foreground">
            {t('settings.modsStorage.description')}
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
            {t('settings.modsStorage.open')}
          </Button>
        </div>

        <div className="space-y-2 border-t pt-4">
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <div className="text-sm font-medium">{t('settings.updates.title')}</div>
              <div className="text-sm text-muted-foreground">
                {updateStatus.available
                  ? t('settings.updates.available', {
                      version: updateStatus.latestVersion ?? 'unknown',
                    })
                  : t('settings.updates.current', {
                      version: updateStatus.currentVersion ?? 'unknown',
                    })}
              </div>
            </div>
            <Button
              variant="outline"
              size="sm"
              disabled={updateStatus.checking}
              onClick={onCheckForUpdates}
            >
              {updateStatus.checking ? t('settings.updates.checking') : t('settings.updates.check')}
            </Button>
          </div>
        </div>

        <div className="space-y-2 border-t pt-4">
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <div className="text-sm font-medium">{t('settings.language.title')}</div>
              <div className="text-sm text-muted-foreground">
                {t('settings.language.description')}
              </div>
            </div>
          </div>
          <Select value={locale} onValueChange={(val) => onLocaleChange(val as SupportedLocale)}>
            <SelectTrigger className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {localeOptions.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </CardContent>
    </Card>
  );
}
