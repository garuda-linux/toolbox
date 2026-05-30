import { PartitionInfo } from '../boot-options/types';

export type BootRepairAction = 'reinstall' | 'repair' | 'backup' | 'restore';
export type BootInstallTarget = 'mbr' | 'root' | 'esp';
export type BackupRestoreTarget = 'mbr' | 'pbr';

export interface PartitionWithPartType extends PartitionInfo {
  parttype: string;
}

export interface DiskInfo {
  name: string;
  size: string;
  label: string;
  model: string;
}
