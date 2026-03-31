import { ChangeDetectionStrategy, Component, computed, effect, inject, OnInit, signal, untracked } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { Card } from 'primeng/card';
import { Select } from 'primeng/select';
import { Button } from 'primeng/button';
import { RadioButton } from 'primeng/radiobutton';
import { Password } from 'primeng/password';
import { ConfirmationService } from 'primeng/api';
import { BootRepairService, DiskInfo } from './boot-repair.service';
import { PartitionInfo } from '../boot-options/types';
import { MessageToastService } from '@garudalinux/core';
import { dialogOpen, dialogSave } from '../../electron-services/electron-api-utils.js';
import { BackupRestoreTarget, BootInstallTarget, BootRepairAction } from './types';
import { ConfigService } from '../config/config.service';
import { Logger } from '../../logging/logging';
import { LoadingService } from '../loading-indicator/loading-indicator.service';
import { Tooltip } from 'primeng/tooltip';

@Component({
  selector: 'toolbox-boot-repair',
  standalone: true,
  imports: [CommonModule, FormsModule, TranslocoDirective, Card, Select, Button, RadioButton, Password, Tooltip],
  templateUrl: './boot-repair.component.html',
  styleUrl: './boot-repair.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BootRepairComponent implements OnInit {
  private readonly bootRepairService = inject(BootRepairService);
  private readonly toast = inject(MessageToastService);
  private readonly confirmationService = inject(ConfirmationService);
  private readonly configService = inject(ConfigService);
  private readonly loadingService = inject(LoadingService);
  private readonly logger = Logger.getInstance();
  protected readonly transloco = inject(TranslocoService);

  private luksRootMap = signal<Record<string, boolean>>({});
  private luksMapperPathMap = signal<Record<string, string>>({});

  loading = signal(true);
  disks = signal<DiskInfo[]>([]);
  partitions = signal<PartitionInfo[]>([]);
  linuxPartitions = signal<PartitionInfo[]>([]);
  espPartitions = signal<PartitionInfo[]>([]);
  isUefi = signal(false);

  action = signal<BootRepairAction>('reinstall');
  installTarget = signal<BootInstallTarget>('root');
  backupRestoreTarget = signal<BackupRestoreTarget>('mbr');

  selectedLocation = signal<string | null>(null);
  selectedRoot = signal<string | null>(null);
  currentRootPartition = signal<string>('');
  luksPassword = signal('');
  selectedRootIsLuks = computed(() => {
    const root = this.selectedRoot();
    return root ? this.luksRootMap()[root] : false;
  });
  isCurrentInstalledRoot = computed(() => {
    const selectedRoot = this.selectedRoot();
    if (this.configService.state().isLiveSystem !== false || !selectedRoot) {
      return false;
    }

    const selectedRootPath = '/dev/' + selectedRoot;
    const current = this.currentRootPartition();
    const selectedNorm = this.normalizeDevice(selectedRootPath);
    const currentNorm = this.normalizeDevice(current);

    if (selectedNorm === currentNorm) {
      return true;
    }

    if (current.startsWith('/dev/mapper/luks-')) {
      const expectedMapperPath = this.luksMapperPathMap()[selectedRoot] || '';
      return expectedMapperPath.length > 0 && this.normalizeDevice(expectedMapperPath) === currentNorm;
    }

    return false;
  });
  shouldShowLuksPrompt = computed(() => this.selectedRootIsLuks() && !this.isCurrentInstalledRoot());

  locations = signal<{ label: string; value: string }[]>([]);
  roots = signal<{ label: string; value: string }[]>([]);

  private formatRootOption(partition: PartitionInfo): { label: string; value: string } {
    const luksSuffix = this.luksRootMap()[partition.name]
      ? ' [' + this.transloco.translate('bootRepair.luksTag') + ']'
      : '';

    return {
      label:
        partition.name + ' (' + partition.size + ')' + (partition.label ? ' - ' + partition.label : '') + luksSuffix,
      value: partition.name,
    };
  }

  private normalizeDevice(device: string): string {
    const normalized = device
      .trim()
      .replace(/\[.*\]$/, '')
      .replace('/dev/', '')
      .trim();

    if (normalized.startsWith('mapper/luks-')) {
      return normalized.replace(/^mapper\/luks-/, 'luks-');
    }

    return normalized;
  }

  private async refreshCurrentRootPartition(reason: string): Promise<void> {
    if (this.configService.state().isLiveSystem !== false) {
      this.currentRootPartition.set('');
      this.logger.debug(`[boot-repair] skip current-root refresh (${reason}): live system`);
      return;
    }

    const currentRootPartition = await this.bootRepairService.getCurrentRootPartition();
    this.currentRootPartition.set(currentRootPartition);
    this.logger.debug(`[boot-repair] refreshed current root (${reason}): ${currentRootPartition}`);
  }

  constructor() {
    effect(() => {
      const isLiveSystem = this.configService.state().isLiveSystem;
      if (isLiveSystem === false && this.currentRootPartition().length === 0) {
        void this.refreshCurrentRootPartition('config-state-change');
      }
    });

    effect(() => {
      const act = this.action();
      const target = this.installTarget();
      const backupTarget = this.backupRestoreTarget();
      const parts = this.partitions();
      const linuxParts = this.linuxPartitions();
      const dsks = this.disks();

      untracked(() => {
        const rootOptions = linuxParts.map((p) => this.formatRootOption(p));
        const partOptions = parts.map((p) => ({
          label: p.name + ' (' + p.size + ')' + (p.label ? ' - ' + p.label : ''),
          value: p.name,
        }));
        this.roots.set(rootOptions);

        if (act === 'repair' || act === 'backup' || act === 'restore') {
          if (act === 'backup' || act === 'restore') {
            if (backupTarget === 'mbr') {
              this.locations.set(
                dsks.map((d) => ({
                  label: d.name + ' (' + d.size + ')' + (d.model ? ' - ' + d.model : ''),
                  value: d.name,
                })),
              );
            } else {
              this.locations.set(partOptions);
            }
          } else {
            this.locations.set(rootOptions);
          }
          return;
        }

        if (target === 'mbr') {
          this.locations.set(
            dsks.map((d) => ({
              label: d.name + ' (' + d.size + ')' + (d.model ? ' - ' + d.model : ''),
              value: d.name,
            })),
          );
        } else if (target === 'esp') {
          this.locations.set(
            this.espPartitions().map((p) => ({
              label: p.name + ' (' + p.size + ')' + (p.label ? ' - ' + p.label : ''),
              value: p.name,
            })),
          );
        } else {
          this.locations.set(rootOptions);
        }

        const locationValues = new Set(this.locations().map((option) => option.value));
        const selectedLocation = this.selectedLocation();
        if (!selectedLocation || !locationValues.has(selectedLocation)) {
          this.selectedLocation.set(this.locations()[0]?.value ?? null);
        }

        if (this.selectedRoot() && this.isCurrentInstalledRoot()) {
          this.luksPassword.set('');
        }
      });
    });
  }

  async ngOnInit(): Promise<void> {
    await this.loadDevices();
  }

  async loadDevices(): Promise<void> {
    this.loading.set(true);
    try {
      const [disks, partitionsWithType, isUefi] = await Promise.all([
        this.bootRepairService.getDisks(),
        this.bootRepairService.getPartitionsWithPartType(),
        this.bootRepairService.isUefiSystem(),
      ]);
      this.disks.set(disks);
      this.isUefi.set(isUefi);

      const linuxParts: PartitionInfo[] = [];
      const espParts: PartitionInfo[] = [];
      const luksMap: Record<string, boolean> = {};
      const mapperMap: Record<string, string> = {};
      for (const p of partitionsWithType) {
        const partType = p.parttype.toLowerCase();
        if (
          /^(0x83|0fc63daf-8483-4772-8e79-3d69d8477de4|44479540-f297-41b2-9af7-d131d5f0458a|4f68bce3-e8cd-4db1-96e7-fbcaf984b709)$/i.test(
            partType,
          )
        ) {
          linuxParts.push(p);
          const isLuks =
            (p.fstype || '').toLowerCase().includes('crypto_luks') || (await this.bootRepairService.isLuks(p.name));
          luksMap[p.name] = isLuks;
          if (isLuks) {
            mapperMap[p.name] = await this.bootRepairService.getLuksMapperPath('/dev/' + p.name);
          }
        } else if (/^c12a7328-f81f-11d2-ba4b-00a0c93ec93b$/i.test(partType)) {
          espParts.push(p);
        }
      }
      this.luksRootMap.set(luksMap);
      this.luksMapperPathMap.set(mapperMap);
      this.partitions.set(partitionsWithType);
      this.linuxPartitions.set(linuxParts);
      this.espPartitions.set(espParts);

      this.roots.set(linuxParts.map((p) => this.formatRootOption(p)));

      if (this.action() === 'reinstall') {
        this.installTarget.set(isUefi && espParts.length > 0 ? 'esp' : 'root');
      }

      const rootGuess = linuxParts.find((p) => p.label.includes('rootGaruda')) || linuxParts[0];
      if (rootGuess) {
        this.selectedRoot.set(rootGuess.name);
      }

      this.selectedLocation.set(null);

      await this.refreshCurrentRootPartition('load-devices');
    } catch (e) {
      this.toast.error(this.transloco.translate('bootRepair.errorTitle'), String(e));
    } finally {
      this.loading.set(false);
    }
  }

  async apply(): Promise<void> {
    const act = this.action();
    const loc = this.selectedLocation();
    const root = this.selectedRoot();
    const selectedRootPath = root ? '/dev/' + root : '';

    if (!loc) {
      this.toast.error(
        this.transloco.translate('bootRepair.errorTitle'),
        this.transloco.translate('bootRepair.selectLocation'),
      );
      return;
    }

    const isCurrentInstalledRoot = this.isCurrentInstalledRoot();

    if (
      (act === 'repair' || act === 'reinstall') &&
      this.selectedRootIsLuks() &&
      !isCurrentInstalledRoot &&
      this.luksPassword().length === 0
    ) {
      this.toast.error(
        this.transloco.translate('bootRepair.errorTitle'),
        this.transloco.translate('bootRepair.luksPasswordRequired'),
      );
      return;
    }

    const actionSuccessMap: Record<BootRepairAction, string> = {
      reinstall: 'bootRepair.successReinstall',
      repair: 'bootRepair.successRepair',
      backup: 'bootRepair.successBackup',
      restore: 'bootRepair.successRestore',
    };

    this.loadingService.loadingOn();
    try {
      if (act === 'repair') {
        if (!root) {
          this.toast.error(
            this.transloco.translate('bootRepair.errorTitle'),
            this.transloco.translate('bootRepair.selectRoot'),
          );
          return;
        }
        await this.bootRepairService.repairGrub(
          selectedRootPath,
          this.selectedRootIsLuks() && !isCurrentInstalledRoot ? this.luksPassword() : undefined,
        );
      } else if (act === 'reinstall') {
        if (!root) {
          this.toast.error(
            this.transloco.translate('bootRepair.errorTitle'),
            this.transloco.translate('bootRepair.selectRoot'),
          );
          return;
        }
        await this.bootRepairService.installGrub(
          selectedRootPath,
          loc,
          this.installTarget() === 'esp',
          this.selectedRootIsLuks() && !isCurrentInstalledRoot ? this.luksPassword() : undefined,
        );
      } else if (act === 'backup') {
        const result = await dialogSave({
          properties: ['showOverwriteConfirmation'],
          filters: [{ name: this.transloco.translate('bootRepair.backupFile'), extensions: ['img', 'bin'] }],
        });
        if (result && result.length > 0) {
          await this.bootRepairService.backupMbr(loc, result);
        } else {
          return;
        }
      } else if (act === 'restore') {
        const result = await dialogOpen({
          properties: ['openFile'],
          filters: [{ name: this.transloco.translate('bootRepair.backupFile'), extensions: ['img', 'bin'] }],
        });
        if (result && result.length > 0) {
          await this.bootRepairService.restoreMbr(loc, result[0]);
        } else {
          return;
        }
      }

      await this.showMessageBox(
        this.transloco.translate('bootRepair.successTitle'),
        this.transloco.translate(actionSuccessMap[act]),
        'success',
      );
    } catch (e) {
      const details = e instanceof Error ? e.message : String(e);
      await this.showMessageBox(this.transloco.translate('bootRepair.errorTitle'), details, 'error');
    } finally {
      this.loadingService.loadingOff();
    }
  }

  private showMessageBox(title: string, message: string, type: 'success' | 'error'): Promise<void> {
    return new Promise((resolve) => {
      this.confirmationService.confirm({
        header: title,
        message,
        icon: type === 'success' ? 'pi pi-check-circle' : 'pi pi-times-circle',
        closable: false,
        closeOnEscape: false,
        rejectVisible: false,
        acceptButtonProps: {
          label: this.transloco.translate('confirmation.accept'),
          severity: type === 'success' ? 'success' : 'danger',
        },
        accept: () => resolve(),
      });
    });
  }

  onRootChange(): void {
    if (!this.selectedRootIsLuks() || this.isCurrentInstalledRoot()) {
      this.luksPassword.set('');
    }
  }
}
