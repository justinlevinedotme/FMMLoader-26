import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { RefreshCw, Upload, CheckCircle2 } from 'lucide-react';
import { useI18n } from '@/lib/i18n';
import type { Config, ExtractionProgress, GraphicsPackMetadata } from '@/types';

interface GraphicsTabProps {
  config: Config | null;
  graphicsPacks: GraphicsPackMetadata[];
  importingGraphics: boolean;
  graphicsProgress: ExtractionProgress | null;
  validatingGraphics: boolean;
  onImportGraphicsPack: () => void;
  onValidateGraphics: () => void;
}

export function GraphicsTab({
  config,
  graphicsPacks,
  importingGraphics,
  graphicsProgress,
  validatingGraphics,
  onImportGraphicsPack,
  onValidateGraphics,
}: GraphicsTabProps) {
  const { t } = useI18n();
  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>{t('graphicsTab.title')}</CardTitle>
            <CardDescription>
              {t('graphicsTab.subtitle', { count: graphicsPacks.length })}
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Button
              onClick={() => void onImportGraphicsPack()}
              disabled={importingGraphics || !config?.user_dir_path}
            >
              <Upload className="mr-2 h-4 w-4" />
              {t('graphicsTab.import.button')}
            </Button>
            <Button
              variant="outline"
              onClick={() => void onValidateGraphics()}
              disabled={validatingGraphics || !config?.user_dir_path}
            >
              <CheckCircle2 className="mr-2 h-4 w-4" />
              {t('graphicsTab.validate.button')}
            </Button>
          </div>
        </div>
        {importingGraphics && (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <RefreshCw className="h-4 w-4 animate-spin" />
            {t('graphicsTab.progress.importing')}
          </div>
        )}
        {graphicsProgress && (
          <div className="text-sm bg-muted p-3 rounded-md space-y-2">
            <div className="flex justify-between">
              <strong>{t('graphicsTab.progress.heading')}</strong>
              <span>{Math.round((graphicsProgress.current / graphicsProgress.total) * 100)}%</span>
            </div>
            <div className="text-xs text-muted-foreground">
              {t('graphicsTab.progress.files', {
                current: graphicsProgress.current,
                total: graphicsProgress.total,
              })}
            </div>
            <div className="w-full bg-secondary rounded-full h-2">
              <div
                className="bg-primary h-2 rounded-full transition-all"
                style={{
                  width: `${(graphicsProgress.current / graphicsProgress.total) * 100}%`,
                }}
              />
            </div>
          </div>
        )}
        {!config?.user_dir_path && (
          <p className="text-sm text-amber-600 dark:text-amber-400">
            {t('graphicsTab.warnings.needUserDir')}
          </p>
        )}
      </CardHeader>
      <CardContent className="flex-1 overflow-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{t('graphicsTab.table.name')}</TableHead>
              <TableHead>{t('graphicsTab.table.type')}</TableHead>
              <TableHead>{t('graphicsTab.table.files')}</TableHead>
              <TableHead>{t('graphicsTab.table.installed')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {graphicsPacks.map((pack) => (
              <TableRow key={pack.id} className="hover:bg-muted/50">
                <TableCell className="font-medium">{pack.name}</TableCell>
                <TableCell>
                  <span className="inline-flex items-center rounded-md bg-purple-50 px-2 py-1 text-xs font-medium text-purple-700 ring-1 ring-inset ring-purple-700/10">
                    {pack.pack_type}
                  </span>
                </TableCell>
                <TableCell className="text-muted-foreground text-xs">
                  {pack.file_count} files
                </TableCell>
                <TableCell className="text-muted-foreground text-xs">
                  {new Date(pack.install_date).toLocaleDateString()}
                </TableCell>
              </TableRow>
            ))}
            {graphicsPacks.length === 0 && (
              <TableRow>
                <TableCell colSpan={4} className="text-center text-muted-foreground">
                  {t('graphicsTab.table.empty')}
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}
