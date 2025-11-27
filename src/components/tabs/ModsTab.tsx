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
import { Switch } from '@/components/ui/switch';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { Download, Trash2 } from 'lucide-react';
import type { ModManifest, Config } from '@/types';

export interface ModWithInfo extends ModManifest {
  id: string;
  enabled: boolean;
}

interface ModsTabProps {
  mods: ModWithInfo[];
  config: Config | null;
  loading: boolean;
  onApplyMods: () => void;
  onToggleMod: (modId: string, enabled: boolean) => void;
  onSelectMod: (mod: ModWithInfo) => void;
  onDeleteMod: (modId: string) => void;
}

export function ModsTab({
  mods,
  config,
  loading,
  onApplyMods,
  onToggleMod,
  onSelectMod,
  onDeleteMod,
}: ModsTabProps) {
  return (
    <div className="h-full">
      <Card className="flex flex-col h-full">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Installed Mods</CardTitle>
              <CardDescription>{mods.length} mods installed</CardDescription>
            </div>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button onClick={onApplyMods} disabled={loading || !config?.target_path}>
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
                  onClick={() => onSelectMod(mod)}
                >
                  <TableCell>
                    <Switch
                      checked={mod.enabled}
                      onCheckedChange={(checked: boolean) => {
                        void onToggleMod(mod.id, checked);
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
                        onDeleteMod(mod.id);
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
  );
}
