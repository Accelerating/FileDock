import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

export function formatDate(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  // Less than 1 minute
  if (diff < 60 * 1000) {
    return "just now";
  }

  // Less than 1 hour
  if (diff < 60 * 60 * 1000) {
    const minutes = Math.floor(diff / (60 * 1000));
    return `${minutes} min ago`;
  }

  // Less than 24 hours
  if (diff < 24 * 60 * 60 * 1000) {
    const hours = Math.floor(diff / (60 * 60 * 1000));
    return `${hours} hour${hours > 1 ? "s" : ""} ago`;
  }

  // Less than 7 days
  if (diff < 7 * 24 * 60 * 60 * 1000) {
    const days = Math.floor(diff / (24 * 60 * 60 * 1000));
    return `${days} day${days > 1 ? "s" : ""} ago`;
  }

  // Otherwise, show full date
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function getFileExtension(filename: string): string {
  const lastDot = filename.lastIndexOf(".");
  if (lastDot === -1) return "";
  return filename.slice(lastDot + 1).toLowerCase();
}

export function getFileIcon(filename: string, isDir: boolean): string {
  if (isDir) return "folder";

  const ext = getFileExtension(filename);
  const iconMap: Record<string, string> = {
    // Documents
    pdf: "file-text",
    doc: "file-text",
    docx: "file-text",
    txt: "file-text",
    md: "file-text",
    // Images
    jpg: "image",
    jpeg: "image",
    png: "image",
    gif: "image",
    svg: "image",
    webp: "image",
    // Videos
    mp4: "video",
    avi: "video",
    mov: "video",
    mkv: "video",
    // Audio
    mp3: "music",
    wav: "music",
    flac: "music",
    ogg: "music",
    // Archives
    zip: "archive",
    rar: "archive",
    tar: "archive",
    gz: "archive",
    "7z": "archive",
    // Code
    js: "code",
    ts: "code",
    jsx: "code",
    tsx: "code",
    py: "code",
    java: "code",
    cpp: "code",
    c: "code",
    rs: "code",
    go: "code",
    html: "code",
    css: "code",
    json: "code",
    xml: "code",
    yaml: "code",
    yml: "code",
    // Data
    csv: "table",
    xls: "table",
    xlsx: "table",
    // Executables
    exe: "terminal",
    sh: "terminal",
    bat: "terminal",
  };

  return iconMap[ext] || "file";
}

export function getParentPath(path: string): string {
  if (path === "/" || path === "") return "/";
  const parts = path.split("/").filter(Boolean);
  parts.pop();
  return "/" + parts.join("/");
}

export function joinPath(...parts: string[]): string {
  return parts
    .map((part) => part.replace(/^\/|\/$/g, ""))
    .filter(Boolean)
    .join("/");
}

export function getBreadcrumbs(path: string): { name: string; path: string }[] {
  if (path === "/" || path === "") {
    return [{ name: "Root", path: "/" }];
  }

  const parts = path.split("/").filter(Boolean);
  const breadcrumbs = [{ name: "Root", path: "/" }];

  let currentPath = "";
  for (const part of parts) {
    currentPath += "/" + part;
    breadcrumbs.push({ name: part, path: currentPath });
  }

  return breadcrumbs;
}
