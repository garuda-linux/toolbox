import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { protocol, net } from 'electron';
import { join } from 'node:path';
import { existsSync, readFileSync } from 'node:fs';
import { Logger } from '../logging/logging.js';
import { pathToFileURL } from 'node:url';
import { execSync } from 'node:child_process';
import { homedir } from 'node:os';

class AppIconModule implements AppModule {
  private readonly logger = Logger.getInstance();

  private pkgToIconMap = new Map<string, string>();
  private resolvedCache = new Map<string, string>();
  private negativeCache = new Set<string>();
  private currentTheme = 'hicolor';
  private fallbackIconPath = '';

  async enable({ app }: ModuleContext): Promise<void> {
    await app.whenReady();

    this.detectCurrentTheme();
    this.buildPackageIconMap();
    this.findFallbackIcon();
    this.setupIconProtocol();
  }

  private detectCurrentTheme(): void {
    try {
      const kdeglobals = join(homedir(), '.config/kdeglobals');
      if (existsSync(kdeglobals)) {
        const content = readFileSync(kdeglobals, 'utf8');
        const match = content.match(/\[Icons][\s\S]*?Theme=(.*?)\n/);
        if (match) {
          this.currentTheme = match[1].trim();
          this.logger.debug(`Detected system icon theme: ${this.currentTheme}`);
          return;
        }
      }
      const theme = execSync('gsettings get org.gnome.desktop.interface icon-theme 2>/dev/null')
        .toString()
        .trim()
        .replace(/'/g, '');
      if (theme) {
        this.currentTheme = theme;
        this.logger.debug(`Detected system icon theme: ${this.currentTheme}`);
      }
    } catch {
      this.currentTheme = 'hicolor';
    }
  }

  private findFallbackIcon(): void {
    const fallbacks = [
      `/usr/share/icons/${this.currentTheme}/48x48/status/image-missing.png`,
      `/usr/share/icons/${this.currentTheme}/scalable/status/image-missing.svg`,
      '/usr/share/icons/hicolor/48x48/status/image-missing.png',
      '/usr/share/icons/hicolor/scalable/apps/system-run.svg',
    ];
    for (const p of fallbacks) {
      if (existsSync(p)) {
        this.fallbackIconPath = p;
        break;
      }
    }
  }

  private buildPackageIconMap(): void {
    const xmlDirs = ['/usr/share/swcatalog/xml/'];
    const iconBaseDirs = ['/usr/share/swcatalog/icons/'];

    try {
      for (const xmlDir of xmlDirs) {
        if (!existsSync(xmlDir)) continue;
        const files = execSync(`find "${xmlDir}" -name "*.xml.gz" 2>/dev/null`).toString().split('\n').filter(Boolean);
        for (const file of files) {
          try {
            const output = execSync(`gzip -dc "${file}"`, { maxBuffer: 50 * 1024 * 1024 }).toString();
            const components = output.match(/<component[\s\S]*?<\/component>/g) || [];
            for (const comp of components) {
              const pkgMatch = comp.match(/<pkgname>(.*?)<\/pkgname>/);
              if (!pkgMatch) continue;
              const pkgname = pkgMatch[1];
              const iconMatch =
                comp.match(/<icon[^>]*type="cached"[^>]*>(.*?)<\/icon>/) ||
                comp.match(/<icon[^>]*type="stock"[^>]*>(.*?)<\/icon>/);

              if (iconMatch) {
                const iconName = iconMatch[1];
                for (const base of iconBaseDirs) {
                  for (const size of ['64x64', '128x128', '48x48']) {
                    const fullPath = join(base, size, iconName);
                    if (existsSync(fullPath)) {
                      this.pkgToIconMap.set(pkgname, fullPath);
                      break;
                    }
                    if (existsSync(fullPath + '.png')) {
                      this.pkgToIconMap.set(pkgname, fullPath + '.png');
                      break;
                    }
                  }
                  if (this.pkgToIconMap.has(pkgname)) break;
                }
                if (!this.pkgToIconMap.has(pkgname)) this.pkgToIconMap.set(pkgname, iconName);
              }
            }
          } catch {
            // ignore
          }
        }
      }
    } catch {
      // ignore
    }
  }

  private setupIconProtocol(): void {
    protocol.handle('app-icon', async (request) => {
      const url = new URL(request.url);
      let target = decodeURIComponent(url.hostname + url.pathname);
      if (target.endsWith('/')) target = target.slice(0, -1);

      if (this.resolvedCache.has(target))
        return net.fetch(pathToFileURL(this.resolvedCache.get(target) as string).toString());
      if (this.negativeCache.has(target)) return this.serveFallback();

      let resolvedPath = '';

      // Package mapping
      if (target.startsWith('package/')) {
        const pkgname = target.replace('package/', '');
        const mapped = this.pkgToIconMap.get(pkgname);
        if (mapped?.startsWith('/') && existsSync(mapped)) {
          resolvedPath = mapped;
        } else {
          target = mapped || pkgname; // Try name resolution
        }
      }

      // Absolute path resolution
      if (!resolvedPath) {
        let absPath = target;
        if (absPath.startsWith('usr/') || absPath.startsWith('var/')) absPath = '/' + absPath;
        if (absPath.startsWith('/') && existsSync(absPath) && !absPath.endsWith('/')) {
          resolvedPath = absPath;
        }
      }

      // System theme & search paths
      if (!resolvedPath) {
        const searchPaths = [
          `/usr/share/icons/${this.currentTheme}/64x64/apps/`,
          `/usr/share/icons/${this.currentTheme}/48x48/apps/`,
          `/usr/share/icons/${this.currentTheme}/scalable/apps/`,
          '/usr/share/swcatalog/icons/archlinux-arch-core/64x64/',
          '/usr/share/icons/hicolor/64x64/apps/',
          '/usr/share/icons/hicolor/scalable/apps/',
        ];

        for (const basePath of searchPaths) {
          const fullPath = target.includes('.') ? join(basePath, target) : join(basePath, target + '.png');
          if (existsSync(fullPath)) {
            resolvedPath = fullPath;
            break;
          }
          if (!target.includes('.')) {
            const svgPath = join(basePath, target + '.svg');
            if (existsSync(svgPath)) {
              resolvedPath = svgPath;
              break;
            }
          }
        }
      }

      if (resolvedPath) {
        this.resolvedCache.set(url.hostname + url.pathname, resolvedPath);
        return net.fetch(pathToFileURL(resolvedPath).toString());
      }

      this.negativeCache.add(url.hostname + url.pathname);
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
