export interface BootEntry {
  label: string;
  id: string;
  isSubmenu: boolean;
  parentLabel?: string;
}

export interface PlymouthInfo {
  themes: string[];
  currentTheme: string;
  isInstalled: boolean;
}

export interface PartitionInfo {
  name: string;
  size: string;
  fstype: string;
  mountpoint: string;
  label: string;
}
