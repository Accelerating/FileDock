import { useState, useCallback } from "react";
import { Trash2, AlertTriangle } from "lucide-react";
import { Button } from "~/components/ui/button";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "~/components/ui/alert-dialog";
import { toast } from "sonner";

interface DeleteDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  itemName: string;
  isDir?: boolean;
  onDelete: () => Promise<void>;
  onForceDelete?: () => Promise<void>;
}

export function DeleteDialog({
  open,
  onOpenChange,
  itemName,
  isDir = false,
  onDelete,
  onForceDelete,
}: DeleteDialogProps) {
  const [deleting, setDeleting] = useState(false);
  const [showForceConfirm, setShowForceConfirm] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const handleDelete = useCallback(async () => {
    setDeleting(true);
    setErrorMessage(null);

    try {
      await onDelete();
      toast.success(`"${itemName}" deleted`);
      onOpenChange(false);
      setShowForceConfirm(false);
    } catch (error) {
      const msg = error instanceof Error ? error.message : "Unknown error";
      // If it's a non-empty directory error, show force delete option
      if (msg.includes("not empty") && isDir && onForceDelete) {
        setErrorMessage(msg);
        setShowForceConfirm(true);
      } else {
        toast.error(`Failed to delete: ${msg}`);
      }
    } finally {
      setDeleting(false);
    }
  }, [itemName, onDelete, onOpenChange, isDir, onForceDelete]);

  const handleForceDelete = useCallback(async () => {
    if (!onForceDelete) return;
    
    setDeleting(true);

    try {
      await onForceDelete();
      toast.success(`"${itemName}" and its contents deleted`);
      onOpenChange(false);
      setShowForceConfirm(false);
      setErrorMessage(null);
    } catch (error) {
      toast.error(`Failed to delete: ${error instanceof Error ? error.message : "Unknown error"}`);
    } finally {
      setDeleting(false);
    }
  }, [itemName, onForceDelete, onOpenChange]);

  const handleCancel = useCallback(() => {
    setShowForceConfirm(false);
    setErrorMessage(null);
    onOpenChange(false);
  }, [onOpenChange]);

  // Show force delete confirmation for non-empty directories
  if (showForceConfirm) {
    return (
      <AlertDialog open={open} onOpenChange={handleCancel}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="flex items-center gap-2">
              <AlertTriangle className="size-5 text-warning" />
              Directory is not empty
            </AlertDialogTitle>
            <AlertDialogDescription>
              "{itemName}" contains files or subdirectories. 
              Deleting it will permanently remove all its contents. 
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deleting} onClick={handleCancel}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={handleForceDelete}
              disabled={deleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {deleting ? (
                <>
                  <Trash2 className="size-4 mr-2 animate-spin" />
                  Deleting all contents...
                </>
              ) : (
                <>
                  <Trash2 className="size-4 mr-2" />
                  Delete all contents
                </>
              )}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    );
  }

  // Normal delete confirmation
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Are you sure?</AlertDialogTitle>
          <AlertDialogDescription>
            This will permanently delete "{itemName}". This action cannot be
            undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={deleting}>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDelete}
            disabled={deleting}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {deleting ? (
              <>
                <Trash2 className="size-4 mr-2 animate-spin" />
                Deleting...
              </>
            ) : (
              <>
                <Trash2 className="size-4 mr-2" />
                Delete
              </>
            )}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
