import { useState, useCallback, useEffect } from "react";
import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "~/components/ui/dialog";
import { toast } from "sonner";

interface RenameDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  currentPath: string;
  currentName: string;
  onRename: (newName: string) => Promise<void>;
}

export function RenameDialog({
  open,
  onOpenChange,
  currentPath,
  currentName,
  onRename,
}: RenameDialogProps) {
  const [newName, setNewName] = useState(currentName);
  const [renaming, setRenaming] = useState(false);

  useEffect(() => {
    if (open) {
      setNewName(currentName);
    }
  }, [open, currentName]);

  const handleRename = useCallback(async () => {
    if (!newName.trim()) {
      toast.error("Name is required");
      return;
    }

    if (newName === currentName) {
      onOpenChange(false);
      return;
    }

    setRenaming(true);

    try {
      await onRename(newName);
      toast.success(`Renamed to "${newName}"`);
      onOpenChange(false);
    } catch (error) {
      toast.error(`Failed to rename: ${error instanceof Error ? error.message : "Unknown error"}`);
    } finally {
      setRenaming(false);
    }
  }, [newName, currentName, onRename, onOpenChange]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Rename</DialogTitle>
          <DialogDescription>
            Rename "{currentName}" to a new name
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="new-name">New Name</Label>
            <Input
              id="new-name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  handleRename();
                }
              }}
              autoFocus
            />
          </div>
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={renaming}
          >
            Cancel
          </Button>
          <Button
            onClick={handleRename}
            disabled={!newName.trim() || newName === currentName || renaming}
          >
            {renaming ? "Renaming..." : "Rename"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
