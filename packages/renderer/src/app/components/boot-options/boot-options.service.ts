import { inject, Injectable, signal } from '@angular/core';
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

  isVirtualMachine = signal<boolean>(false);
  private vmChecked = false;

  setChroot(path: string) {
    this.osInteract.setChroot(path);
  }

  getChroot() {
    return this.osInteract.getChroot();
  }

  async checkIfVirtualMachine(): Promise<boolean> {
    if (this.vmChecked) {
      return this.isVirtualMachine();
    }

    const virtResult = await this.taskManager.executeAndWaitBash('systemd-detect-virt');
    if (virtResult.code === 0 && virtResult.stdout.trim() !== 'none' && virtResult.stdout.trim() !== 'container') {
      this.isVirtualMachine.set(true);
      this.vmChecked = true;
      return true;
    }

    this.isVirtualMachine.set(false);
    this.vmChecked = true;
    return false;
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

  /**
   * Find line numbers where each entry starts and ends in grub.cfg
   */
  private findEntryLocations(grubCfg: string): Map<string, { start: number; end: number }> {
    const lines = grubCfg.split('\n');
    const locations = new Map<string, { start: number; end: number }>();

    const submenuStack: { id: string; label: string; depth: number }[] = [];
    const entryStack: { start: number; fullId: string; depth: number }[] = [];
    let currentDepth = 0;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const trimmed = line.trim();

      if (trimmed.startsWith('menuentry ') || trimmed.startsWith('submenu ')) {
        const labelMatch = trimmed.match(/(["'])(.*?)\1/);
        const idMatch = trimmed.match(/--id\s+(['"]?)(.*?)\1(?:\s|$)/);
        const varIdMatch = trimmed.match(/\$menuentry_id_option\s+(['"])(.*?)\1/);

        let rawId = labelMatch ? labelMatch[2] : 'Unknown';
        if (idMatch && idMatch[2]) {
          rawId = idMatch[2];
        } else if (varIdMatch && varIdMatch[2]) {
          rawId = varIdMatch[2];
        }

        const parentPrefix = submenuStack.length > 0 ? `${submenuStack[submenuStack.length - 1].id}>` : '';
        const fullId = parentPrefix + rawId;
        const isSubmenu = trimmed.startsWith('submenu ');

        entryStack.push({ start: i, fullId, depth: currentDepth });
        if (isSubmenu) {
          submenuStack.push({ id: rawId, label: labelMatch ? labelMatch[2] : 'Unknown', depth: currentDepth });
        }
      }

      if (trimmed.endsWith('{')) {
        currentDepth++;
      } else if (trimmed === '}') {
        currentDepth--;
        if (submenuStack.length > 0 && currentDepth < submenuStack[submenuStack.length - 1].depth) {
          submenuStack.pop();
        }
      }

      if (entryStack.length > 0) {
        const currentEntry = entryStack[entryStack.length - 1];
        if (currentDepth === currentEntry.depth && i > currentEntry.start) {
          locations.set(currentEntry.fullId, { start: currentEntry.start, end: i });
          entryStack.pop();

          if (submenuStack.length > 0 && currentDepth < submenuStack[submenuStack.length - 1].depth) {
            submenuStack.pop();
          } else if (submenuStack.length > 0 && currentDepth === submenuStack[submenuStack.length - 1].depth) {
            submenuStack.pop();
          }
        }
      }
    }

    return locations;
  }

  /**
   * Extract a specific menuentry block from grub.cfg content
   */
  extractEntryFromGrubCfg(grubCfg: string, entry: BootEntry): string {
    const lines = grubCfg.split('\n');
    const locations = this.findEntryLocations(grubCfg);
    const location = locations.get(entry.id);

    if (!location) {
      throw new Error(`Entry "${entry.label}" (id: ${entry.id}) not found in grub.cfg`);
    }

    return lines.slice(location.start, location.end + 1).join('\n');
  }

  /**
   * Replace a specific menuentry block in grub.cfg content with new content
   */
  replaceEntryInGrubCfg(grubCfg: string, entry: BootEntry, newEntryContent: string): string {
    const lines = grubCfg.split('\n');
    const locations = this.findEntryLocations(grubCfg);
    const location = locations.get(entry.id);

    if (!location) {
      throw new Error(`Entry "${entry.label}" (id: ${entry.id}) not found in grub.cfg`);
    }

    const result = [
      ...lines.slice(0, location.start),
      ...newEntryContent.split('\n'),
      ...lines.slice(location.end + 1),
    ];

    return result.join('\n');
  }

  private parseGrubCfg(content: string): BootEntry[] {
    const lines = content.split('\n');
    const entries: BootEntry[] = [];
    const stack: { id: string; label: string; depth: number }[] = [];
    let currentDepth = 0;

    for (const line of lines) {
      const trimmed = line.trim();

      if (trimmed.endsWith('{')) {
        currentDepth++;
      } else if (trimmed === '}') {
        currentDepth--;
      }

      if (trimmed.startsWith('menuentry ') || trimmed.startsWith('submenu ')) {
        const isSubmenu = trimmed.startsWith('submenu ');
        const labelMatch = trimmed.match(/(["'])(.*?)\1/);
        const label = labelMatch ? labelMatch[2] : 'Unknown';

        let rawId = label;
        const idMatch = trimmed.match(/--id\s+(['"]?)(.*?)\1(?:\s|$)/);
        const varIdMatch = trimmed.match(/\$menuentry_id_option\s+(['"])(.*?)\1/);

        if (idMatch && idMatch[2]) {
          rawId = idMatch[2];
        } else if (varIdMatch && varIdMatch[2]) {
          rawId = varIdMatch[2];
        }

        const parentPrefix = stack.length > 0 ? `${stack[stack.length - 1].id}>` : '';
        const fullId = parentPrefix + rawId;

        entries.push({
          label: label,
          id: fullId,
          isSubmenu,
          parentLabel: stack.length > 0 ? stack[stack.length - 1].label : undefined,
        });

        if (isSubmenu) {
          stack.push({ id: rawId, label, depth: currentDepth });
        }
      } else if (trimmed === '}') {
        if (stack.length > 0 && currentDepth < stack[stack.length - 1].depth) {
          stack.pop();
        }
      }
    }
    return entries;
  }
}
