import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  HostListener,
  inject,
  OnInit,
  signal,
  untracked,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { Card } from 'primeng/card';
import { Select } from 'primeng/select';
import { InputNumber } from 'primeng/inputnumber';
import { InputText } from 'primeng/inputtext';
import { Button } from 'primeng/button';
import { Message } from 'primeng/message';
import { Checkbox } from 'primeng/checkbox';
import { TableModule } from 'primeng/table';
import { TooltipModule } from 'primeng/tooltip';
import { DialogModule } from 'primeng/dialog';
import { TextareaModule } from 'primeng/textarea';
import { ProgressBar } from 'primeng/progressbar';
import { MessageToastService } from '@garudalinux/core';
import { LoadingService } from '../loading-indicator/loading-indicator.service';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { OsInteractService } from '../task-manager/os-interact.service';
import { BootEntry, PartitionInfo } from './types';
import { BootOptionsService } from './boot-options.service';
import { ConfigService } from '../config/config.service';
import { dialogOpen } from '../../electron-services/electron-api-utils.js';
import { ActivatedRoute } from '@angular/router';

@Component({
  selector: 'toolbox-boot-options',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    TranslocoDirective,
    Card,
    Select,
    InputNumber,
    InputText,
    Button,
    Message,
    Checkbox,
    TableModule,
    TooltipModule,
    DialogModule,
    TextareaModule,
    ProgressBar,
  ],
  templateUrl: './boot-options.component.html',
  styleUrl: './boot-options.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BootOptionsComponent implements OnInit {
  protected readonly bootService = inject(BootOptionsService);
  protected readonly osInteract = inject(OsInteractService);
  private readonly taskManager = inject(TaskManagerService);
  private readonly configService = inject(ConfigService);
  private readonly loadingService = inject(LoadingService);
  private readonly route = inject(ActivatedRoute);
  private readonly messageToastService = inject(MessageToastService);
  private readonly translocoService = inject(TranslocoService);

  error = signal<string | null>(null);
  warning = signal<string | null>(null);
  syncEnabled = false;
  previousFlatMenus = signal(false);
  previousBootsplash = signal(false);
  previousPlymouthTheme = signal<string>('');

  highlightDefaultEntry = signal(false);

  isLive = signal(false);
  partitions = signal<PartitionInfo[]>([]);
  selectedPartition = signal<string | null>(null);

  grubEntries = signal<BootEntry[]>([]);
  selectableEntries = signal<BootEntry[]>([]);
  plymouthThemes = signal<string[]>([]);
  isPlymouthInstalled = signal<boolean>(false);

  selectedDefaultEntry = signal<string>('');
  timeout = signal<number>(5);
  selectedPlymouthTheme = signal<string>('');
  kernelParameters = signal<string>('');
  enableBootsplash = signal<boolean>(false);
  messageLevel = signal<'limited' | 'detailed' | 'veryDetailed'>('detailed');

  mitigationsOff = signal(false);
  sysrqEnabled = signal(false);
  flatMenus = signal(false);
  saveDefault = signal(false);
  osProberEnabled = signal(false);

  grubBackground = signal('');
  grubTheme = signal('');

  editorDialogVisible = signal(false);
  editorFileType = signal<'grub'>('grub');
  editorEntry = signal<BootEntry | null>(null);
  editorContent = signal('');
  editorOriginalContent = signal('');
  editorFullGrubCfg = signal('');
  editorSaving = signal(false);
  editorError = signal<string | null>(null);
  isEditorModified = computed(() => this.editorContent() !== this.editorOriginalContent());

  messageLevelOptions = [
    { label: 'bootOptions.messageLevelLimited', value: 'limited' },
    { label: 'bootOptions.messageLevelDetailed', value: 'detailed' },
    { label: 'bootOptions.messageLevelVeryDetailed', value: 'veryDetailed' },
  ];

  @HostListener('document:click')
  onDocumentClick() {
    if (this.highlightDefaultEntry()) {
      this.highlightDefaultEntry.set(false);
    }
  }

  constructor() {
    effect(() => {
      const bootsplash = this.enableBootsplash();
      const defEntry = this.selectedDefaultEntry();
      const time = this.timeout();
      const params = this.kernelParameters();
      const msgLvl = this.messageLevel();
      const mitOff = this.mitigationsOff();
      const sysrq = this.sysrqEnabled();
      const flat = this.flatMenus();
      const saveDef = this.saveDefault();
      const osProber = this.osProberEnabled();
      const bg = this.grubBackground();
      const theme = this.grubTheme();
      const plyTheme = this.selectedPlymouthTheme();

      untracked(() => {
        if (!this.syncEnabled || this.loadingService.loading()) return;

        if (bootsplash && !this.isPlymouthInstalled() && !this.osInteract.check('plymouth', 'pkg')) {
          this.osInteract.setPackage('plymouth', true);
        } else if (!bootsplash && this.osInteract.check('plymouth', 'pkg') && !this.isPlymouthInstalled()) {
          this.osInteract.togglePackage('plymouth', true);
        }

        let kernelParams = params;
        const syncParam = (p: string, enabled: boolean) => {
          const regex = new RegExp(`\\s*${p.replace(/\./g, '\\.')}(?:=off|=[0-9]+)?\\b`, 'g');
          if (enabled && !kernelParams.includes(p)) kernelParams += ` ${p}`;
          else if (!enabled) kernelParams = kernelParams.replace(regex, '');
        };

        syncParam('splash', bootsplash);
        syncParam('mitigations=off', mitOff);
        syncParam('sysrq_always_enabled=1', sysrq);

        kernelParams = kernelParams.replace(/\s*quiet\b/g, '').replace(/\s*hush\b/g, '');
        if (msgLvl === 'limited') kernelParams += ' quiet hush';
        else if (msgLvl === 'detailed') kernelParams += ' quiet';

        const selectedEntry = this.grubEntries().find((e) => e.id === defEntry);
        const grubDefault = selectedEntry ? selectedEntry.id : defEntry;

        this.osInteract.setGrubSetting('GRUB_DEFAULT', saveDef ? 'saved' : grubDefault);
        this.osInteract.setGrubSetting('GRUB_TIMEOUT', time.toString());
        this.osInteract.setGrubSetting('GRUB_CMDLINE_LINUX_DEFAULT', kernelParams.trim());
        this.osInteract.setGrubSetting('GRUB_DISABLE_SUBMENU', flat ? 'y' : 'n');
        this.osInteract.setGrubSetting('GRUB_SAVEDEFAULT', saveDef ? 'true' : 'false');
        this.osInteract.setGrubSetting('GRUB_DISABLE_OS_PROBER', osProber ? 'false' : 'true');
        this.osInteract.setGrubSetting('GRUB_BACKGROUND', bg);
        this.osInteract.setGrubSetting('GRUB_THEME', theme);

        this.osInteract.setWantedPlymouthTheme(bootsplash ? plyTheme || null : null);

        if (saveDef) {
          const entryToSave = grubDefault || '0';
          this.osInteract.setWantedSaveDefaultEntry(entryToSave);
        } else {
          this.osInteract.setWantedSaveDefaultEntry(null);
        }

        const bootsplashEnabled = !this.previousBootsplash() && bootsplash;
        const themeChanged = this.previousPlymouthTheme() !== plyTheme && bootsplash;
        const shouldRebuildDracut =
          (bootsplashEnabled || themeChanged) && this.osInteract.check('plymouth', 'pkg', true);
        this.osInteract.setWantedDracutRebuild(shouldRebuildDracut);

        this.previousBootsplash.set(bootsplash);
        this.previousPlymouthTheme.set(plyTheme || '');

        if (bootsplash && this.bootService.isVirtualMachine()) {
          this.warning.set(this.translocoService.translate('bootOptions.plymouthVmWarning'));
        } else if (!bootsplash) {
          this.warning.set(null);
        }

        if (flat !== this.previousFlatMenus()) {
          this.previousFlatMenus.set(flat);
        }
      });
    });

    effect(() => {
      if (!this.taskManager.running() && this.syncEnabled) {
        void untracked(async () => {
          await this.loadSettings();
        });
      }
    });
  }

  async ngOnInit() {
    this.isLive.set(this.configService.state().isLiveSystem === true);
    await this.bootService.checkIfVirtualMachine();
    if (this.isLive()) await this.loadPartitions();
    else await this.loadSettings();
  }

  private triggerHighlight() {
    setTimeout(() => {
      this.highlightDefaultEntry.set(true);
    }, 300);
  }

  async loadPartitions() {
    this.loadingService.loadingOn();
    try {
      const parts = await this.bootService.getLinuxPartitions();
      this.partitions.set(parts);
      const guess = parts.find((p) => p.label.includes('rootGaruda'));
      if (guess) this.selectedPartition.set(guess.name);
    } catch (err) {
      this.error.set(String(err));
    } finally {
      this.loadingService.loadingOff();
    }
  }

  async onPartitionSelect() {
    if (!this.selectedPartition()) return;
    const path = `/tmp/@${this.selectedPartition()}`;
    await this.taskManager.executeAndWaitBash(
      `pkexec mkdir -p ${path} && pkexec mount /dev/${this.selectedPartition()} ${path}`,
    );
    this.bootService.setChroot(path);
    await this.loadSettings();
  }

  async loadSettings() {
    this.loadingService.loadingOn();
    this.syncEnabled = false;
    this.error.set(null);
    try {
      const [entries, grubSettings, plymouth] = await Promise.all([
        this.bootService.getGrubEntries(),
        this.osInteract.getGrubSettings(),
        this.osInteract.getPlymouthInfo(),
      ]);

      this.grubEntries.set(entries);
      const selectable = entries.filter((e: BootEntry) => !e.isSubmenu);
      this.selectableEntries.set(selectable);
      this.plymouthThemes.set(plymouth.themes);
      this.isPlymouthInstalled.set(plymouth.isInstalled);

      const rawDefault = grubSettings.get('GRUB_DEFAULT') || '0';

      const matchedEntry = entries.find(
        (e) => e.id === rawDefault || e.label === rawDefault || e.id.endsWith(`>${rawDefault}`),
      );

      if (matchedEntry) {
        this.selectedDefaultEntry.set(matchedEntry.id);
      } else {
        const index = parseInt(rawDefault);
        if (!isNaN(index) && selectable[index]) {
          this.selectedDefaultEntry.set(selectable[index].id);
        } else {
          this.selectedDefaultEntry.set(rawDefault);
        }
      }

      this.saveDefault.set(rawDefault === 'saved' || grubSettings.get('GRUB_SAVEDEFAULT') === 'true');
      this.timeout.set(parseInt(grubSettings.get('GRUB_TIMEOUT') || '5'));
      this.osProberEnabled.set(grubSettings.get('GRUB_DISABLE_OS_PROBER') !== 'true');

      const kernelParams = grubSettings.get('GRUB_CMDLINE_LINUX_DEFAULT') || '';
      this.kernelParameters.set(kernelParams);
      this.enableBootsplash.set(kernelParams.includes('splash'));
      this.mitigationsOff.set(kernelParams.includes('mitigations=off'));
      this.sysrqEnabled.set(kernelParams.includes('sysrq_always_enabled=1'));

      if (kernelParams.includes('hush')) this.messageLevel.set('limited');
      else if (kernelParams.includes('quiet')) this.messageLevel.set('detailed');
      else this.messageLevel.set('veryDetailed');

      this.flatMenus.set(grubSettings.get('GRUB_DISABLE_SUBMENU') === 'y');
      this.grubBackground.set(
        grubSettings.get('GRUB_BACKGROUND') || grubSettings.get('export GRUB_MENU_PICTURE') || '',
      );
      this.grubTheme.set(grubSettings.get('GRUB_THEME') || '');
      this.selectedPlymouthTheme.set(plymouth.currentTheme || '');

      this.loadingService.loadingOff();

      if (this.route.snapshot.queryParams['highlight'] === 'default-entry') {
        this.triggerHighlight();
      }

      setTimeout(() => (this.syncEnabled = true), 0);
    } catch (err) {
      this.error.set(String(err));
      this.loadingService.loadingOff();
    }
  }

  async selectFile(type: 'background' | 'theme') {
    const filters =
      type === 'background'
        ? [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'tga'] }]
        : [{ name: 'Theme', extensions: ['txt'] }];

    const result = await dialogOpen({ properties: ['openFile'], filters });
    if (result && result.length > 0) {
      if (type === 'background') this.grubBackground.set(result[0]);
      else this.grubTheme.set(result[0]);
    }
  }

  /**
   * Open the editor dialog for a specific boot entry.
   */
  async openEntryEditor(entry: BootEntry) {
    const chroot = this.bootService.getChroot();
    const grubCfgPath = chroot ? `${chroot}/boot/grub/grub.cfg` : '/boot/grub/grub.cfg';

    this.editorError.set(null);
    this.editorSaving.set(false);

    try {
      const fullCfg = await this.osInteract.readPrivilegedFile(grubCfgPath);
      this.editorFullGrubCfg.set(fullCfg);

      const entryContent = this.bootService.extractEntryFromGrubCfg(fullCfg, entry);

      this.editorContent.set(entryContent);
      this.editorOriginalContent.set(entryContent);
      this.editorEntry.set(entry);
      this.editorFileType.set('grub');
      this.editorDialogVisible.set(true);
    } catch (err) {
      this.editorError.set(String(err));
      this.editorDialogVisible.set(true);
    }
  }

  /**
   * Escape special regex characters in a string
   */
  private regexEscape(str: string): string {
    return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  /**
   * Close the editor dialog without saving.
   */
  closeEditor() {
    this.editorDialogVisible.set(false);
    this.editorContent.set('');
    this.editorOriginalContent.set('');
    this.editorFullGrubCfg.set('');
    this.editorEntry.set(null);
    this.editorError.set(null);
  }

  /**
   * Save the editor content to the file.
   */
  async saveEditor() {
    this.editorSaving.set(true);
    this.editorError.set(null);

    try {
      if (this.editorEntry()) {
        await this.saveEntryToGrubCfg();
      } else {
        await this.saveGrubDefault();
      }

      this.editorOriginalContent.set(this.editorContent());
      this.closeEditor();

      await this.loadSettings();

      this.messageToastService.success(
        this.translocoService.translate('bootOptions.saveSuccess'),
        this.editorEntry()
          ? this.translocoService.translate('bootOptions.entrySaved')
          : this.translocoService.translate('bootOptions.configSaved'),
      );
    } catch (err) {
      this.editorError.set(String(err));
    } finally {
      this.editorSaving.set(false);
    }
  }

  /**
   * Preview the currently selected Plymouth thme.
   */
  async previewPlymouthTheme(): Promise<void> {
    const theme = this.selectedPlymouthTheme();
    if (!theme || theme === 'details') return;

    const chroot = this.bootService.getChroot();
    const plymouthSetCmd = chroot ? `chroot ${chroot} plymouth-set-default-theme` : `plymouth-set-default-theme`;

    try {
      const currentThemeResult = await this.taskManager.executeAndWaitBash(plymouthSetCmd);
      const currentTheme = currentThemeResult.stdout.trim();

      await this.taskManager.executeAndWaitBash(`${plymouthSetCmd} -R ${theme}`);

      const previewCmd = chroot
        ? `pkexec chroot ${chroot} bash -c 'plymouthd; plymouth --show-splash; for ((i=0; i<4; i++)); do plymouth --update=test$i; sleep 1; done; plymouth quit'`
        : `pkexec bash -c 'plymouthd; plymouth --show-splash; for ((i=0; i<4; i++)); do plymouth --update=test$i; sleep 1; done; plymouth quit'`;

      const themeToRestore = currentTheme || 'details';
      const fullCmd = previewCmd + ' && ' + plymouthSetCmd + ' -R ' + themeToRestore;
      await this.taskManager.executeAndWaitBash(fullCmd);

      this.messageToastService.info(
        this.translocoService.translate('bootOptions.previewComplete'),
        this.translocoService.translate('bootOptions.previewCompleteMessage'),
      );
    } catch (err) {
      this.messageToastService.error(this.translocoService.translate('bootOptions.previewError'), String(err));
    }
  }

  /**
   * Save an individual entry to grub.cfg
   */
  private async saveEntryToGrubCfg() {
    const entry = this.editorEntry() as BootEntry;
    const newEntryContent = this.editorContent();
    const fullCfg = this.editorFullGrubCfg();
    const chroot = this.bootService.getChroot();

    const newContent = this.bootService.replaceEntryInGrubCfg(fullCfg, entry, newEntryContent);
    const grubCfgPath = chroot ? `${chroot}/boot/grub/grub.cfg` : '/boot/grub/grub.cfg';
    await this.writeFileWithPkexec(grubCfgPath, newContent);
  }

  /**
   * Save /etc/default/grub
   */
  private async saveGrubDefault() {
    const content = this.editorContent();
    const chroot = this.bootService.getChroot();
    const grubPath = chroot ? `${chroot}/etc/default/grub` : '/etc/default/grub';
    await this.writeFileWithPkexec(grubPath, content);
  }

  /**
   * Write content to a file using pkexec
   */
  private async writeFileWithPkexec(filePath: string, content: string): Promise<void> {
    const tempFile = `/tmp/garuda-grub-edit-${Date.now()}.tmp`;

    const writeCmd = `cat > "${tempFile}" << 'GRUBEDIT_EOF'\n${content}\nGRUBEDIT_EOF`;
    const writeResult = await this.taskManager.executeAndWaitBash(writeCmd);
    if (writeResult.code !== 0) {
      throw new Error(`Failed to create temp file: ${writeResult.stderr}`);
    }

    const copyCmd = `pkexec cp "${tempFile}" "${filePath}"`;
    const copyResult = await this.taskManager.executeAndWaitBash(copyCmd);
    if (copyResult.code !== 0) {
      throw new Error(`Failed to write file: ${copyResult.stderr}`);
    }

    await this.taskManager.executeAndWaitBash(`rm -f "${tempFile}"`);
  }
}
