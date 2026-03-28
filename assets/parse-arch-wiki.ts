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

    pkgname: selectedPkg,
    aur: pkgs.length === 0,
  };
}

/**
 * Enrich the hardcoded gaming package lists with AppStream icons
 */

/**
 * Enrich the setup wizard package lists with AppStream icons
 */

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
