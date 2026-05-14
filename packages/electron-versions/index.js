import { execFileSync } from 'node:child_process';
import electronPath from 'electron';

function getElectronBinaryPath() {
  if (typeof electronPath === 'string') {
    return electronPath;
  }

  if (electronPath && typeof electronPath.default === 'string') {
    return electronPath.default;
  }

  throw new TypeError('Unable to resolve Electron binary path');
}

function getElectronEnv() {
  const output = execFileSync(getElectronBinaryPath(), ['-p', 'JSON.stringify(process.versions)'], {
    encoding: 'utf-8',
    env: {
      ...process.env,
      ELECTRON_RUN_AS_NODE: '1',
    },
  });

  return JSON.parse(output.trim());
}

function createElectronEnvLoader() {
  let inMemoryCache = null;

  return () => {
    if (inMemoryCache) {
      return inMemoryCache;
    }

    inMemoryCache = getElectronEnv();
    return inMemoryCache;
  };
}

const envLoader = createElectronEnvLoader();

export function getElectronVersions() {
  return envLoader();
}

export function getChromeVersion() {
  return getElectronVersions().chrome;
}

export function getChromeMajorVersion() {
  return getMajorVersion(getChromeVersion());
}

export function getNodeVersion() {
  return getElectronVersions().node;
}

export function getNodeMajorVersion() {
  return getMajorVersion(getNodeVersion());
}

function getMajorVersion(version) {
  return Number.parseInt(version.split('.')[0]);
}
