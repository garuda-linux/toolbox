import { writeFileSync, mkdirSync, existsSync, readFileSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { join } from 'node:path';

/**
 * ARCH WIKI SYNC & PARSE SCRIPT
 * Consolidates fetching raw wiki data and parsing it into structured JSON.
 * Enriches data with system AppStream icons where available.
 */

const CATEGORIES = ['Games', 'Documents', 'Internet', 'Multimedia', 'Science', 'Security', 'Utilities', 'Other'];

const BASE_URL = 'https://wiki.archlinux.org/index.php?title=';
const RAW_SUFFIX = '&action=raw';

interface Package {
  name: string;
  description?: string;
  url?: string;
  icon?: string;
  pkgname: string[];
  aur?: boolean;
}

/**
 * Clean Wiki markup from a string
 */
function cleanWiki(text: string): string {
  if (!text) return '';
  return text
    .replace(/\{\{Grp\|kde-games}}/gi, 'Part of KDE Games')
    .replace(/\{\{Dead link\|.*?}}/gi, '')
    .replace(/\{\{man\|.*?\|(.*?)(?:\|.*?)*}}/gi, '$1')
    .replace(/\{\{Pkg\|(.*?)(?:\|.*?)*}}/gi, '$1')
    .replace(/\{\{AUR\|(.*?)(?:\|.*?)*}}/gi, '$1')
    .replace(/\{\{=}}/g, '=')
    .replace(/\{\{ic\|(.*?)}}/gi, '$1')
    .replace(/\{\{Grp\|(.*?)}}/gi, '$1')
    .replace(/\{\{.*?}}/g, '')
    .replace(/\[\[Wikipedia:(.*?)\|(.*?)]]/g, '$2')
    .replace(/\[\[(.*?)\|(.*?)]]/g, '$2')
    .replace(/\[\[(.*?)]]/g, '$1')
    .replace(/\[https?:\/\/.*?\s+(.*?)]/g, '$1')
    .replace(/'''/g, '')
    .replace(/''/g, '')
    .replace(/&amp;/g, '&')
    .replace(/\s+/g, ' ')
    .trim();
}

/**
 * Build a map of package names to AppStream icon names/paths
 */
function getAppStreamIconMap(): Map<string, string> {
  const iconMap = new Map<string, string>();
  const xmlSearchPaths = ['/usr/share/swcatalog/xml/'];
  const iconSearchPaths = [
    '/usr/share/swcatalog/icons/archlinux-arch-core/64x64/',
    '/usr/share/swcatalog/icons/archlinux-arch-extra/64x64/',
    '/usr/share/swcatalog/icons/archlinux-arch-multilib/64x64/',
    '/usr/share/swcatalog/icons/archlinux-arch-core/128x128/',
    '/usr/share/swcatalog/icons/archlinux-arch-extra/128x128/',
    '/usr/share/swcatalog/icons/archlinux-arch-multilib/128x128/',
  ];

  try {
    const files: string[] = [];
    for (const dir of xmlSearchPaths) {
      try {
        if (!existsSync(dir)) {
          console.log(`Directory does not exist: ${dir}`);
          continue;
        }
        const found = execSync(`find "${dir}" -name "*.xml.gz" 2>/dev/null`).toString().split('\n').filter(Boolean);
        console.log(`Found ${found.length} files in ${dir}`);
        files.push(...found);
      } catch (e) {
        console.warn(`Error searching ${dir}:`, e.message);
      }
    }

    if (files.length === 0) {
      console.warn('No AppStream XML files found! Icon map will be empty.');
      return iconMap;
    }

    for (const file of files) {
      try {
        console.log(`Processing ${file}...`);
        const output = execSync(`gzip -dc "${file}"`, { maxBuffer: 100 * 1024 * 1024 }).toString();
        const components = output.match(/<component[\s\S]*?<\/component>/g) || [];
        console.log(`Found ${components.length} components in ${file}`);

        for (const comp of components) {
          const pkgMatch = comp.match(/<pkgname>(.*?)<\/pkgname>/);
          if (!pkgMatch) continue;

          const pkgname = pkgMatch[1];
          const cachedMatch = comp.match(/<icon[^>]*type="cached"[^>]*>(.*?)<\/icon>/);
          const stockMatch = comp.match(/<icon[^>]*type="stock"[^>]*>(.*?)<\/icon>/);

          const iconName = cachedMatch?.[1] || stockMatch?.[1];
          if (!iconName) continue;

          let absolutePath = '';
          for (const iconDir of iconSearchPaths) {
            const fullPath = join(iconDir, iconName);
            if (existsSync(fullPath)) {
              absolutePath = fullPath;
              break;
            }
            if (!iconName.includes('.')) {
              if (existsSync(fullPath + '.png')) {
                absolutePath = fullPath + '.png';
                break;
              }
              if (existsSync(fullPath + '.svg')) {
                absolutePath = fullPath + '.svg';
                break;
              }
            }
          }

          if (absolutePath && !iconMap.has(pkgname)) {
            iconMap.set(pkgname, absolutePath);
          } else if (iconName && !iconMap.has(pkgname)) {
            // If we have an icon name but couldn't find the absolute path, still keep the name
            iconMap.set(pkgname, iconName);
          }
        }
      } catch (e) {
        console.warn(`Error processing ${file}:`, e.message);
      }
    }
  } catch (e) {
    console.warn('Error building AppStream map:', e);
  }
  console.log(`Final AppStream icon map has ${iconMap.size} entries.`);
  return iconMap;
}

/**
 * Parse an App entry from wiki source
 */
function parseAppEntry(line: string): Package | null {
  const appMatch = line.match(/\{\{App\|(.*)}}/i);
  if (!appMatch) return null;

  const content = appMatch[1];
  const parts: string[] = [];
  let current = '';
  let depth = 0;

  for (const char of content) {
    if (char === '{' || char === '[') depth++;
    if (char === '}' || char === ']') depth--;

    if (char === '|' && depth === 0) {
      parts.push(current);
      current = '';
    } else {
      current += char;
    }
  }
  parts.push(current);

  if (parts.length < 4) return null;

  const name = cleanWiki(parts[0]);
  const description = cleanWiki(parts[1]);
  const url = cleanWiki(parts[2]);
  const pkgInfo = parts.slice(3).join('|');

  const pkgs: string[] = [];
  const aurPkgs: string[] = [];

  const pkgMatches = pkgInfo.matchAll(/\{\{Pkg\|(.*?)}}/gi);
  for (const m of pkgMatches) {
    pkgs.push(m[1].split('|')[0].trim());
  }

  const aurMatches = pkgInfo.matchAll(/\{\{AUR\|(.*?)}}/gi);
  for (const m of aurMatches) {
    aurPkgs.push(m[1].split('|')[0].trim());
  }

  if (pkgs.length === 0 && aurPkgs.length === 0) return null;
  const selectedPkg = pkgs.length > 0 ? [pkgs[0]] : [aurPkgs[0]];

  return {
    name,
    description,
    url: url.startsWith('http') ? url : undefined,
    icon: 'generic-dark.svg',
    pkgname: selectedPkg,
    aur: pkgs.length === 0,
  };
}

/**
 * Enrich the hardcoded gaming package lists with AppStream icons
 */
async function enrichGamingPackageLists(iconMap: Map<string, string>) {
  const gamingPath = join(process.cwd(), 'packages/renderer/src/app/components/gaming/package-lists.ts');
  if (!existsSync(gamingPath)) return;

  console.log('Enriching gaming package lists with AppStream icons...');
  const content = readFileSync(gamingPath, 'utf8');
  const lines = content.split('\n');
  let updatedCount = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    // We only target lines that have the generic icon
    if (!line.includes("icon: 'generic-dark.svg'") && !line.includes('icon: "generic-dark.svg"')) continue;

    let pkgname: string | undefined;

    // Extract pkgname from current line
    let match = line.match(/pkgname: \['([^']+)']/);
    if (!match) match = line.match(/pkgname: \["([^"]+)"]/);

    if (match) {
      pkgname = match[1];
    } else {
      for (let j = Math.max(0, i - 3); j < i; j++) {
        let pkgMatch = lines[j].match(/pkgname: \['([^']+)']/);
        if (!pkgMatch) pkgMatch = lines[j].match(/pkgname: \["([^"]+)"]/);
        if (pkgMatch) {
          pkgname = pkgMatch[1];
          break;
        }
      }
    }

    if (pkgname) {
      let betterIcon = iconMap.get(pkgname);
      if (!betterIcon) {
        for (const suffix of ['-git', '-bin']) {
          if (pkgname.endsWith(suffix)) {
            betterIcon = iconMap.get(pkgname.replace(suffix, ''));
            if (betterIcon) break;
          }
        }
      }

      if (betterIcon) {
        const isSingleQuote = line.includes("icon: 'generic-dark.svg'");
        const replacement = isSingleQuote ? `icon: '${betterIcon}'` : `icon: "${betterIcon}"`;

        lines[i] = line.replace(/icon: (['"])generic-dark\.svg\1/, replacement);
        updatedCount++;
      }
    }
  }

  if (updatedCount > 0) {
    writeFileSync(gamingPath, lines.join('\n'));
    console.log(`Successfully updated ${updatedCount} icons in gaming package lists.`);
  }
}

/**
 * Enrich the setup wizard package lists with AppStream icons
 */
async function enrichSetupWizardPackageLists(iconMap: Map<string, string>) {
  const setupPath = join(process.cwd(), 'packages/renderer/src/app/components/setup-wizard/setup-wizard.data.ts');
  if (!existsSync(setupPath)) return;

  console.log('Enriching setup wizard package lists with AppStream icons...');
  const content = readFileSync(setupPath, 'utf8');
  const lines = content.split('\n');
  let updatedCount = 0;
  let currentPkgname: string | undefined;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Track the package name for the current item block
    const pkgMatch = line.match(/packages:\s*\[\s*['"]([^'"]+)['"]/);
    if (pkgMatch) {
      currentPkgname = pkgMatch[1];
    }

    // Target icon fields
    if (!line.includes("icon: '") && !line.includes('icon: "')) continue;

    if (currentPkgname) {
      let betterIcon = iconMap.get(currentPkgname);
      // Try common variations
      if (!betterIcon) {
        for (const suffix of ['-git', '-bin', '-beta', '-launcher', '-desktop']) {
          if (currentPkgname.endsWith(suffix)) {
            betterIcon = iconMap.get(currentPkgname.replace(suffix, ''));
            if (betterIcon) break;
          }
        }
      }

      if (betterIcon) {
        const currentIconMatch = line.match(/icon:\s*['"]([^'"]+)['"]/);
        const currentIcon = currentIconMatch ? currentIconMatch[1] : '';

        // Only update if it's currently a generic icon or if the betterIcon is an absolute path (more specific)
        if (currentIcon.includes('generic') || (betterIcon.startsWith('/') && !currentIcon.startsWith('/'))) {
          console.log(`Matching icon for ${currentPkgname}: ${betterIcon}`);
          const isSingleQuote = line.includes("icon: '");
          const quote = isSingleQuote ? "'" : '"';

          lines[i] = line.replace(/icon:\s*['"][^'"]+['"]/, `icon: ${quote}${betterIcon}${quote}`);
          updatedCount++;
        }
      }
    }
  }

  if (updatedCount > 0) {
    writeFileSync(setupPath, lines.join('\n'));
    console.log(`Successfully updated ${updatedCount} icons in setup wizard data.`);
  } else {
    console.log('No icons were updated in setup wizard data.');
  }
}

async function main() {
  const mode = process.argv.includes('--local') ? 'local' : 'fetch';
  const includeAur = process.argv.includes('--include-aur');
  const updateGaming = process.argv.includes('--gaming');
  const updateSetup = process.argv.includes('--setup');
  console.log(
    `Starting ArchWiki sync (mode: ${mode}, includeAur: ${includeAur}, updateGaming: ${updateGaming}, updateSetup: ${updateSetup})...`,
  );

  const sourcesDir = 'assets/sources';
  const parsedDir = 'assets/parsed';

  if (!existsSync(sourcesDir)) mkdirSync(sourcesDir, { recursive: true });
  if (!existsSync(parsedDir)) mkdirSync(parsedDir, { recursive: true });

  const iconMap = getAppStreamIconMap();
  console.log(`AppStream icon map built with ${iconMap.size} entries.`);

  if (updateGaming) {
    await enrichGamingPackageLists(iconMap);
  }

  if (updateSetup || updateGaming) {
    await enrichSetupWizardPackageLists(iconMap);
  }

  if (!updateGaming && !updateSetup) {
    let inRepo: string[] = [];
    try {
      inRepo = execSync('pacman -Ssq')
        .toString()
        .split('\n')
        .map((p) => p.trim())
        .filter((p) => p);
    } catch {
      console.warn('Could not run pacman -Ssq, will not filter by repository availability.');
    }

    const hasPacman = inRepo.length > 0;
    for (const cat of CATEGORIES) {
      let text: string | NodeJS.ArrayBufferView<ArrayBufferLike>;
      const sourceFile = join(sourcesDir, `${cat.toLowerCase()}.txt`);

      if (mode === 'fetch') {
        const pageTitle = cat === 'Games' ? 'List_of_games' : `List_of_applications/${cat}`;
        const url = `${BASE_URL}${pageTitle}${RAW_SUFFIX}`;
        console.log(`Fetching ${cat} from ${url}...`);

        try {
          const response = await fetch(url);
          if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);

          text = await response.text();
          writeFileSync(sourceFile, text);
        } catch (error) {
          console.error(`Failed to fetch ${cat}:`, error);
          continue;
        }
      } else {
        if (!existsSync(sourceFile)) {
          console.warn(`Source file missing for ${cat}: ${sourceFile}`);
          continue;
        }
        text = readFileSync(sourceFile, 'utf8');
      }

      const lines = text.split('\n');
      const apps: Package[] = [];
      let skippedCount = 0;

      for (const line of lines) {
        const app = parseAppEntry(line);
        if (app) {
          const pkgname = app.pkgname[0];

          if (hasPacman) {
            const foundInRepo = inRepo.includes(pkgname);
            if (foundInRepo) {
              app.aur = false;
            } else {
              app.aur = true;
              if (!includeAur) {
                skippedCount++;
                continue;
              }
            }
          } else if (!includeAur && app.aur) {
            skippedCount++;
            continue;
          }

          let appStreamIcon = iconMap.get(pkgname);

          if (!appStreamIcon) {
            for (const suffix of ['-git', '-bin']) {
              if (pkgname.endsWith(suffix)) {
                appStreamIcon = iconMap.get(pkgname.replace(suffix, ''));
                if (appStreamIcon) break;
              }
            }
          }

          if (appStreamIcon) app.icon = appStreamIcon;

          apps.push(app);
        }
      }

      apps.sort((a, b) => a.name.localeCompare(b.name));
      writeFileSync(join(parsedDir, `${cat.toLowerCase()}-repo.json`), JSON.stringify(apps, null, 2));
      console.log(
        `Processed ${cat}: ${apps.length} applications found${skippedCount > 0 ? ` (${skippedCount} non-repo packages skipped)` : ''}.`,
      );
    }

    console.log('Sync complete!');
  }
}

void main();
