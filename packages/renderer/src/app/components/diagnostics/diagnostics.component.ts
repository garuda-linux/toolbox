import {
  type AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  type OnInit,
  type Signal,
  viewChild,
} from '@angular/core';
import { Button } from 'primeng/button';
import { Card } from 'primeng/card';
import { Tooltip } from 'primeng/tooltip';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { CatppuccinXtermJs } from '../../theme';
import type { ITerminalOptions, ITheme } from '@xterm/xterm';
import { clear, writeText } from '../../electron-services';
import { MessageToastService } from '@garudalinux/core';
import { GarudaBin } from '../privatebin/privatebin';
import { type NgTerminal, NgTerminalModule } from 'ng-terminal';
import { LoadingService } from '../loading-indicator/loading-indicator.service';
import { ConfigService } from '../config/config.service';
import { Logger } from '../../logging/logging';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { Router, type UrlTree } from '@angular/router';
import { DesignerService } from '../designer/designerservice';

@Component({
  selector: 'toolbox-diagnostics',
  imports: [Button, Card, Tooltip, TranslocoDirective, NgTerminalModule],
  templateUrl: './diagnostics.component.html',
  styleUrl: './diagnostics.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DiagnosticsComponent implements AfterViewInit, OnInit {
  private outputCache = '';

  readonly term = viewChild<NgTerminal>('term');

  private readonly configService = inject(ConfigService);
  private readonly designerService = inject(DesignerService);
  private readonly garudaBin = inject(GarudaBin);
  private readonly loadingService = inject(LoadingService);
  private readonly logger = Logger.getInstance();
  private readonly messageToastService = inject(MessageToastService);
  private readonly translocoService = inject(TranslocoService);
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly router = inject(Router);

  diagnosticTools = [
    { id: 'inxi', label: 'diagnostics.inxi', icon: 'pi pi-info-circle', description: 'diagnostics.inxiDesc' },
    { id: 'garuda-health', label: 'diagnostics.health', icon: 'pi pi-heart', description: 'diagnostics.healthDesc' },
    {
      id: 'systemd-analyze',
      label: 'diagnostics.systemctl',
      icon: 'pi pi-chart-bar',
      description: 'diagnostics.systemctlDesc',
    },
    {
      id: 'journalctl',
      label: 'diagnostics.journalctl',
      icon: 'pi pi-list',
      description: 'diagnostics.journalctlDesc',
    },
    { id: 'pacman', label: 'diagnostics.pacman', icon: 'pi pi-box', description: 'diagnostics.pacmanDesc' },
    {
      id: 'dmesg',
      label: 'diagnostics.dmesg',
      icon: 'pi pi-exclamation-triangle',
      description: 'diagnostics.dmesgDesc',
    },
    { id: 'full', label: 'diagnostics.full', icon: 'pi pi-file-import', description: 'diagnostics.fullDesc' },
    { id: 'upload', label: 'diagnostics.upload', icon: 'pi pi-upload', description: 'diagnostics.uploadDesc' },
  ];

  readonly xtermOptions: Signal<ITerminalOptions> = computed(() => {
    let theme: ITheme = this.configService.settings().darkMode ? CatppuccinXtermJs.dark : CatppuccinXtermJs.light;
    if (!this.configService.settings().activeTheme.includes('Catppuccin Mocha')) {
      const isDarkMode = this.configService.settings().darkMode;
      theme = this.designerService.getXtermTheme(isDarkMode);
    }

    return {
      disableStdin: false,
      scrollback: 10000,
      convertEol: true,
      theme: theme,
    };
  });

  constructor() {
    effect(() => {
      const _darkMode: boolean = this.configService.settings().darkMode;
      const term = this.term();
      if (term?.underlying) {
        term.underlying.options.theme = this.xtermOptions().theme;
      }
      this.logger.trace('Terminal theme switched via effect');
    });

    this.logger.debug('Diagnostics component initialized');
  }

  async ngOnInit(): Promise<void> {
    const url: UrlTree = this.router.parseUrl(this.router.url);
    if (!url.queryParams['action']) return;

    if (url.queryParams['action']) {
      switch (url.queryParams['action']) {
        case 'inxi':
        case 'systemd-analyze':
        case 'journalctl':
        case 'dmesg':
        case 'pacman':
        case 'full-logs':
          await this.getOutput(url.queryParams['action']);
          break;
        default:
          this.logger.error('Invalid action');
      }
      if (url.queryParams['upload'] === 'true') {
        await this.uploadPrivateBin();
      }
    }
  }

  /**
   * Set the fragment in the URL.
   * @param fragment The fragment to navigate to.
   */
  navigate(fragment: string) {
    void this.router.navigate([], { fragment });
  }

  ngAfterViewInit(): void {
    this.term()?.underlying?.loadAddon(new WebLinksAddon());
  }

  /**
   * Get the full logs for the system and copy them to the clipboard.
   */
  async getFullLogs(): Promise<void> {
    this.logger.debug('Getting full logs');
    this.loadingService.loadingOn();

    let cmd = '';
    for (const type of ['inxi', 'systemd-analyze', 'journalctl', 'pacman-log', 'dmesg', 'health']) {
      const command = this.getCommand(type);
      if (!command) {
        this.logger.error(`Failed to get command for ${type}`);
        continue;
      }

      cmd += `echo "### ${type} ###"; ${command.cmd}; echo "### END ${command.cmd} ###"; echo`;
    }

    const result = await this.taskManagerService.executeAndWaitBash(`pkexec sh -c '${cmd}'`);

    this.loadingService.loadingOff();

    await this.processResult(result);
  }

  /**
   * Upload the output to PrivateBin and copy the URL to the clipboard.
   */
  async uploadPrivateBin() {
    this.logger.trace('Uploading buffer to PrivateBin');
    this.loadingService.loadingOn();

    const url: string = await this.garudaBin.sendText(this.outputCache);
    this.logger.info(`Uploaded to ${url}`);

    await writeText(url);
    this.messageToastService.info(
      this.translocoService.translate('diagnostics.success'),
      this.translocoService.translate('diagnostics.uploadSuccess'),
    );

    this.loadingService.loadingOff();
  }

  /**
   * Get the output for the specified type of diagnostic.
   * @param type The type of diagnostic to get the output for.
   */
  async getOutput(type: string): Promise<void> {
    this.logger.debug(`Getting output for ${type}`);

    this.loadingService.loadingOn();
    try {
      const command = this.getCommand(type);
      if (!command) {
        this.messageToastService.error(
          this.translocoService.translate('diagnostics.failedCmdHeader'),
          this.translocoService.translate('diagnostics.failedCmd'),
        );
        return;
      }

      this.logger.trace(`Getting output for ${type}`);
      const result = await this.executeCommand(command.cmd, command.sudo);
      await this.processResult(result);
    } catch (err: any) {
      this.logger.trace(`Error getting output for ${type}: ${err}`);
    }

    this.logger.trace(`Done getting output for ${type}`);
    this.loadingService.loadingOff();
  }

  /**
   * Process the result of the command execution.
   * @param result The result of the command execution
   * @private
   */
  private async processResult(result: any): Promise<void> {
    const term = this.term();
    if (!term) return;

    if (result.code === 0) {
      this.logger.trace('Writing to clear terminal and buffer');
      term.underlying?.clear();
      term.write(result.stdout);

      this.outputCache = result.stdout;

      if (this.configService.settings().copyDiagnostics) {
        this.logger.trace('Writing to clipboard');
        await clear();
        await writeText(result.stdout);
        this.messageToastService.info(
          this.translocoService.translate('diagnostics.copySuccess'),
          this.translocoService.translate('diagnostics.copySuccess'),
        );
      }
    } else {
      this.messageToastService.error(this.translocoService.translate('diagnostics.failedCmdHeader'), result.stderr);
      this.logger.error(`Error collecting output: ${result.stderr}`);
    }
  }

  /**
   * Assemble the command to be executed, optionally with sudo and prompting for the password if needed.
   * @param command The command to be executed.
   * @param needsSudo Whether the command needs to be run with sudo.
   */
  private async executeCommand(command: string, needsSudo = false): Promise<any> {
    if (needsSudo) {
      return await this.taskManagerService.executeAndWaitBash(`pkexec ${command}`);
    }
    return await this.taskManagerService.executeAndWaitBash(command);
  }

  /**
   * Get the command to be executed based on the type of diagnostic.
   * @param command The type of diagnostic to get the command for.
   * @private
   */
  private getCommand(command: string): { sudo: boolean; cmd: string } | null {
    const result = { sudo: false, cmd: '' };

    switch (command) {
      case 'inxi':
        result.cmd = 'garuda-inxi';
        break;
      case 'systemd-analyze':
        result.cmd = 'systemd-analyze blame --no-pager && systemd-analyze critical-chain --no-pager';
        break;
      case 'journalctl':
        result.cmd = 'journalctl -e --no-pager';
        result.sudo = true;
        break;
      case 'pacman':
        result.cmd = "tac /var/log/pacman.log | awk '!flag; /PACMAN.*pacman/{flag = 1};' | tac";
        break;
      case 'dmesg':
        result.cmd = 'dmesg';
        result.sudo = true;
        break;
      case 'garuda-health':
        result.cmd = 'garuda-health';
        result.sudo = true;
        break;
      default:
        this.logger.error('Invalid type');
        return null;
    }

    return result;
  }
}
