import { inject, Injectable } from '@angular/core';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { PartitionInfo } from '../boot-options/types';
import { Logger } from '../../logging/logging';

export interface PartitionWithPartType extends PartitionInfo {
  parttype: string;
}

export interface DiskInfo {
  name: string;
  size: string;
  label: string;
  model: string;
}

@Injectable({
  providedIn: 'root',
})
export class BootRepairService {
  private readonly taskManager = inject(TaskManagerService);
  private readonly logger = Logger.getInstance();

  private shellQuote(value: string): string {
    return `'${value.replaceAll("'", `'"'"'`)}'`;
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
    ].join('\n');
  }

  async getDisks(): Promise<DiskInfo[]> {
    const cmd = "lsblk -ln -o NAME,SIZE,LABEL,MODEL -d -e 2,11 -x NAME | grep -E '^x?[h,s,v].[a-z]|^mmcblk|^nvme'";
    const result = await this.taskManager.executeAndWaitBash(cmd);
    const lines = result.stdout
      .trim()
      .split('\n')
      .filter((l) => l.length > 0);
    return lines.map((line) => {
      const parts = line.split(/\s+/);
      return {
        name: parts[0] || '',
        size: parts[1] || '',
        label: parts[2] && parts[2] !== parts[3] ? parts[2] : '',
        model: parts.slice(3).join(' ') || (parts[2] ? parts.slice(2).join(' ') : ''),
      };
    });
  }

  async getPartitions(): Promise<PartitionInfo[]> {
    const cmd =
      "lsblk -ln -o NAME,SIZE,FSTYPE,MOUNTPOINT,LABEL,PARTTYPE -e 2,11 -x NAME | grep -E '^x?[h,s,v].[a-z][0-9]|^mmcblk[0-9]+p|^nvme[0-9]+n[0-9]+p'";
    const result = await this.taskManager.executeAndWaitBash(cmd);
    const lines = result.stdout
      .trim()
      .split('\n')
      .filter((l) => l.length > 0);
    return lines.map((line) => {
      const parts = line.split(/\s+/);
      return {
        name: parts[0] || '',
        size: parts[1] || '',
        fstype: parts[2] || '',
        mountpoint: parts[3] || '',
        label: parts.slice(4, -1).join(' ') || '',
      };
    });
  }

  async getPartitionsWithPartType(): Promise<PartitionWithPartType[]> {
    const cmd =
      "lsblk -ln -o NAME,SIZE,FSTYPE,MOUNTPOINT,LABEL,PARTTYPE -e 2,11 -x NAME | grep -E '^x?[h,s,v].[a-z][0-9]|^mmcblk[0-9]+p|^nvme[0-9]+n[0-9]+p'";
    const result = await this.taskManager.executeAndWaitBash(cmd);
    const lines = result.stdout
      .trim()
      .split('\n')
      .filter((l) => l.length > 0);

    return lines.map((line) => {
      const parts = line.split(/\s+/);
      return {
        name: parts[0] || '',
        size: parts[1] || '',
        fstype: parts[2] || '',
        mountpoint: parts[3] || '',
        label: parts.slice(4, -1).join(' ') || '',
        parttype: (parts[parts.length - 1] || '').toLowerCase(),
      };
    });
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

  private async isCurrentRootSelection(rootPartition: string): Promise<boolean> {
    const currentRoot = await this.getCurrentRootSource();

    if (!currentRoot || !rootPartition) {
      return false;
    }

    if (currentRoot === rootPartition || currentRoot === '/dev/mapper/rootfs') {
      return true;
    }

    if (currentRoot.startsWith('/dev/mapper/luks-')) {
      const expectedMapperPath = await this.getLuksMapperPath(rootPartition);
      if (expectedMapperPath && expectedMapperPath === currentRoot) {
        return true;
      }
    }

    return false;
  }

  async getCurrentRootPartition(): Promise<string> {
    const result = await this.taskManager.executeAndWaitBash("findmnt -no SOURCE / | sed 's/\\[.*\\]$//'");
    const source = result.stdout.trim();
    this.logger.debug(`[boot-repair] current root source: ${source}`);

    if (!source) {
      return '';
    }

    if (source.startsWith('/dev/mapper/')) {
      const escapedSource = this.shellQuote(source);
      const parents = await this.taskManager.executeAndWaitBash(
        `lsblk -ndo PKNAME "$(readlink -f ${escapedSource})" 2>/dev/null || true`,
      );
      const parent = parents.stdout
        .trim()
        .split('\n')
        .find((line) => line.length > 0);
      if (parent) {
        this.logger.debug(`[boot-repair] root mapper parent via lsblk: /dev/${parent}`);
        return `/dev/${parent}`;
      }

      const mapperName = source.replace('/dev/mapper/', '');
      const cryptStatus = await this.taskManager.executeAndWaitBash(
        `cryptsetup status ${this.shellQuote(mapperName)} 2>/dev/null | sed -nE 's/^[[:space:]]*device:[[:space:]]*([^[:space:]]+).*/\\1/p'`,
      );
      const mappedDevice = cryptStatus.stdout
        .trim()
        .split('\n')
        .find((line) => line.length > 0);

      this.logger.debug(`[boot-repair] root mapper parent via cryptsetup: ${mappedDevice || '(none)'}`);

      return mappedDevice || source;
    }

    return source;
  }

  /**
   * Generates a chroot script.
   */
  private generateChrootMountScript(rootDev: string, path: string): string {
    return [
      `mkdir -p "${path}"`,
      `mount -o subvol=@ -o subvol=@ ${rootDev} "${path}"`,
      `mount -o subvol=@ -o bind /dev "${path}/dev"`,
      `mount -o subvol=@ -o bind /sys "${path}/sys"`,
      `mount -o subvol=@ -o bind /proc "${path}/proc"`,
      `mount -t efivarfs efivarfs "${path}/sys/firmware/efi/efivars" 2>/dev/null || true`,
    ].join('\n');
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

    const isLuksPartition = await this.isLuks(rootPartition.replace('/dev/', ''));
    const effectiveRoot = isLuksPartition ? '/dev/mapper/chrootfs' : rootPartition;

    const path = '/tmp/grub-repair-mnt';
    const script = [
      'set -e',
      `cleanup() {\n${this.generateCleanupScript(path, isLuksPartition)}\n}`,
      'trap cleanup EXIT',
      isLuksPartition ? this.openLuksScript(rootPartition, luksPassword) : '',
      this.generateChrootMountScript(effectiveRoot, path),
      `chroot "${path}" update-grub`,
    ].join('\n');
    await this.runPrivilegedScript(script);
  }

  async installGrub(rootPartition: string, location: string, isEsp: boolean, luksPassword?: string): Promise<void> {
    const isCurrentRoot = await this.isCurrentRootSelection(rootPartition);
    const isLuksPartition = !isCurrentRoot && (await this.isLuks(rootPartition.replace('/dev/', '')));
    const arch = await this.getArch();
    const path = isCurrentRoot ? '/' : '/tmp/grub-install-mnt';
    const effectiveRoot = isLuksPartition ? '/dev/mapper/chrootfs' : rootPartition;

    let installCmd = '';
    if (isEsp) {
      installCmd = [
        `test -d "${path}/boot/efi" || mkdir -p "${path}/boot/efi"`,
        `mount /dev/${location} "${path}/boot/efi"`,
        `chroot "${path}" grub-install --target=${arch}-efi --efi-directory=/boot/efi --bootloader-id=Garuda --recheck --force`,
        `chroot "${path}" update-grub`,
      ].join('\n');
    } else {
      installCmd = [
        `chroot "${path}" grub-install --target=i386-pc --recheck --force /dev/${location}`,
        `chroot "${path}" update-grub`,
      ].join('\n');
    }

    const scriptParts = ['set -e'];
    if (!isCurrentRoot) {
      scriptParts.push(`cleanup() {\n${this.generateCleanupScript(path, isLuksPartition)}\n}`);
      scriptParts.push('trap cleanup EXIT');
      if (isLuksPartition) {
        scriptParts.push(this.openLuksScript(rootPartition, luksPassword));
      }
      scriptParts.push(this.generateChrootMountScript(effectiveRoot, path));
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
