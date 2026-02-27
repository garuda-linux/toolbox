import { readdirSync, statSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';
import pkg from './package.json' with { type: 'json' };

/**
 * Get list of files from each workspace using simple Node.js fs functions
 */
async function getListOfFilesFromEachWorkspace() {
  const allFilesToInclude = [];
  const packagesDir = join(process.cwd(), 'packages');

  try {
    // List all directories in packages folder
    const packageDirs = readdirSync(packagesDir).filter((dir) => {
      const dirPath = join(packagesDir, dir);
      return statSync(dirPath).isDirectory();
    });

    for (const packageDir of packageDirs) {
      const packagePath = join(packagesDir, packageDir);
      const packageJsonPath = join(packagePath, 'package.json');

      // Skip if no package.json exists
      if (!existsSync(packageJsonPath)) {
        continue;
      }

      try {
        // Read package.json to get package name and files
        const { default: workspacePkg } = await import(pathToFileURL(packageJsonPath), {
          with: { type: 'json' },
        });

        const packageName = workspacePkg.name;
        let patterns = workspacePkg.files || ['dist/**', 'package.json'];

        // Map patterns to node_modules location
        patterns = patterns.map((p) => join('node_modules', packageName, p));
        allFilesToInclude.push(...patterns);
      } catch (error) {
        console.warn(`Could not process package ${packageDir}:`, error.message);
      }
    }
  } catch (error) {
    console.warn('Could not read packages directory:', error.message);
  }

  return allFilesToInclude;
}

/**
 * @type {import('electron-builder').Configuration}
 * @see https://www.electron.build/configuration
 */
export default {
  productName: 'garuda-toolbox',
  appId: 'org.garudalinux.toolbox',
  copyright: 'GPL-3.0',

  compression: 'normal',
  removePackageScripts: true,

  directories: {
    output: 'dist',
    buildResources: 'buildResources',
  },

  generateUpdatesFilesForAllChannels: false,

  artifactName: '${productName}-${version}-${os}-${arch}.${ext}',
  files: ['LICENSE*', pkg.main, '!node_modules/@app/**', ...(await getListOfFilesFromEachWorkspace())],
};
