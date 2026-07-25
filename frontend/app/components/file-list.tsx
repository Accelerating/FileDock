import { useState, useCallback } from "react";
import { useNavigate } from "react-router";
import {
  Folder,
  File,
  FileText,
  FileImage,
  FileVideo,
  FileAudio,
  FileArchive,
  FileCode,
  MoreHorizontal,
  Download,
  Trash2,
  Copy,
  Move,
  Edit,
  Info,
} from "lucide-react";
import { cn, formatFileSize, formatDate, getFileExtension } from "~/lib/utils";
import { Button } from "~/components/ui/button";
import { Badge } from "~/components/ui/badge";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuGroup,
} from "~/components/ui/dropdown-menu";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "~/components/ui/tooltip";
import type { FileEntry } from "~/lib/api";

interface FileListProps {
  entries: FileEntry[];
  onRefresh: () => void;
  onDelete: (path: string) => void;
  onRename: (path: string) => void;
  onCopy: (path: string) => void;
  onMove: (path: string) => void;
  onDownload: (path: string) => void;
}

function getFileIcon(entry: FileEntry) {
  if (entry.is_dir) {
    return <Folder className="size-5 text-blue-500" />;
  }

  const ext = getFileExtension(entry.name);
  const iconClass = "size-5";

  // Images
  if (["jpg", "jpeg", "png", "gif", "svg", "webp", "bmp"].includes(ext)) {
    return <FileImage className={cn(iconClass, "text-green-500")} />;
  }

  // Videos
  if (["mp4", "avi", "mov", "mkv", "webm"].includes(ext)) {
    return <FileVideo className={cn(iconClass, "text-purple-500")} />;
  }

  // Audio
  if (["mp3", "wav", "flac", "ogg", "aac"].includes(ext)) {
    return <FileAudio className={cn(iconClass, "text-orange-500")} />;
  }

  // Archives
  if (["zip", "rar", "tar", "gz", "7z"].includes(ext)) {
    return <FileArchive className={cn(iconClass, "text-yellow-500")} />;
  }

  // Code
  if (
    [
      "js", "ts", "jsx", "tsx", "py", "java", "cpp", "c", "rs", "go",
      "html", "css", "json", "xml", "yaml", "yml", "md",
    ].includes(ext)
  ) {
    return <FileCode className={cn(iconClass, "text-cyan-500")} />;
  }

  // Text files
  if (["txt", "log", "csv"].includes(ext)) {
    return <FileText className={cn(iconClass, "text-gray-500")} />;
  }

  return <File className={cn(iconClass, "text-gray-400")} />;
}

export function FileList({
  entries,
  onRefresh,
  onDelete,
  onRename,
  onCopy,
  onMove,
  onDownload,
}: FileListProps) {
  const navigate = useNavigate();
  const [selectedEntries, setSelectedEntries] = useState<Set<string>>(new Set());

  const handleSelect = useCallback((path: string, e: React.MouseEvent) => {
    setSelectedEntries((prev) => {
      const next = new Set(prev);
      if (e.ctrlKey || e.metaKey) {
        if (next.has(path)) {
          next.delete(path);
        } else {
          next.add(path);
        }
      } else {
        next.clear();
        next.add(path);
      }
      return next;
    });
  }, []);

  const handleClick = useCallback(
    (entry: FileEntry, e: React.MouseEvent) => {
      if (entry.is_dir) {
        // Single click on directory navigates into it
        const path = entry.path.startsWith('/') ? entry.path : `/${entry.path}`;
        navigate(`/browse${path}`);
      } else {
        // Single click on file selects it
        handleSelect(entry.path, e);
      }
    },
    [navigate, handleSelect]
  );

  if (entries.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
        <Folder className="size-12 mb-4 opacity-50" />
        <p className="text-lg font-medium">This folder is empty</p>
        <p className="text-sm">Drop files here or use the upload button</p>
      </div>
    );
  }

  return (
    <div className="border rounded-lg overflow-hidden">
      {/* Header */}
      <div className="grid grid-cols-[1fr_120px_150px_100px] gap-4 p-3 bg-muted/50 text-sm font-medium text-muted-foreground">
        <div>Name</div>
        <div className="text-right">Size</div>
        <div className="text-right">Modified</div>
        <div className="text-right">Actions</div>
      </div>

      {/* Entries */}
      <div className="divide-y">
        {entries.map((entry) => {
          const isSelected = selectedEntries.has(entry.path);

          return (
            <div
              key={entry.path}
              className={cn(
                "grid grid-cols-[1fr_120px_150px_100px] gap-4 p-3 hover:bg-muted/50 cursor-pointer transition-colors",
                isSelected && "bg-muted/50"
              )}
              onClick={(e) => handleClick(entry, e)}
            >
              {/* Name */}
              <div className="flex items-center gap-3 min-w-0">
                {getFileIcon(entry)}
                <span className="truncate font-medium">
                  {entry.name}
                </span>
                {entry.is_dir && (
                  <Badge variant="secondary" className="ml-auto shrink-0">
                    Folder
                  </Badge>
                )}
              </div>

              {/* Size */}
              <div className="text-right text-sm text-muted-foreground">
                {entry.is_dir ? "—" : formatFileSize(entry.size)}
              </div>

              {/* Modified */}
              <div className="text-right text-sm text-muted-foreground">
                {formatDate(entry.modified)}
              </div>

              {/* Actions */}
              <div className="text-right">
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="size-8"
                      onClick={(e) => e.stopPropagation()}
                    >
                      <MoreHorizontal className="size-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuGroup>
                      {!entry.is_dir && (
                        <DropdownMenuItem
                          onClick={() => onDownload(entry.path)}
                        >
                          <Download className="size-4 mr-2" />
                          Download
                        </DropdownMenuItem>
                      )}
                      <DropdownMenuItem onClick={() => onRename(entry.path)}>
                        <Edit className="size-4 mr-2" />
                        Rename
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => onCopy(entry.path)}>
                        <Copy className="size-4 mr-2" />
                        Copy
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => onMove(entry.path)}>
                        <Move className="size-4 mr-2" />
                        Move
                      </DropdownMenuItem>
                    </DropdownMenuGroup>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem
                      onClick={() => onDelete(entry.path)}
                      className="text-destructive"
                    >
                      <Trash2 className="size-4 mr-2" />
                      Delete
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
