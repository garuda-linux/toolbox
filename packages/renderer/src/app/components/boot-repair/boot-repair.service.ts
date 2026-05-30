import { inject, Injectable } from '@angular/core';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { Logger } from '../../logging/logging';
import { DiskInfo, PartitionWithPartType } from './types';

@Injectable({
  providedIn: 'root',
})
export class BootRepairService {
  private readonly taskManager = inject(TaskManagerService);
  private readonly logger = Logger.getInstance();

  DISK_NAME_PATTERN = /^(?:[shv]d[a-z]+|xvd[a-z]+|nvme\d+n\d+|mmcblk\d+|zram\d+)$/i;
  PARTITION_NAME_PATTERN = /^(?:[shv]d[a-z]+\d+|xvd[a-z]+\d+|nvme\d+n\d+p\d+|mmcblk\d+p\d+)$/i;

  async getDisks(): Promise<DiskInfo[]> {
    const cmd = 'lsblk -J -o NAME,SIZE,LABEL,MODEL -d -e 2,11 -x NAME 2>/dev/null';
    const result = await this.taskManager.executeAndWaitBash(cmd);

    if (result.code !== 0 || !result.stdout.trim()) {
      this.logger.error('Failed to get disks');
      return [];
    }

    try {
      const data = JSON.parse(result.stdout);
      const disks: DiskInfo[] = [];

      if (data.blockdevices && Array.isArray(data.blockdevices)) {
        for (const device of data.blockdevices) {
          const isDisk = this.DISK_NAME_PATTERN.test(device.name);
          if (isDisk) {
            disks.push({
              name: device.name || '',
              size: device.size || '',
              label: device.label || '',
              model: device.model || '',
            });
          }
        }
      }

      return disks;
    } catch (e) {
      this.logger.error(`Failed to parse lsblk JSON output for disks: ${e}`);
      return [];
    }
  }

  async getPartitionsWithPartType(): Promise<PartitionWithPartType[]> {
    const cmd = 'lsblk -J -o NAME,SIZE,FSTYPE,MOUNTPOINT,LABEL,PARTTYPE -e 2,11 -x NAME 2>/dev/null';
    const result = await this.taskManager.executeAndWaitBash(cmd);

    if (result.code !== 0 || !result.stdout.trim()) {
      this.logger.error('Failed to get partitions with parttype');
      return [];
    }

    try {
      const data = JSON.parse(result.stdout);
      const partitions: PartitionWithPartType[] = [];

      const extractPartitions = (device: any): void => {
        if (device.name) {
          const isPartition = this.PARTITION_NAME_PATTERN.test(device.name);
          if (isPartition) {
            partitions.push({
              name: device.name || '',
              size: device.size || '',
              fstype: device.fstype || '',
              mountpoint: device.mountpoint || '',
              label: device.label || '',
              parttype: (device.parttype || '').toLowerCase(),
            });
          }
        }
        if (device.children && Array.isArray(device.children)) {
          for (const child of device.children) {
            extractPartitions(child);
          }
        }
      };

      if (data.blockdevices && Array.isArray(data.blockdevices)) {
        for (const device of data.blockdevices) {
          extractPartitions(device);
        }
      }

      return partitions;
    } catch (e) {
      this.logger.error(`Failed to parse lsblk JSON output: ${e}`);
      return [];
    }
  }

  async isUefiSystem(): Promise<boolean> {
    const result = await this.taskManager.executeAndWaitBash('test -d /sys/firmware/efi');
    return result.code === 0;
  }

  async isEspPartition(partitionName: string): Promise<boolean> {
    const cmd = `lsblk -ln -o PARTTYPE /dev/${partitionName} | grep -qi 'c12a7328-f81f-11d2-ba4b-00a0c93ec93b'`;
    const result = await this.taskManager.executeAndWaitBash(cmd);
    return result.code === 0;
  }

  async isLinuxPartition(partitionName: string): Promise<boolean> {
    const cmd = `lsblk -ln -o PARTTYPE /dev/${partitionName} | grep -qEi '0x83|0fc63daf-8483-4772-8e79-3d69d8477de4|44479540-F297-41B2-9AF7-D131D5F0458A|4F68BCE3-E8CD-4DB1-96E7-FBCAF984B709'`;
    const result = await this.taskManager.executeAndWaitBash(cmd);
    return result.code === 0;
  }

  async isLuks(partitionName: string): Promise<boolean> {
    const cmd = `/sbin/cryptsetup isLuks /dev/${partitionName}`;
    const result = await this.taskManager.executeAndWaitBash(cmd);
    return result.code === 0;
  }

  async getArch(): Promise<string> {
    const result = await this.taskManager.executeAndWaitBash('uname -m');
    let arch = result.stdout.trim();
    if (arch === 'i686') {
      arch = 'i386';
    }
    return arch;
  }

  async getCurrentRootSource(): Promise<string> {
    const result = await this.taskManager.executeAndWaitBash("findmnt -no SOURCE / | sed 's/\\[.*\\]$//'");
    return result.stdout.trim();
  }

  async getLuksMapperPath(partitionPath: string): Promise<string> {
    const blkidResult = await this.taskManager.executeAndWaitBash(
      `blkid -s TYPE -s UUID -o export ${this.shellQuote(partitionPath)} 2>/dev/null || true`,
    );
    const blkidLines = blkidResult.stdout
      .trim()
      .split('\n')
      .filter((line) => line.length > 0);
    const typeLine = blkidLines.find((line) => line.startsWith('TYPE='));
    const uuidLine = blkidLines.find((line) => line.startsWith('UUID='));
    if (typeLine === 'TYPE=crypto_LUKS' && uuidLine) {
      const uuidFromBlkid = uuidLine.replace('UUID=', '').trim();
      if (uuidFromBlkid.length > 0) {
        return `/dev/mapper/luks-${uuidFromBlkid}`;
      }
    }

    const result = await this.taskManager.executeAndWaitBash(
      `cryptsetup luksUUID ${this.shellQuote(partitionPath)} 2>/dev/null || true`,
    );
    const uuid =
      result.stdout
        .trim()
        .split('\n')
        .find((line) => line.length > 0) || '';
    return uuid ? `/dev/mapper/luks-${uuid}` : '';
  }

  async isCurrentRootSelection(rootPartition: string): Promise<boolean> {
    if (!rootPartition) {
      return false;
    }

    const selectedNorm = this.normalizeDevicePath(rootPartition);
    const currentRootSource = await this.getCurrentRootSource();
    const currentRootPartition = await this.getCurrentRootPartition();

    if (!currentRootSource && !currentRootPartition) {
      return false;
    }

    const sourceNorm = this.normalizeDevicePath(currentRootSource || '');
    const partitionNorm = this.normalizeDevicePath(currentRootPartition || '');

    if (selectedNorm === sourceNorm || selectedNorm === partitionNorm) {
      return true;
    }

    if (currentRootSource.startsWith('/dev/mapper/')) {
      const backingDevice = await this.getMapperBackingDevice(currentRootSource);
      const backingNorm = this.normalizeDevicePath(backingDevice || '');
      if (backingNorm && selectedNorm === backingNorm) {
        return true;
      }
    }

    return sourceNorm === 'mapper/rootfs';
  }

  async getCurrentRootPartition(): Promise<string> {
    const result = await this.taskManager.executeAndWaitBash("findmnt -no SOURCE / | sed 's/\\[.*\\]$//'");
    const source = result.stdout.trim();
    if (!source) {
      return '';
    }

    if (source.startsWith('/dev/mapper/')) {
      const backingDevice = await this.getMapperBackingDevice(source);
      return backingDevice || source;
    }

    return source;
  }

  private shellQuote(value: string): string {
    return `'${value.replaceAll("'", `'"'"'`)}'`;
  }

  private normalizeDevicePath(device: string): string {
    const normalized = device
      .trim()
      .replace(/\[.*]$/, '')
      .replace(/^\/dev\//, '')
      .trim();

    if (normalized.startsWith('mapper/')) {
      return normalized.replace(/^mapper\//, '');
    }

    return normalized;
  }

  private async getMapperBackingDevice(mapperPath: string): Promise<string> {
    const escapedMapper = this.shellQuote(mapperPath);
    const directParent = await this.taskManager.executeAndWaitBash(
      `lsblk -ndo PKNAME ${escapedMapper} 2>/dev/null | sed -n '1p' || true`,
    );
    const direct = directParent.stdout
      .trim()
      .split('\n')
      .find((line) => line.length > 0);
    if (direct) {
      return `/dev/${direct}`;
    }

    const linkParent = await this.taskManager.executeAndWaitBash(
      `lsblk -ndo PKNAME "$(readlink -f ${escapedMapper})" 2>/dev/null | sed -n '1p' || true`,
    );
    const linked = linkParent.stdout
      .trim()
      .split('\n')
      .find((line) => line.length > 0);
    if (linked) {
      return `/dev/${linked}`;
    }

    const mapperNameForSysfs = mapperPath.replace('/dev/mapper/', '');
    const resolvedNodeResult = await this.taskManager.executeAndWaitBash(
      `basename "$(readlink -f ${escapedMapper})" 2>/dev/null || true`,
    );
    const resolvedNode = resolvedNodeResult.stdout
      .trim()
      .split('\n')
      .find((line) => line.length > 0);

    const sysfsCandidates = [mapperNameForSysfs, resolvedNode].filter((name): name is string => Boolean(name));
    for (const nodeName of sysfsCandidates) {
      const sysfsParent = await this.taskManager.executeAndWaitBash(
        `ls -1 /sys/class/block/${this.shellQuote(nodeName)}/slaves 2>/dev/null | sed -n '1p' || true`,
      );
      const sysParent = sysfsParent.stdout
        .trim()
        .split('\n')
        .find((line) => line.length > 0);
      if (sysParent) {
        return `/dev/${sysParent}`;
      }
    }

    return '';
  }

  private async readFstab(path: string): Promise<Map<string, string>> {
    const cmd = `cat ${this.shellQuote(path + '/etc/fstab')} 2>/dev/null || true`;
    const result = await this.taskManager.executeAndWaitBash(cmd);
    const mountMap = new Map<string, string>();

    if (result.code !== 0) {
      return mountMap;
    }

    for (const line of result.stdout.trim().split('\n')) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('#')) continue;

      const parts = trimmed.split(/\s+/);
      if (parts.length >= 2) {
        const device = parts[0];
        const mountPoint = parts[1];
        if (device && mountPoint && !device.startsWith('#')) {
          mountMap.set(mountPoint, device);
        }
      }
    }

    return mountMap;
  }

  private async runPrivilegedScript(script: string): Promise<void> {
    const cmd = `pkexec bash -lc ${this.shellQuote(script)}`;
    const result = await this.taskManager.executeAndWaitBash(cmd);
    if (result.code !== 0) {
      throw new Error((result.stderr || result.stdout || 'Privileged command failed').trim());
    }
  }

  private openLuksScript(rootPartition: string, luksPassword: string | undefined): string {
    if (!luksPassword) {
      throw new Error('LUKS password is required for encrypted partitions');
    }
    return [
      `printf '%s' ${this.shellQuote(luksPassword)} | cryptsetup luksOpen ${rootPartition} chrootfs --key-file -`,
      'if [ $? -ne 0 ]; then',
      "  echo 'Failed to unlock LUKS partition' >&2",
      '  exit 1',
      'fi',
      'if [ ! -e /dev/mapper/chrootfs ]; then',
      "  echo 'LUKS partition opened but mapper device not found' >&2",
      '  exit 1',
      'fi',
    ].join('\n');
  }

  private async findBootPartition(
    chrootPath: string,
    mountPoint: string,
    allPartitions: PartitionWithPartType[],
  ): Promise<string | null> {
    const fstabMounts = await this.readFstab(chrootPath);
    const fstabDevice = fstabMounts.get(mountPoint);

    if (fstabDevice) {
      if (fstabDevice.startsWith('UUID=')) {
        const uuid = fstabDevice.replace('UUID=', '').replace(/"/g, '');
        return `/dev/disk/by-uuid/${uuid}`;
      } else if (fstabDevice.startsWith('LABEL=')) {
        const label = fstabDevice.replace('LABEL=', '').replace(/"/g, '');
        return `/dev/disk/by-label/${label}`;
      } else if (fstabDevice.startsWith('/')) {
        return fstabDevice;
      }
    }

    for (const part of allPartitions) {
      if (part.mountpoint === mountPoint) {
        if (mountPoint === '/boot/efi') {
          const isEsp = await this.isEspPartition(part.name);
          if (isEsp) return `/dev/${part.name}`;
        } else if (mountPoint === '/boot') {
          if (part.fstype && ['ext2', 'ext3', 'ext4', 'btrfs', 'xfs', 'vfat'].includes(part.fstype)) {
            return `/dev/${part.name}`;
          }
        }
      }
    }

    return null;
  }

  private generateChrootMountScript(rootDev: string, path: string, isLuks = false): string {
    const mountCommands: string[] = [`mkdir -p "${path}"`];

    if (isLuks) {
      mountCommands.push(
        `FSTYPE=$(blkid -o value -s TYPE ${rootDev} 2>/dev/null || echo "")`,
        `if [ "$FSTYPE" = "btrfs" ]; then`,
        `  mount -o subvol=@ ${rootDev} "${path}"`,
        `else`,
        `  mount ${rootDev} "${path}"`,
        `fi`,
      );
    } else {
      mountCommands.push(`mount ${rootDev} "${path}"`);
    }

    mountCommands.push(
      `mount --bind /dev "${path}/dev"`,
      `mount --bind /sys "${path}/sys"`,
      `mount --bind /proc "${path}/proc"`,
      `mount -t efivarfs efivarfs "${path}/sys/firmware/efi/efivars" 2>/dev/null || true`,
    );
    return mountCommands.join('\n');
  }

  private generateCleanupScript(path: string, luksOpened = false): string {
    if (path === '/') return '';
    return [
      `mountpoint -q "${path}/boot/efi" && umount "${path}/boot/efi"`,
      `mountpoint -q "${path}/boot" && umount -R "${path}/boot"`,
      `umount "${path}/proc" "${path}/sys" "${path}/dev" 2>/dev/null || true`,
      `umount -R "${path}" 2>/dev/null || true`,
      `rmdir "${path}" 2>/dev/null || true`,
      luksOpened ? 'cryptsetup luksClose chrootfs 2>/dev/null || true' : '',
    ].join('\n');
  }

  async repairGrub(rootPartition: string, luksPassword?: string): Promise<void> {
    const isCurrentRoot = await this.isCurrentRootSelection(rootPartition);

    if (isCurrentRoot) {
      await this.runPrivilegedScript('set -e\nupdate-grub');
      return;
    }

    const runtimeIsLuks = await this.isLuks(rootPartition.replace('/dev/', ''));
    const isLuksPartition = luksPassword ? true : runtimeIsLuks;
    const effectiveRoot = isLuksPartition ? '/dev/mapper/chrootfs' : rootPartition;
    const path = '/tmp/grub-repair-mnt';
    const chrootScript = this.generateChrootMountScript(effectiveRoot, path, isLuksPartition);
    const script = [
      'set -e',
      `cleanup() {\n${this.generateCleanupScript(path, isLuksPartition)}\n}`,
      'trap cleanup EXIT',
      isLuksPartition ? this.openLuksScript(rootPartition, luksPassword) : '',
      chrootScript,
      `chroot "${path}" update-grub`,
    ].join('\n');

    await this.runPrivilegedScript(script);
  }

  async installGrub(rootPartition: string, location: string, isEsp: boolean, luksPassword?: string): Promise<void> {
    const isCurrentRoot = await this.isCurrentRootSelection(rootPartition);
    const runtimeIsLuks = await this.isLuks(rootPartition.replace('/dev/', ''));
    const isLuksPartition = !isCurrentRoot && (luksPassword ? true : runtimeIsLuks);
    const arch = await this.getArch();
    const path = isCurrentRoot ? '/' : '/tmp/grub-install-mnt';
    const effectiveRoot = isLuksPartition ? '/dev/mapper/chrootfs' : rootPartition;
    const scriptParts = ['set -e'];

    if (!isCurrentRoot) {
      scriptParts.push(`cleanup() {\n${this.generateCleanupScript(path, isLuksPartition)}\n}`);
      scriptParts.push('trap cleanup EXIT');
      if (isLuksPartition) {
        scriptParts.push(this.openLuksScript(rootPartition, luksPassword));
      }
      const chrootScript = this.generateChrootMountScript(effectiveRoot, path, isLuksPartition);
      scriptParts.push(chrootScript);
    }

    let installCmd: string;
    if (isEsp) {
      installCmd = [
        `test -d "${path}/boot/efi" || mkdir -p "${path}/boot/efi"`,
        !isCurrentRoot ? `mountpoint -q "${path}/boot/efi" || mount /dev/${location} "${path}/boot/efi"` : '',
        `chroot "${path}" grub-install --target=${arch}-efi --efi-directory=/boot/efi --bootloader-id=Garuda --recheck --force`,
        `chroot "${path}" update-grub`,
      ]
        .filter(Boolean)
        .join('\n');
    } else {
      installCmd = [
        `chroot "${path}" grub-install --target=i386-pc --recheck --force /dev/${location}`,
        `chroot "${path}" update-grub`,
      ].join('\n');
    }

    scriptParts.push(installCmd);
    await this.runPrivilegedScript(scriptParts.join('\n'));
  }

  async backupMbr(location: string, filename: string): Promise<void> {
    const script = `set -e\ndd if=/dev/${location} of="${filename}" bs=446 count=1`;
    await this.runPrivilegedScript(script);
  }

  async restoreMbr(location: string, filename: string): Promise<void> {
    const script = `set -e\ndd if="${filename}" of=/dev/${location} bs=446 count=1`;
    await this.runPrivilegedScript(script);
  }
}
