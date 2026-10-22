/** The slice of the Modal JS SDK the bridge uses, so tests can supply a mock. */

export interface ProcessLike {
  stdout: AsyncIterable<string> & { readText(): Promise<string> };
  stderr: AsyncIterable<string> & { readText(): Promise<string> };
  wait(): Promise<number>;
}

export interface ExecParams {
  workdir?: string;
  env?: Record<string, string>;
}

export interface ImageLike {
  readonly imageId: string;
  dockerfileCommands(commands: string[]): ImageLike;
  publish(name: string): Promise<void>;
}

export interface SandboxLike {
  readonly sandboxId: string;
  exec(argv: string[], params?: ExecParams): Promise<ProcessLike>;
  readonly filesystem: {
    copyFromLocal(localPath: string, remotePath: string): Promise<void>;
    copyToLocal(remotePath: string, localPath: string): Promise<void>;
  };
  snapshotFilesystem(params?: { timeoutMs?: number }): Promise<ImageLike>;
  getTags(): Promise<Record<string, string>>;
  terminate(): Promise<void>;
}

export interface AppLike {
  readonly appId: string;
}

export interface CreateParams {
  gpu?: string;
  cpu?: number;
  memoryMiB?: number;
  timeoutMs?: number;
  tags?: Record<string, string>;
}

export interface ModalLike {
  apps: { fromName(name: string, params: { createIfMissing: boolean }): Promise<AppLike> };
  images: {
    fromName(name: string): Promise<ImageLike>;
    fromRegistry(tag: string): ImageLike;
  };
  sandboxes: {
    create(app: AppLike, image: ImageLike, params: CreateParams): Promise<SandboxLike>;
    list(params: { appId: string; tags: Record<string, string> }): AsyncIterable<SandboxLike>;
    fromId(id: string): Promise<SandboxLike>;
  };
}
