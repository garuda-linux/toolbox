import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { net, protocol } from 'electron';
import { join } from 'node:path';
import { access, readdir, readFile } from 'node:fs/promises';
import { Logger } from '../logging/logging.js';
import { pathToFileURL } from 'node:url';
import { exec } from 'node:child_process';
import { promisify } from 'node:util';
import { homedir } from 'node:os';

const execAsync = promisify(exec);

async function exists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

class AppIconModule implements AppModule {
  private readonly logger = Logger.getInstance();

  private resolvedCache = new Map<string, string>();
  private negativeCache = new Set<string>();
  private currentTheme = 'hicolor';
  private activeThemes: string[] = ['hicolor'];
  private fallbackIconPath = '';
  private swcatalogDirs: string[] = [];
  private searchPaths: string[] = [];
  private dirCache = new Map<string, Set<string> | null>();

  private initPromise: Promise<void> | null = null;

  async enable({ app }: ModuleContext): Promise<void> {
    await app.whenReady();

    this.setupIconProtocol();
    this.initPromise = this.initialize();
  }

  private async initialize(): Promise<void> {
    await this.detectCurrentTheme();
    await this.findFallbackIcon();
    this.buildSearchPaths();
  }

  private buildSearchPaths(): void {
    const sizes = ['scalable', '256x256', '128x128', '96x96', '64x64', '48x48', '32x32', '16x16'];

    for (const theme of this.activeThemes) {
      for (const size of sizes) {
        this.searchPaths.push(join('/usr/share/icons/', theme, size, 'apps/'));
        this.searchPaths.push(join('/usr/share/icons/', theme, size, 'mimetypes/'));
        this.searchPaths.push(join('/usr/share/icons/', theme, size, 'devices/'));
        this.searchPaths.push(join('/usr/share/icons/', theme, size, 'categories/'));
        this.searchPaths.push(join('/usr/share/icons/', theme, size, 'places/'));
        this.searchPaths.push(join('/usr/share/icons/', theme, size, 'status/'));
      }
    }

    for (const dir of this.swcatalogDirs) {
      this.searchPaths.push(join(dir, '128x128/'));
      this.searchPaths.push(join(dir, '64x64/'));
      this.searchPaths.push(join(dir, '48x48/'));
    }

    this.searchPaths.push('/usr/share/pixmaps/');
  }

  private async getDirContents(dirPath: string): Promise<Set<string> | null> {
    if (this.dirCache.has(dirPath)) {
      return this.dirCache.get(dirPath) || null;
    }
    try {
      const files = await readdir(dirPath);
      const set = new Set(files);
      this.dirCache.set(dirPath, set);
      return set;
    } catch {
      this.dirCache.set(dirPath, null);
      return null;
    }
  }

  private async getThemeInheritance(themeName: string, visited = new Set<string>()): Promise<string[]> {
    if (visited.has(themeName)) return [];
    visited.add(themeName);
    const themes = [themeName];
    try {
      const indexPath = join('/usr/share/icons/', themeName, 'index.theme');
      if (await exists(indexPath)) {
        const content = await readFile(indexPath, 'utf8');
        const match = content.match(/^Inherits=(.*)$/m);
        if (match) {
          const parents = match[1].split(',').map((s) => s.trim());
          for (const parent of parents) {
            themes.push(...(await this.getThemeInheritance(parent, visited)));
          }
        }
      }
    } catch {
      // ignore
    }
    return themes;
  }

  private async detectCurrentTheme(): Promise<void> {
    try {
      const kdeglobals = join(homedir(), '.config/kdeglobals');
      if (await exists(kdeglobals)) {
        const content = await readFile(kdeglobals, 'utf8');
        const match = content.match(/\[Icons\][\s\S]*?Theme=(.*?)\n/);
        if (match) {
          this.currentTheme = match[1].trim();
          this.logger.debug(`Detected system icon theme: ${this.currentTheme}`);
        }
      } else {
        const { stdout } = await execAsync('gsettings get org.gnome.desktop.interface icon-theme 2>/dev/null');
        const theme = stdout.trim().replace(/'/g, '');
        if (theme) {
          this.currentTheme = theme;
          this.logger.debug(`Detected system icon theme: ${this.currentTheme}`);
        }
      }
    } catch {
      this.currentTheme = 'hicolor';
    }

    this.activeThemes = await this.getThemeInheritance(this.currentTheme);
    if (!this.activeThemes.includes('hicolor')) {
      this.activeThemes.push('hicolor');
    }
  }

  private async findFallbackIcon(): Promise<void> {
    const fallbacks = [
      `/usr/share/icons/${this.currentTheme}/48x48/status/image-missing.png`,
      `/usr/share/icons/${this.currentTheme}/scalable/status/image-missing.svg`,
      `/usr/share/icons/${this.currentTheme}/status/64/image-missing.svg`,
      `/usr/share/icons/${this.currentTheme}/scalable/apps/system-run.svg`,
      '/usr/share/icons/Adwaita/scalable/status/image-missing.svg',
      '/usr/share/icons/Adwaita/scalable/mimetypes/application-x-executable.svg',
      '/usr/share/icons/hicolor/48x48/status/image-missing.png',
      '/usr/share/icons/hicolor/scalable/apps/system-run.svg',
    ];
    for (const p of fallbacks) {
      if (await exists(p)) {
        this.fallbackIconPath = p;
        break;
      }
    }
  }

  private setupIconProtocol(): void {
    protocol.handle('app-icon', async (request) => {
      if (this.initPromise) {
        await this.initPromise;
      }

      const url = new URL(request.url);
      let target = decodeURIComponent(url.hostname + url.pathname);
      if (target.endsWith('/')) target = target.slice(0, -1);

      if (this.resolvedCache.has(target))
        return net.fetch(pathToFileURL(this.resolvedCache.get(target) as string).toString());
      if (this.negativeCache.has(target)) {
        if (url.searchParams.get('fallback') === 'false') {
          return new Response('Not found', { status: 404 });
        }
        return this.serveFallback();
      }

      let resolvedPath = '';
      const originalTarget = target;
      let isPackageRequest = false;

      if (target.startsWith('package/')) {
        target = target.replace('package/', '');
        isPackageRequest = true;
      }

      const baseName =
        target
          .split('/')
          .pop()
          ?.replace(/\.(png|svg)$/i, '') || target;
      const iconNamesToSearch = [baseName];
      if (baseName.includes('_')) {
        const parts = baseName.split('_');
        if (parts.length >= 2) {
          const secondPart = parts.slice(1).join('_');
          if (!iconNamesToSearch.includes(secondPart)) iconNamesToSearch.push(secondPart);
          const firstPart = parts[0];
          if (!iconNamesToSearch.includes(firstPart)) iconNamesToSearch.push(firstPart);
        }
      }

      const searchResults = await Promise.all(
        this.searchPaths.map(async (basePath) => {
          const files = await this.getDirContents(basePath);
          if (!files) return null;

          for (const name of iconNamesToSearch) {
            if (files.has(name + '.svg')) {
              return join(basePath, name + '.svg');
            }
            if (files.has(name + '.png')) {
              return join(basePath, name + '.png');
            }
          }

          if (isPackageRequest && basePath.includes('swcatalog/icons')) {
            const prefix = baseName + '_';
            for (const file of files) {
              if (file.startsWith(prefix) && (file.endsWith('.png') || file.endsWith('.svg'))) {
                return join(basePath, file);
              }
            }
          }
          return null;
        }),
      );

      resolvedPath = searchResults.find((p) => p !== null) || '';

      if (!resolvedPath) {
        let absPath = target;
        if (absPath.startsWith('usr/') || absPath.startsWith('var/')) absPath = '/' + absPath;
        if (absPath.startsWith('/')) {
          if (await exists(absPath)) {
            resolvedPath = absPath;
          } else if (await exists(absPath + '.png')) {
            resolvedPath = absPath + '.png';
          } else if (await exists(absPath + '.svg')) {
            resolvedPath = absPath + '.svg';
          }
        }
      }

      if (resolvedPath) {
        this.resolvedCache.set(originalTarget, resolvedPath);
        return net.fetch(pathToFileURL(resolvedPath).toString());
      }

      this.negativeCache.add(originalTarget);
      if (url.searchParams.get('fallback') === 'false') {
        return new Response('Not found', { status: 404 });
      }
      return this.serveFallback();
    });
  }

  private serveFallback(): Promise<Response> {
    if (this.fallbackIconPath) return net.fetch(pathToFileURL(this.fallbackIconPath).toString());
    return Promise.resolve(new Response('Not found', { status: 404 }));
  }
}

export function createAppIconModule() {
  return new AppIconModule();
}
