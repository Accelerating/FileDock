import { useState, useEffect, useCallback, useRef } from "react";
import { useParams, useNavigate } from "react-router";
import {
  FolderOpen,
  Upload,
  FolderPlus,
  FilePlus,
  RefreshCw,
  Grid,
  List,
  SortAsc,
  SortDesc,
  UploadIcon,
} from "lucide-react";
import { api, type FileEntry, type ListParams } from "~/lib/api";
import { Button } from "~/components/ui/button";
import { Badge } from "~/components/ui/badge";
import { Skeleton } from "~/components/ui/skeleton";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "~/components/ui/tooltip";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "~/components/ui/dropdown-menu";
import { BreadcrumbNav } from "~/components/breadcrumb-nav";
import { FileList } from "~/components/file-list";
import { UploadDialog } from "~/components/upload-dialog";
import { CreateDialog } from "~/components/create-dialog";
import { RenameDialog } from "~/components/rename-dialog";
import { DeleteDialog } from "~/components/delete-dialog";
import { toast } from "sonner";

export default function BrowsePage() {
  const { "*": path } = useParams();
  const navigate = useNavigate();
  const dragCountRef = useRef(0);

  const [entries, setEntries] = useState<FileEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [sortBy, setSortBy] = useState<ListParams["sort_by"]>("name");
  const [sortOrder, setSortOrder] = useState<ListParams["sort_order"]>("asc");
  const [isDragging, setIsDragging] = useState(false);
  const [uploading, setUploading] = useState(false);

  // Dialog states
  const [uploadOpen, setUploadOpen] = useState(false);
  const [createOpen, setCreateOpen] = useState(false);
  const [renameOpen, setRenameOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [selectedEntry, setSelectedEntry] = useState<FileEntry | null>(null);

  const currentPath = path || "";

  const fetchEntries = useCallback(async () => {
    try {
      setLoading(true);
      const response = await api.listDir({
        path: currentPath ? `/${currentPath}` : "/",
        sort_by: sortBy,
        sort_order: sortOrder,
      });
      setEntries(response.items);
    } catch (error) {
      toast.error(`Failed to load directory: ${error instanceof Error ? error.message : "Unknown error"}`);
    } finally {
      setLoading(false);
    }
  }, [currentPath, sortBy, sortOrder]);

  useEffect(() => {
    fetchEntries();
  }, [fetchEntries]);

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    await fetchEntries();
    setRefreshing(false);
  }, [fetchEntries]);

  // Drag and drop handlers
  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    // Don't show drag overlay if upload dialog is open
    if (uploadOpen) return;
    dragCountRef.current++;
    if (dragCountRef.current === 1) {
      setIsDragging(true);
    }
  }, [uploadOpen]);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (uploadOpen) return;
    dragCountRef.current--;
    if (dragCountRef.current === 0) {
      setIsDragging(false);
    }
  }, [uploadOpen]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCountRef.current = 0;
    setIsDragging(false);

    // Don't handle drop if upload dialog is open
    if (uploadOpen) return;

    const files = Array.from(e.dataTransfer.files);
    if (files.length === 0) return;

    setUploading(true);
    try {
      await api.uploadFiles(currentPath ? `/${currentPath}` : "/", files);
      toast.success(`Uploaded ${files.length} file(s)`);
      await fetchEntries();
    } catch (error) {
      toast.error(`Upload failed: ${error instanceof Error ? error.message : "Unknown error"}`);
    } finally {
      setUploading(false);
    }
  }, [currentPath, fetchEntries, uploadOpen]);

  const handleUpload = useCallback(
    async (files: File[]) => {
      await api.uploadFiles(currentPath ? `/${currentPath}` : "/", files);
      await fetchEntries();
    },
    [currentPath, fetchEntries]
  );

  const handleCreateDir = useCallback(
    async (path: string) => {
      await api.createDir(path);
      await fetchEntries();
    },
    [fetchEntries]
  );

  const handleCreateFile = useCallback(
    async (path: string, content: string) => {
      await api.writeFile(path, content);
      await fetchEntries();
    },
    [fetchEntries]
  );

  const handleDelete = useCallback(
    async (path: string) => {
      const entry = entries.find((e) => e.path === path);
      if (entry) {
        setSelectedEntry(entry);
        setDeleteOpen(true);
      }
    },
    [entries]
  );

  const handleDeleteConfirm = useCallback(async () => {
    if (!selectedEntry) return;
    await api.delete(selectedEntry.path);
    await fetchEntries();
  }, [selectedEntry, fetchEntries]);

  const handleForceDeleteConfirm = useCallback(async () => {
    if (!selectedEntry) return;
    await api.forceDelete(selectedEntry.path);
    await fetchEntries();
  }, [selectedEntry, fetchEntries]);

  const handleRename = useCallback(
    async (path: string) => {
      const entry = entries.find((e) => e.path === path);
      if (entry) {
        setSelectedEntry(entry);
        setRenameOpen(true);
      }
    },
    [entries]
  );

  const handleRenameConfirm = useCallback(
    async (newName: string) => {
      if (!selectedEntry) return;
      const parentPath = selectedEntry.path.split("/").slice(0, -1).join("/");
      const newPath = `${parentPath}/${newName}`;
      await api.rename(selectedEntry.path, newPath);
      await fetchEntries();
    },
    [selectedEntry, fetchEntries]
  );

  const handleCopy = useCallback(
    async (path: string) => {
      const entry = entries.find((e) => e.path === path);
      if (entry) {
        const newName = `${entry.name}_copy`;
        const parentPath = path.split("/").slice(0, -1).join("/");
        const newPath = `${parentPath}/${newName}`;
        await api.copy(path, newPath);
        toast.success(`Copied to ${newName}`);
        await fetchEntries();
      }
    },
    [entries, fetchEntries]
  );

  const handleMove = useCallback(
    async (path: string) => {
      // For now, just show a toast
      toast.info("Move functionality coming soon");
    },
    []
  );

  const handleDownload = useCallback(async (path: string) => {
    try {
      const { blob, filename } = await api.downloadFile(path);
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      toast.success(`Downloaded ${filename}`);
    } catch (error) {
      toast.error(`Download failed: ${error instanceof Error ? error.message : "Unknown error"}`);
    }
  }, []);

  return (
    <div
      className="flex flex-col h-full relative"
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
    >
      {/* Drag overlay */}
      {isDragging && (
        <div className="absolute inset-0 z-50 bg-background/80 backdrop-blur-sm flex items-center justify-center">
          <div className="flex flex-col items-center gap-4 p-8 border-2 border-dashed border-primary rounded-lg">
            <UploadIcon className="size-12 text-primary animate-bounce" />
            <p className="text-lg font-medium">Drop files here to upload</p>
            <p className="text-sm text-muted-foreground">
              Files will be uploaded to {currentPath ? `/${currentPath}` : "/"}
            </p>
          </div>
        </div>
      )}

      {/* Upload progress indicator */}
      {uploading && (
        <div className="absolute top-4 right-4 z-50">
          <Badge variant="secondary" className="gap-2">
            <RefreshCw className="size-3 animate-spin" />
            Uploading...
          </Badge>
        </div>
      )}

      {/* Header */}
      <header className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center gap-4">
          <BreadcrumbNav path={currentPath} />
          <Badge variant="secondary">
            {entries.length} item{entries.length !== 1 ? "s" : ""}
          </Badge>
        </div>

        <div className="flex items-center gap-2">
          {/* Sort dropdown */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="gap-2">
                {sortOrder === "asc" ? (
                  <SortAsc className="size-4" />
                ) : (
                  <SortDesc className="size-4" />
                )}
                Sort
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => setSortBy("name")}>
                Name
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => setSortBy("size")}>
                Size
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => setSortBy("modified")}>
                Modified
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => setSortOrder(sortOrder === "asc" ? "desc" : "asc")}>
                {sortOrder === "asc" ? "Descending" : "Ascending"}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          {/* Refresh */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="outline"
                size="icon"
                onClick={handleRefresh}
                disabled={refreshing}
              >
                <RefreshCw
                  className={`size-4 ${refreshing ? "animate-spin" : ""}`}
                />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Refresh</TooltipContent>
          </Tooltip>

          {/* Create */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="gap-2">
                <FolderPlus className="size-4" />
                New
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={() => setCreateOpen(true)}>
                <FolderPlus className="size-4 mr-2" />
                Folder
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => setCreateOpen(true)}>
                <FilePlus className="size-4 mr-2" />
                File
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          {/* Upload */}
          <Button
            size="sm"
            className="gap-2"
            onClick={() => setUploadOpen(true)}
          >
            <Upload className="size-4" />
            Upload
          </Button>
        </div>
      </header>

      {/* Content */}
      <main className="flex-1 overflow-auto p-4">
        {loading ? (
          <div className="space-y-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="flex items-center gap-4">
                <Skeleton className="size-5" />
                <Skeleton className="h-4 w-48" />
                <Skeleton className="h-4 w-24 ml-auto" />
                <Skeleton className="h-4 w-32" />
              </div>
            ))}
          </div>
        ) : (
          <FileList
            entries={entries}
            onRefresh={handleRefresh}
            onDelete={handleDelete}
            onRename={handleRename}
            onCopy={handleCopy}
            onMove={handleMove}
            onDownload={handleDownload}
          />
        )}
      </main>

      {/* Dialogs */}
      <UploadDialog
        open={uploadOpen}
        onOpenChange={setUploadOpen}
        currentPath={currentPath ? `/${currentPath}` : "/"}
        onUpload={handleUpload}
      />

      <CreateDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        currentPath={currentPath ? `/${currentPath}` : "/"}
        onCreateDir={handleCreateDir}
        onCreateFile={handleCreateFile}
      />

      {selectedEntry && (
        <>
          <RenameDialog
            open={renameOpen}
            onOpenChange={setRenameOpen}
            currentPath={selectedEntry.path}
            currentName={selectedEntry.name}
            onRename={handleRenameConfirm}
          />

          <DeleteDialog
            open={deleteOpen}
            onOpenChange={setDeleteOpen}
            itemName={selectedEntry.name}
            isDir={selectedEntry.is_dir}
            onDelete={handleDeleteConfirm}
            onForceDelete={selectedEntry.is_dir ? handleForceDeleteConfirm : undefined}
          />
        </>
      )}
    </div>
  );
}
