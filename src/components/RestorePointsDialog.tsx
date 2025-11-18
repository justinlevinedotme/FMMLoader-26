import { useEffect, useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { History, Undo2 } from 'lucide-react';
import { tauriCommands, type RestorePoint } from '@/hooks/useTauri';

interface RestorePointsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onRestore?: () => void;
}

export function RestorePointsDialog({ open, onOpenChange, onRestore }: RestorePointsDialogProps) {
  const [restorePoints, setRestorePoints] = useState<RestorePoint[]>([]);
  const [loading, setLoading] = useState(false);
  const [creating, setCreating] = useState(false);
  const [newPointName, setNewPointName] = useState('');
  const [showCreateForm, setShowCreateForm] = useState(false);

  useEffect(() => {
    if (open) {
      void loadRestorePoints();
    }
  }, [open]);

  const loadRestorePoints = async () => {
    setLoading(true);
    try {
      const points = await tauriCommands.getRestorePoints();
      setRestorePoints(points);
    } catch (error) {
      console.error('Failed to load restore points:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleRestore = async (point: RestorePoint) => {
    if (!confirm(`Are you sure you want to restore to "${point.timestamp}"?`)) {
      return;
    }

    try {
      await tauriCommands.restoreFromPoint(point.path);
      if (onRestore) {
        onRestore();
      }
      onOpenChange(false);
    } catch (error) {
      console.error('Failed to restore:', error);
      alert(`Failed to restore: ${error instanceof Error ? error.message : String(error)}`);
    }
  };

  const handleCreate = async () => {
    if (!newPointName.trim()) {
      return;
    }

    setCreating(true);
    try {
      await tauriCommands.createBackupPoint(newPointName);
      setNewPointName('');
      setShowCreateForm(false);
      await loadRestorePoints();
    } catch (error) {
      console.error('Failed to create restore point:', error);
      alert(
        `Failed to create restore point: ${error instanceof Error ? error.message : String(error)}`
      );
    } finally {
      setCreating(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[700px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <History className="h-5 w-5" />
            Restore Points
          </DialogTitle>
          <DialogDescription>
            Restore your game to a previous state or create a new backup.
          </DialogDescription>
        </DialogHeader>

        {!showCreateForm ? (
          <div className="space-y-4">
            <div className="flex justify-end">
              <Button onClick={() => setShowCreateForm(true)} size="sm" variant="outline">
                <History className="mr-2 h-4 w-4" />
                Create Restore Point
              </Button>
            </div>

            <div className="max-h-[400px] overflow-y-auto border rounded-lg">
              {loading ? (
                <div className="flex items-center justify-center py-8">
                  <p className="text-muted-foreground">Loading restore points...</p>
                </div>
              ) : restorePoints.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-8 text-center">
                  <History className="h-12 w-12 text-muted-foreground/50 mb-3" />
                  <p className="text-muted-foreground">No restore points yet</p>
                  <p className="text-sm text-muted-foreground mt-1">
                    Create one to backup your current state
                  </p>
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Timestamp</TableHead>
                      <TableHead className="text-right">Actions</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {restorePoints.map((point, index) => (
                      <TableRow key={index}>
                        <TableCell className="font-mono text-sm">{point.timestamp}</TableCell>
                        <TableCell className="text-right">
                          <Button variant="ghost" size="sm" onClick={() => handleRestore(point)}>
                            <Undo2 className="mr-2 h-4 w-4" />
                            Restore
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              )}
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="grid gap-2">
              <Label htmlFor="point-name">Restore Point Name</Label>
              <Input
                id="point-name"
                value={newPointName}
                onChange={(e) => setNewPointName(e.target.value)}
                placeholder="e.g., Before installing new tactics"
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !creating) {
                    void handleCreate();
                  }
                }}
              />
            </div>

            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => {
                  setShowCreateForm(false);
                  setNewPointName('');
                }}
                disabled={creating}
              >
                Cancel
              </Button>
              <Button onClick={handleCreate} disabled={!newPointName.trim() || creating}>
                {creating ? 'Creating...' : 'Create'}
              </Button>
            </DialogFooter>
          </div>
        )}

        {!showCreateForm && (
          <DialogFooter>
            <Button onClick={() => onOpenChange(false)}>Close</Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
}
