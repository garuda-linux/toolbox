import { execSync } from 'node:child_process';
import { gamingPackageLists } from '../packages/renderer/src/app/components/gaming/package-lists';
import { readFileSync } from 'node:fs';
import type { FullPackageDefinition } from '../packages/renderer/src/app/components/gaming/interfaces';

const inRepo: string[] = execSync('pacman -Ssq').toString().split('\n');
const inAur: string[] = execSync('paru -Slaq', { maxBuffer: 10240000 }).toString().split('\n');

console.log(`In total ${inRepo.length} packages were found in the repositories`);
console.log(`In total ${inAur.length} packages were found in the AUR\n`);

let total = 0;
let totalMissing = 0;

function checkMissing(missing: string[], missingAur: string[]) {
  if (missing.length > 0) {
    console.log(`\n${missing.length} packages of ${total} are missing from the repositories`);
    console.log(missing.join('\n'));
  }
  if (missingAur.length > 0) {
    console.log(`\n${missingAur.length} packages are only available in the AUR`);
    console.log(missingAur.join('\n'));
  }
}

function checkGaming(): number {
  console.log('Checking package availability of gaming packages in repositories');
  const missing: string[] = [];
  const missingAur: string[] = [];

  for (const tab of gamingPackageLists) {
    console.log(`Checking tab: ${tab.name}`);

    for (const section of tab.sections) {
      if (!inRepo.includes(section.pkgname[0])) {
        missing.push(section.pkgname[0]);
      }
      total++;
    }
  }

  checkMissing(missing, missingAur);
  return missing.length;
}

function checkGeneral(): number {
  console.log('\nChecking package availability of all other packages in the repositories');
  const toCheck: string[] = ['documents', 'internet', 'multimedia', 'other', 'science', 'security'];
  const missing: string[] = [];
  const missingAur: string[] = [];

  for (const section of toCheck) {
    console.log(`Checking section: ${section}`);
    const resourcePath = `./assets/parsed/${section}-repo.json`;
    const packages: FullPackageDefinition[] = JSON.parse(readFileSync(resourcePath, 'utf8'));
    for (const pkg of packages) {
      if (!inRepo.includes(pkg.pkgname[0]) && !inAur.includes(pkg.pkgname[0])) {
        missing.push(pkg.pkgname[0]);
      } else if (!inRepo.includes(pkg.pkgname[0]) && inAur.includes(pkg.pkgname[0])) {
        missingAur.push(pkg.pkgname[0]);
      }
      total++;
    }
  }

  checkMissing(missing, missingAur);
  return missing.length;
}

totalMissing += checkGaming();
totalMissing += checkGeneral();

if (totalMissing > 0) process.exit(1);
console.log('\nAll packages are available in the repositories and AUR!');
