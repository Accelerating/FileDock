const API_BASE = import.meta.env.VITE_API_URL || '/api';

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  has_more: boolean;
}

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: string;
  created?: string;
}

export interface FileMetadata {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
  modified: string;
  created?: string;
  accessed?: string;
  permissions?: {
    readable: boolean;
    writable: boolean;
    executable: boolean;
  };
}

export interface DirStats {
  total_files: number;
  total_dirs: number;
  total_size: number;
}

export interface SearchResult {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
  modified: string;
}

export interface BatchResult {
  success_count: number;
  failure_count: number;
  failures: { path: string; error: string }[];
}

export interface HealthResponse {
  status: string;
  version: string;
  uptime_seconds: number;
  data_dir: string;
}

export interface ListParams {
  path: string;
  page?: number;
  page_size?: number;
  sort_by?: 'name' | 'size' | 'modified' | 'created';
  sort_order?: 'asc' | 'desc';
}

export interface SearchParams {
  path: string;
  pattern: string;
  recursive?: boolean;
  page?: number;
  page_size?: number;
}

async function fetchApi<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const url = `${API_BASE}${endpoint}`;
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || `HTTP error ${response.status}`);
  }

  return response.json();
}

export const api = {
  // Health check
  async health(): Promise<HealthResponse> {
    const response = await fetchApi<ApiResponse<HealthResponse>>('/health');
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Health check failed');
    }
    return response.data;
  },

  // List directory
  async listDir(params: ListParams): Promise<PaginatedResponse<FileEntry>> {
    const searchParams = new URLSearchParams();
    searchParams.set('path', params.path);
    if (params.page !== undefined) searchParams.set('page', params.page.toString());
    if (params.page_size !== undefined) searchParams.set('page_size', params.page_size.toString());
    if (params.sort_by) searchParams.set('sort_by', params.sort_by);
    if (params.sort_order) searchParams.set('sort_order', params.sort_order);

    const response = await fetchApi<ApiResponse<PaginatedResponse<FileEntry>>>(
      `/files?${searchParams.toString()}`
    );
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to list directory');
    }
    return response.data;
  },

  // Get file metadata
  async getMetadata(path: string): Promise<FileMetadata> {
    const searchParams = new URLSearchParams({ path });
    const response = await fetchApi<ApiResponse<FileMetadata>>(
      `/files/metadata?${searchParams.toString()}`
    );
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get metadata');
    }
    return response.data;
  },

  // Read file content
  async readFile(path: string, offset?: number, length?: number): Promise<Blob> {
    const searchParams = new URLSearchParams({ path });
    if (offset !== undefined) searchParams.set('offset', offset.toString());
    if (length !== undefined) searchParams.set('length', length.toString());

    const response = await fetch(`${API_BASE}/files/read?${searchParams.toString()}`);
    if (!response.ok) {
      throw new Error(`Failed to read file: ${response.statusText}`);
    }
    return response.blob();
  },

  // Write file content
  async writeFile(path: string, content: string): Promise<FileMetadata> {
    const searchParams = new URLSearchParams({ path });
    const response = await fetchApi<ApiResponse<FileMetadata>>(
      `/files/write?${searchParams.toString()}`,
      {
        method: 'POST',
        body: content,
        headers: { 'Content-Type': 'text/plain' },
      }
    );
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to write file');
    }
    return response.data;
  },

  // Upload files
  async uploadFiles(path: string, files: File[]): Promise<FileMetadata[]> {
    const formData = new FormData();
    files.forEach((file) => formData.append('file', file));

    const searchParams = new URLSearchParams({ path });
    const response = await fetch(
      `${API_BASE}/files/upload?${searchParams.toString()}`,
      {
        method: 'POST',
        body: formData,
      }
    );

    if (!response.ok) {
      throw new Error(`Upload failed: ${response.statusText}`);
    }

    const result: ApiResponse<FileMetadata[]> = await response.json();
    if (!result.success || !result.data) {
      throw new Error(result.error || 'Upload failed');
    }
    return result.data;
  },

  // Download file
  async downloadFile(path: string): Promise<{ blob: Blob; filename: string }> {
    const searchParams = new URLSearchParams({ path });
    const response = await fetch(
      `${API_BASE}/files/download?${searchParams.toString()}`
    );
    if (!response.ok) {
      throw new Error(`Download failed: ${response.statusText}`);
    }

    const contentDisposition = response.headers.get('Content-Disposition');
    const filename = contentDisposition
      ? contentDisposition.split('filename=')[1]?.replace(/"/g, '')
      : path.split('/').pop() || 'download';

    const blob = await response.blob();
    return { blob, filename };
  },

  // Create directory
  async createDir(path: string): Promise<FileMetadata> {
    const response = await fetchApi<ApiResponse<FileMetadata>>('/files', {
      method: 'POST',
      body: JSON.stringify({ path }),
    });
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to create directory');
    }
    return response.data;
  },

  // Delete file/directory (only empty directories)
  async delete(path: string): Promise<{ path: string; name: string; is_dir: boolean }> {
    const searchParams = new URLSearchParams({ path });
    const response = await fetchApi<ApiResponse<{ path: string; name: string; is_dir: boolean }>>(
      `/files/delete?${searchParams.toString()}`,
      { method: 'DELETE' }
    );
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to delete');
    }
    return response.data;
  },

  // Force delete file or directory (even if not empty)
  async forceDelete(path: string): Promise<{ path: string; name: string; is_dir: boolean }> {
    const searchParams = new URLSearchParams({ path });
    const response = await fetchApi<ApiResponse<{ path: string; name: string; is_dir: boolean }>>(
      `/files/force-delete?${searchParams.toString()}`,
      { method: 'DELETE' }
    );
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to delete');
    }
    return response.data;
  },

  // Rename/move
  async rename(from: string, to: string): Promise<FileMetadata> {
    const response = await fetchApi<ApiResponse<FileMetadata>>('/files/rename', {
      method: 'POST',
      body: JSON.stringify({ from, to }),
    });
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to rename');
    }
    return response.data;
  },

  // Copy
  async copy(from: string, to: string): Promise<FileMetadata> {
    const response = await fetchApi<ApiResponse<FileMetadata>>('/files/copy', {
      method: 'POST',
      body: JSON.stringify({ from, to }),
    });
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to copy');
    }
    return response.data;
  },

  // Search files
  async search(params: SearchParams): Promise<PaginatedResponse<SearchResult>> {
    const searchParams = new URLSearchParams();
    searchParams.set('path', params.path);
    searchParams.set('pattern', params.pattern);
    if (params.recursive !== undefined) searchParams.set('recursive', params.recursive.toString());
    if (params.page !== undefined) searchParams.set('page', params.page.toString());
    if (params.page_size !== undefined) searchParams.set('page_size', params.page_size.toString());

    const response = await fetchApi<ApiResponse<PaginatedResponse<SearchResult>>>(
      `/files/search?${searchParams.toString()}`
    );
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to search');
    }
    return response.data;
  },

  // Get directory stats
  async getDirStats(path: string): Promise<{ path: string; stats: DirStats }> {
    const searchParams = new URLSearchParams({ path });
    const response = await fetchApi<ApiResponse<{ path: string; stats: DirStats }>>(
      `/files/stats?${searchParams.toString()}`
    );
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get stats');
    }
    return response.data;
  },

  // Batch delete
  async batchDelete(paths: string[]): Promise<BatchResult> {
    const response = await fetchApi<ApiResponse<BatchResult>>('/files/batch/delete', {
      method: 'POST',
      body: JSON.stringify({ paths }),
    });
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to batch delete');
    }
    return response.data;
  },

  // Batch copy
  async batchCopy(operations: { from: string; to: string }[]): Promise<BatchResult> {
    const response = await fetchApi<ApiResponse<BatchResult>>('/files/batch/copy', {
      method: 'POST',
      body: JSON.stringify({ operations }),
    });
    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to batch copy');
    }
    return response.data;
  },
};
