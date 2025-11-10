import { useEffect, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { AlertTriangle } from "lucide-react";
import { tauriCommands, type ConflictInfo } from "@/hooks/useTauri";

interface ConflictsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDisableMod?: (modName: string) => void;
}

export function ConflictsDialog({
  open,
  onOpenChange,
  onDisableMod,
}: ConflictsDialogProps) {
  const [conflicts, setConflicts] = useState<ConflictInfo[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (open) {
      void loadConflicts();
    }
  }, [open]);

  const loadConflicts = async () => {
    setLoading(true);
    try {
      const result = await tauriCommands.checkConflicts();
      setConflicts(result);
    } catch (error) {
      console.error("Failed to load conflicts:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleDisable = (modName: string) => {
    if (onDisableMod) {
      onDisableMod(modName);
      // Reload conflicts after a short delay
      setTimeout(loadConflicts, 500);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[700px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-yellow-500" />
            Mod Conflicts Detected
          </DialogTitle>
          <DialogDescription>
            The following files are modified by multiple mods. The last enabled
            mod will override others.
          </DialogDescription>
        </DialogHeader>

        <div className="max-h-[500px] overflow-y-auto">
          {loading ? (
            <div className="flex items-center justify-center py-8">
              <p className="text-muted-foreground">Loading conflicts...</p>
            </div>
          ) : conflicts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <p className="text-muted-foreground">No conflicts detected!</p>
              <p className="text-sm text-muted-foreground mt-2">
                All enabled mods are compatible.
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {conflicts.map((conflict, index) => (
                <div
                  key={index}
                  className="rounded-lg border p-4 space-y-3"
                >
                  <div>
                    <p className="font-medium text-sm mb-1">Conflicting File:</p>
                    <p className="text-xs font-mono text-muted-foreground break-all">
                      {conflict.file_path}
                    </p>
                  </div>

                  <div>
                    <p className="font-medium text-sm mb-2">
                      Affected Mods ({conflict.conflicting_mods.length}):
                    </p>
                    <div className="space-y-1">
                      {conflict.conflicting_mods.map((modName) => (
                        <div
                          key={modName}
                          className="flex items-center justify-between bg-muted/50 rounded px-3 py-2"
                        >
                          <span className="text-sm">{modName}</span>
                          {onDisableMod && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleDisable(modName)}
                              className="h-7 text-xs"
                            >
                              Disable
                            </Button>
                          )}
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="flex justify-between items-center pt-4 border-t">
          <p className="text-sm text-muted-foreground">
            {conflicts.length} conflict{conflicts.length !== 1 ? "s" : ""} found
          </p>
          <Button onClick={() => onOpenChange(false)}>Close</Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
