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
  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Wrench className="h-5 w-5" />
          Name Fix
        </CardTitle>
        <CardDescription>Manage FM Name Fix</CardDescription>
      </CardHeader>
      <CardContent className="flex-1 overflow-auto space-y-4">
        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div className="flex-1">
                <CardTitle className="text-lg">FM Name Fix</CardTitle>
                <CardDescription className="mt-1">
                  Fixes licensing issues and unlocks real names for clubs, players, and competitions
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
                <li>Unlocks real names for clubs like AC Milan, Inter, and Lazio</li>
                <li>Fixes Japanese player names</li>
                <li>Removes fake/unlicensed content</li>
                <li>Works with all leagues and competitions</li>
              </ul>
            </div>

            {activeNameFixId && (
              <div className="text-sm bg-muted p-3 rounded-md">
                <strong>Currently Active:</strong>{' '}
                {nameFixSources.find((s) => s.id === activeNameFixId)?.name || 'Unknown'}
              </div>
            )}

            {nameFixSources.length > 0 ? (
              <div className="space-y-2">
                <label className="text-sm font-medium">Select Name Fix Source:</label>
                <Select
                  value={selectedNameFixId}
                  onValueChange={onSelectNameFix}
                  disabled={installingNameFix}
                >
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select a name fix source" />
                  </SelectTrigger>
                  <SelectContent>
                    {nameFixSources.map((source) => (
                      <SelectItem key={source.id} value={source.id}>
                        {source.name}
                        {source.id === activeNameFixId ? ' - Active' : ''}
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
                <p className="text-amber-800 dark:text-amber-200">
                  No name fix sources available. Import a name fix ZIP file to get started.
                </p>
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
                    Installing...
                  </>
                ) : (
                  <>
                    <Download className="mr-2 h-4 w-4" />
                    {nameFixInstalled ? 'Reinstall' : 'Install'}
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
                  Uninstall from Game
                </Button>
              )}

              <Button
                variant="outline"
                onClick={() => void onImport()}
                disabled={installingNameFix}
              >
                <Upload className="mr-2 h-4 w-4" />
                Import from ZIP
              </Button>

              <Button
                variant="outline"
                onClick={() => void onCheckStatus()}
                disabled={checkingNameFix}
              >
                <RefreshCw className="mr-2 h-4 w-4" />
                Check Status
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
                  Delete Source
                </Button>
              )}
            </div>
            {!config?.target_path && (
              <p className="text-sm text-amber-600 dark:text-amber-400">
                Please set your Game Directory first
              </p>
            )}

            <p className="text-xs text-muted-foreground mt-2">
              You can download name fixes from the community or create your own. We recommend fixes
              from our friends at SortItOutSI or FMScout
            </p>
          </CardContent>
        </Card>
      </CardContent>
    </Card>
  );
}
