import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { RefreshCw, Download, Trash2, Upload, Wrench, CheckCircle2, XCircle } from 'lucide-react';
import { useI18n } from '@/lib/i18n';
import type { Config, NameFixSource } from '@/types';

interface NameFixTabProps {
  config: Config | null;
  nameFixInstalled: boolean;
  checkingNameFix: boolean;
  installingNameFix: boolean;
  nameFixSources: NameFixSource[];
  activeNameFixId: string | null;
  selectedNameFixId: string;
  onSelectNameFix: (id: string) => void;
  onInstall: () => void;
  onUninstall: () => void;
  onImport: () => void;
  onCheckStatus: () => void;
  onDeleteSource: (source: NameFixSource) => void;
}

export function NameFixTab({
  config,
  nameFixInstalled,
  checkingNameFix,
  installingNameFix,
  nameFixSources,
  activeNameFixId,
  selectedNameFixId,
  onSelectNameFix,
  onInstall,
  onUninstall,
  onImport,
  onCheckStatus,
  onDeleteSource,
}: NameFixTabProps) {
  const { t } = useI18n();
  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Wrench className="h-5 w-5" />
          {t('nameFix.title')}
        </CardTitle>
        <CardDescription>{t('nameFix.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto space-y-4">
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <CardTitle className="text-lg">{t('nameFix.sectionTitle')}</CardTitle>
                <CardDescription className="mt-1">
                  {t('nameFix.sectionDescription')}
                </CardDescription>
              </div>
              <div className="flex items-center gap-2">
                {checkingNameFix ? (
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <RefreshCw className="h-4 w-4 animate-spin" />
                    {t('nameFix.status.checking')}
                  </div>
                ) : nameFixInstalled ? (
                  <div className="flex items-center gap-2 text-sm text-green-600 dark:text-green-400">
                    <CheckCircle2 className="h-4 w-4" />
                    {t('nameFix.status.installed')}
                  </div>
                ) : (
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <XCircle className="h-4 w-4" />
                    {t('nameFix.status.notInstalled')}
                  </div>
                )}
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="text-sm text-muted-foreground space-y-2">
              <p>
                <strong>{t('nameFix.whatItDoes.heading')}</strong>
              </p>
              <ul className="list-disc list-inside space-y-1 ml-2">
                <li>{t('nameFix.whatItDoes.bullets.0')}</li>
                <li>{t('nameFix.whatItDoes.bullets.1')}</li>
                <li>{t('nameFix.whatItDoes.bullets.2')}</li>
                <li>{t('nameFix.whatItDoes.bullets.3')}</li>
              </ul>
            </div>

            {activeNameFixId && (
              <div className="text-sm bg-muted p-3 rounded-md">
                <strong>{t('nameFix.active.heading')}</strong>{' '}
                {nameFixSources.find((s) => s.id === activeNameFixId)?.name ||
                  t('nameFix.active.unknown')}
              </div>
            )}

            {nameFixSources.length > 0 ? (
              <div className="space-y-2">
                <label className="text-sm font-medium">{t('nameFix.selectLabel')}</label>
                <Select
                  value={selectedNameFixId}
                  onValueChange={onSelectNameFix}
                  disabled={installingNameFix}
                >
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder={t('nameFix.selectPlaceholder')} />
                  </SelectTrigger>
                  <SelectContent>
                    {nameFixSources.map((source) => (
                      <SelectItem key={source.id} value={source.id}>
                        {source.name}
                        {source.id === activeNameFixId ? t('nameFix.activeSuffix') : ''}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  {nameFixSources.find((s) => s.id === selectedNameFixId)?.description}
                </p>
              </div>
            ) : (
              <div className="text-sm bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-800 p-3 rounded-md">
                <p className="text-amber-800 dark:text-amber-200">{t('nameFix.noSources')}</p>
              </div>
            )}

            <div className="flex flex-wrap gap-2">
              <Button
                onClick={() => void onInstall()}
                disabled={
                  installingNameFix ||
                  !config?.target_path ||
                  !selectedNameFixId ||
                  nameFixSources.length === 0 ||
                  nameFixInstalled
                }
              >
                {installingNameFix ? (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    {t('nameFix.status.checking')}
                  </>
                ) : (
                  <>
                    <Download className="mr-2 h-4 w-4" />
                    {nameFixInstalled
                      ? t('nameFix.buttons.reinstall')
                      : t('nameFix.buttons.install')}
                  </>
                )}
              </Button>

              {nameFixInstalled && (
                <Button
                  variant="destructive"
                  onClick={() => void onUninstall()}
                  disabled={installingNameFix}
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  {t('nameFix.buttons.uninstall')}
                </Button>
              )}

              <Button
                variant="outline"
                onClick={() => void onImport()}
                disabled={installingNameFix}
              >
                <Upload className="mr-2 h-4 w-4" />
                {t('nameFix.buttons.importZip')}
              </Button>

              <Button
                variant="outline"
                onClick={() => void onCheckStatus()}
                disabled={checkingNameFix}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                {t('nameFix.buttons.checkStatus')}
              </Button>

              {selectedNameFixId && nameFixSources.length > 0 && (
                <Button
                  variant="outline"
                  onClick={() => {
                    const source = nameFixSources.find((s) => s.id === selectedNameFixId);
                    if (source) onDeleteSource(source);
                  }}
                  disabled={installingNameFix || selectedNameFixId === activeNameFixId}
                >
                  <Trash2 className="mr-2 h-4 w-4" />
                  {t('nameFix.buttons.deleteSource')}
                </Button>
              )}
            </div>
            {!config?.target_path && (
              <p className="text-sm text-amber-600 dark:text-amber-400">
                {t('nameFix.warnings.needGameDir')}
              </p>
            )}

            <p className="text-xs text-muted-foreground mt-2">{t('nameFix.notes.community')}</p>
          </CardContent>
        </Card>
      </CardContent>
    </Card>
  );
}
