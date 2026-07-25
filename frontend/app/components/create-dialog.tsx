import { useState, useCallback } from "react";
import { FolderPlus, FilePlus } from "lucide-react";
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { toast } from "sonner";

interface CreateDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  currentPath: string;
  onCreateDir: (path: string) => Promise<void>;
  onCreateFile: (path: string, content: string) => Promise<void>;
}

export function CreateDialog({
  open,
  onOpenChange,
  currentPath,
  onCreateDir,
  onCreateFile,
}: CreateDialogProps) {
  const [name, setName] = useState("");
  const [content, setContent] = useState("");
  const [creating, setCreating] = useState(false);
  const [tab, setTab] = useState("folder");

  const handleCreate = useCallback(async () => {
    if (!name.trim()) {
      toast.error("Name is required");
      return;
    }

    setCreating(true);

    try {
      const path = currentPath
        ? `${currentPath}/${name}`
        : `/${name}`;

      if (tab === "folder") {
        await onCreateDir(path);
        toast.success(`Folder "${name}" created`);
      } else {
        await onCreateFile(path, content);
        toast.success(`File "${name}" created`);
      }

      setName("");
      setContent("");
      onOpenChange(false);
    } catch (error) {
      toast.error(`Failed to create: ${error instanceof Error ? error.message : "Unknown error"}`);
    } finally {
      setCreating(false);
    }
  }, [name, content, tab, currentPath, onCreateDir, onCreateFile, onOpenChange]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Create New</DialogTitle>
          <DialogDescription>
            Create a new file or folder in {currentPath || "/"}
          </DialogDescription>
        </DialogHeader>

        <Tabs value={tab} onValueChange={setTab}>
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="folder" className="gap-2">
              <FolderPlus className="size-4" />
              Folder
            </TabsTrigger>
            <TabsTrigger value="file" className="gap-2">
              <FilePlus className="size-4" />
              File
            </TabsTrigger>
          </TabsList>

          <TabsContent value="folder" className="space-y-4 mt-4">
            <div className="space-y-2">
              <Label htmlFor="folder-name">Folder Name</Label>
              <Input
                id="folder-name"
                placeholder="New Folder"
                value={name}
                onChange={(e) => setName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    handleCreate();
                  }
                }}
              />
            </div>
          </TabsContent>

          <TabsContent value="file" className="space-y-4 mt-4">
            <div className="space-y-2">
              <Label htmlFor="file-name">File Name</Label>
              <Input
                id="file-name"
                placeholder="new-file.txt"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="file-content">Content</Label>
              <textarea
                id="file-content"
                className="flex min-h-[120px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                placeholder="File content..."
                value={content}
                onChange={(e) => setContent(e.target.value)}
              />
            </div>
          </TabsContent>
        </Tabs>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={creating}
          >
            Cancel
          </Button>
          <Button onClick={handleCreate} disabled={!name.trim() || creating}>
            {creating ? "Creating..." : "Create"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
