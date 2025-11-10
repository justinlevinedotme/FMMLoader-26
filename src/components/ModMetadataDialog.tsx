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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { tauriCommands, type ModMetadata } from "@/hooks/useTauri";

interface ModMetadataDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  sourcePath: string;
  onSubmit: (metadata: ModMetadata) => void;
}

const MOD_TYPES = [
  { value: "ui", label: "UI" },
  { value: "bundle", label: "Bundle" },
  { value: "tactics", label: "Tactics" },
  { value: "graphics", label: "Graphics" },
  { value: "skins", label: "Skins" },
  { value: "editor-data", label: "Editor Data" },
  { value: "misc", label: "Miscellaneous" },
];

export function ModMetadataDialog({
  open,
  onOpenChange,
  sourcePath,
  onSubmit,
}: ModMetadataDialogProps) {
  const [name, setName] = useState("");
  const [version, setVersion] = useState("1.0.0");
  const [modType, setModType] = useState("misc");
  const [author, setAuthor] = useState("");
  const [description, setDescription] = useState("");
  const [detecting, setDetecting] = useState(false);

  // Auto-detect mod type when dialog opens
  useEffect(() => {
    if (open && sourcePath) {
      const detectType = async () => {
        setDetecting(true);
        try {
          const detectedType = await tauriCommands.detectModType(sourcePath);
          setModType(detectedType);
        } catch (err) {
          console.error('Failed to detect mod type:', err);
        } finally {
          setDetecting(false);
        }
      };

      void detectType();

      // Extract a default name from the path
      const pathParts = sourcePath.split(/[/\\]/);
      const lastPart = pathParts[pathParts.length - 1];
      const nameWithoutExt = lastPart.replace(/\.(zip|bundle|fmf)$/i, '');
      setName(nameWithoutExt);
    }
  }, [open, sourcePath]);

  const handleSubmit = () => {
    if (!name || !version || !modType) {
      return;
    }

    onSubmit({
      name,
      version,
      mod_type: modType,
      author: author || undefined,
      description: description || undefined,
    });

    // Reset form
    setName("");
    setVersion("1.0.0");
    setModType("misc");
    setAuthor("");
    setDescription("");
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Mod Information Required</DialogTitle>
          <DialogDescription>
            This mod doesn&apos;t have a manifest.json file. Please provide some
            information about it.
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="name">
              Mod Name <span className="text-destructive">*</span>
            </Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="My Awesome Mod"
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="version">
              Version <span className="text-destructive">*</span>
            </Label>
            <Input
              id="version"
              value={version}
              onChange={(e) => setVersion(e.target.value)}
              placeholder="1.0.0"
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="type">
              Mod Type <span className="text-destructive">*</span>
            </Label>
            <Select value={modType} onValueChange={setModType} disabled={detecting}>
              <SelectTrigger id="type">
                <SelectValue placeholder="Select mod type" />
              </SelectTrigger>
              <SelectContent>
                {MOD_TYPES.map((type) => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {detecting && (
              <p className="text-xs text-muted-foreground">
                Auto-detecting type...
              </p>
            )}
          </div>

          <div className="grid gap-2">
            <Label htmlFor="author">Author (Optional)</Label>
            <Input
              id="author"
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              placeholder="Your Name"
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="description">Description (Optional)</Label>
            <Input
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What does this mod do?"
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!name || !version || !modType}>
            Import Mod
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
