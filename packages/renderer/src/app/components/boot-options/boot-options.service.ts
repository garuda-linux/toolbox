import { inject, Injectable } from '@angular/core';
import { ElectronFsService } from '../../electron-services';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { OsInteractService } from '../task-manager/os-interact.service';
import { BootEntry, PartitionInfo } from './types';

@Injectable({
  providedIn: 'root',
})
export class BootOptionsService {
  private readonly fsService = inject(ElectronFsService);
  private readonly taskManager = inject(TaskManagerService);
  private readonly osInteract = inject(OsInteractService);

  setChroot(path: string) {
    this.osInteract.setChroot(path);
  }

  getChroot() {
    return this.osInteract.getChroot();
  }

  async getGrubEntries(): Promise<BootEntry[]> {
    const content = await this.osInteract.readPrivilegedFile('/boot/grub/grub.cfg');
    return this.parseGrubCfg(content);
  }

  async getLinuxPartitions(): Promise<PartitionInfo[]> {
    const cmd = `lsblk -ln -o NAME,SIZE,FSTYPE,MOUNTPOINT,LABEL -e 2,11 -x NAME | grep -E '^x?[h,s,v].[a-z][0-9]|^mmcblk[0-9]+p|^nvme[0-9]+n[0-9]+p'`;
    const result = await this.taskManager.executeAndWaitBash(cmd);
    const lines = result.stdout.trim().split('\n');
    const partitions: PartitionInfo[] = [];

    for (const line of lines) {
      const parts = line.split(/\s+/);
      if (parts.length >= 2) {
        const name = parts[0];
        // Validate partition type (Legacy app logic)
        const typeCmd = `lsblk -ln -o PARTTYPE /dev/${name} | grep -qEi '0x83|0fc63daf-8483-4772-8e79-3d69d8477de4|44479540-F297-41B2-9AF7-D131D5F0458A|4F68BCE3-E8CD-4DB1-96E7-FBCAF984B709'`;
        const typeResult = await this.taskManager.executeAndWaitBash(typeCmd);

        if (typeResult.code === 0) {
          partitions.push({
            name,
            size: parts[1] || '',
            fstype: parts[2] || '',
            mountpoint: parts[3] || '',
            label: parts[4] || '',
          });
        }
      }
    }
    return partitions;
  }

  private parseGrubCfg(content: string): BootEntry[] {
    const lines = content.split('\n');
    const entries: BootEntry[] = [];
    const stack: { id: string; label: string }[] = [];

    for (const line of lines) {
      const trimmed = line.trim();

      if (trimmed.startsWith('menuentry ') || trimmed.startsWith('submenu ')) {
        const isSubmenu = trimmed.startsWith('submenu ');
        const labelMatch = trimmed.match(/['"](.*?)['"]/);
        const label = labelMatch ? labelMatch[1] : 'Unknown';

        const idMatch = trimmed.match(/--id\s+(['"]?)(.*?)\1(?:\s|$)/);
        const rawId = idMatch ? idMatch[2] : label;

        const fullId = stack.length > 0 ? `${stack[stack.length - 1].id}>${rawId}` : rawId;

        entries.push({
          label: label,
          id: fullId,
          isSubmenu,
          parentLabel: stack.length > 0 ? stack[stack.length - 1].label : undefined,
        });

        if (isSubmenu) {
          stack.push({ id: rawId, label });
        }
      } else if (trimmed === '}') {
        stack.pop();
      }
    }
    return entries;
  }
}
