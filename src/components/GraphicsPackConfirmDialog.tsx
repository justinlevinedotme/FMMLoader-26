/**
 * GraphicsPackConfirmDialog Component
 *
 * Confirmation dialog shown before installing a graphics pack.
 * Displays pack analysis results including detected type, confidence score,
 * installation path preview, and options for mixed pack splitting.
 *
 * Features:
 * - Confidence color coding (green >70%, yellow >50%, red <50%)
 * - Low confidence warnings for packs <50% confidence
 * - Full path preview showing exact installation location
 * - Mixed pack split option to separate types into individual directories
 */
import { useState, useEffect } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { GraphicsPackAnalysis, GraphicsPackType } from "@/hooks/useTauri";

interface GraphicsPackConfirmDialogProps {
  analysis: GraphicsPackAnalysis | null;
  onConfirm: (installPath: string, shouldSplit: boolean) => void;
  onCancel: () => void;
  userDirPath?: string;
}

const getPackTypeLabel = (packType: GraphicsPackType): string => {
  if (typeof packType === "string") {
    return packType;
  }
  if (typeof packType === "object" && "Mixed" in packType) {
    const types = packType.Mixed.map((t) => getPackTypeLabel(t)).join(", ");
    return `Mixed (${types})`;
  }
  return "Unknown";
};

const formatBytes = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${Math.round((bytes / Math.pow(k, i)) * 100) / 100} ${sizes[i]}`;
};

export function GraphicsPackConfirmDialog({
  analysis,
  onConfirm,
  onCancel,
  userDirPath,
}: GraphicsPackConfirmDialogProps) {
  const [selectedPath, setSelectedPath] = useState<string>("");
  const [shouldSplit, setShouldSplit] = useState(false);

  const getResolvedPath = (relativePath: string): string => {
    if (!userDirPath) return relativePath;
    return `${userDirPath}/graphics/${relativePath}`;
  };

  useEffect(() => {
    if (analysis && analysis.suggested_paths.length > 0) {
      setSelectedPath(analysis.suggested_paths[0]);
      setShouldSplit(false);
    }
  }, [analysis]);

  if (!analysis) return null;

  const isMixed =
    typeof analysis.pack_type === "object" && "Mixed" in analysis.pack_type;

  const confidencePercent = Math.round(analysis.confidence * 100);
  const confidenceColor =
    confidencePercent >= 70
      ? "text-green-600 dark:text-green-400"
      : confidencePercent >= 50
      ? "text-yellow-600 dark:text-yellow-400"
      : "text-red-600 dark:text-red-400";

  const open = analysis !== null;

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && onCancel()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Configure Graphics Pack Installation</DialogTitle>
          <DialogDescription>
            {getPackTypeLabel(analysis.pack_type)} - {analysis.file_count.toLocaleString()} files ({formatBytes(analysis.total_size_bytes)})
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 my-4">
          {/* Pack Info */}
          <div className="text-sm">
            <p className="mb-2">
              Detected as <strong>{getPackTypeLabel(analysis.pack_type)}</strong> with <span className={confidenceColor}>{confidencePercent}% confidence</span>
            </p>
            <div className="bg-muted p-2 rounded text-muted-foreground text-xs space-y-1">
              <div>Structure: {analysis.is_flat_pack ? "Flat" : "Structured"}</div>
              <div>Config XML: {analysis.has_config_xml ? "Found" : "Not found"}</div>
            </div>
          </div>

          {/* Low confidence warning */}
          {confidencePercent < 50 && (
            <div className="bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-800 p-3 rounded-md">
              <p className="text-sm text-amber-900 dark:text-amber-200">
                <strong>Low confidence detection.</strong> Please verify the installation path is correct for your graphics pack type. If unsure, check the pack's documentation or contents.
              </p>
            </div>
          )}

          {/* Installation Path Selection */}
          <div className="space-y-2">
            <Label htmlFor="install-path">Install to</Label>
            <Select value={selectedPath} onValueChange={setSelectedPath}>
              <SelectTrigger id="install-path">
                <SelectValue placeholder="Select installation directory" />
              </SelectTrigger>
              <SelectContent>
                {analysis.suggested_paths.map((path) => (
                  <SelectItem key={path} value={path}>
                    graphics/{path}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {userDirPath && (
              <p className="text-xs text-muted-foreground font-mono break-all">
                {getResolvedPath(selectedPath)}
              </p>
            )}
          </div>

          {/* Mixed Pack Split Option */}
          {isMixed && (
            <div className="bg-blue-50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-800 p-3 rounded-md">
              <div className="flex items-start gap-3">
                <input
                  type="checkbox"
                  checked={shouldSplit}
                  onChange={(e) => setShouldSplit(e.target.checked)}
                  className="mt-1"
                  id="split-pack"
                />
                <label htmlFor="split-pack" className="flex-1 cursor-pointer text-sm">
                  <div className="font-medium">Split pack by type</div>
                  <div className="text-xs text-muted-foreground">
                    Install each type (faces, logos, kits) to its own directory
                  </div>
                </label>
              </div>
            </div>
          )}

          <p className="text-sm text-muted-foreground">
            {shouldSplit
              ? "Multiple directories will be created for each type"
              : "Pack contents will be copied to this directory"}
          </p>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onCancel}>
            Cancel
          </Button>
          <Button onClick={() => onConfirm(selectedPath, shouldSplit)} disabled={!selectedPath}>
            Install Graphics Pack
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
