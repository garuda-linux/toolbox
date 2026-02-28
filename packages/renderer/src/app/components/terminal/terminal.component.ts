import {
  type AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  ElementRef,
  HostListener,
  inject,
  type OnDestroy,
  type OnInit,
  signal,
  type Signal,
  viewChild,
} from '@angular/core';
import { NgTemplateOutlet } from '@angular/common';
import type { ITerminalOptions, ITheme } from '@xterm/xterm';
import { CatppuccinXtermJs } from '../../theme';
import { type NgTerminal, NgTerminalModule } from 'ng-terminal';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { Dialog } from 'primeng/dialog';
import { ProgressBar } from 'primeng/progressbar';
import { Card } from 'primeng/card';
import { ScrollPanel } from 'primeng/scrollpanel';
import { Logger } from '../../logging/logging';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { ConfigService } from '../config/config.service';
import { Subscription } from 'rxjs';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { clear, writeText } from '../../electron-services';
import { MessageToastService } from '@garudalinux/core';
import { GarudaBin } from '../privatebin/privatebin';
import { LoadingService } from '../loading-indicator/loading-indicator.service';
import { DesignerService } from '../designer/designerservice';
import { FitAddon } from '@xterm/addon-fit';

@Component({
  selector: 'toolbox-terminal',
  imports: [NgTerminalModule, TranslocoDirective, Dialog, ProgressBar, Card, ScrollPanel, NgTemplateOutlet],
  templateUrl: './terminal.component.html',
  styleUrl: './terminal.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class TerminalComponent implements OnInit, AfterViewInit, OnDestroy {
  public visible = signal<boolean>(false);
  private subscriptions: Subscription[] = [];

  readonly dialog = viewChild<Dialog>('dialog');
  readonly term = viewChild<NgTerminal>('term');

  protected readonly taskManagerService = inject(TaskManagerService);
  private readonly configService = inject(ConfigService);
  private readonly designerService = inject(DesignerService);
  private readonly garudaBin = inject(GarudaBin);
  private readonly loadingService = inject(LoadingService);
  private readonly logger = Logger.getInstance();
  private readonly messageToastService = inject(MessageToastService);
  private readonly translocoService = inject(TranslocoService);

  private fitAddon = new FitAddon();
  private host = inject(ElementRef);
  private observer: ResizeObserver | null = null;

  readonly progressTracker = computed(() => {
    const progress = this.taskManagerService.progress();
    if (progress === null) {
      return null;
    }
    const count = this.taskManagerService.count();
    if (count <= 1) {
      return (progress === 0 ? 0 : 100).toPrecision(1);
    }
    return (((progress - 1) / (count - 1)) * 100).toPrecision(1);
  });
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
      const darkMode = this.configService.settings().darkMode;
      const appTheme = this.configService.settings().activeTheme;
      const term = this.term();
      if (term?.underlying) {
        let theme: ITheme = darkMode ? CatppuccinXtermJs.dark : CatppuccinXtermJs.light;
        if (!appTheme.includes('Catppuccin Mocha')) {
          theme = this.designerService.getXtermTheme(darkMode);
        }
        term.underlying.options.theme = theme;
      }
      this.logger.trace('Terminal theme switched via effect');
    });
  }

  ngOnInit(): void {
    this.subscriptions.push(
      this.taskManagerService.events.subscribe((output: string) => {
        if (output === 'show') this.visible.set(true);
        else if (output === 'hide') this.visible.set(false);
      }),
    );

    this.observer = new ResizeObserver(() => this.fitAddon.fit());
    this.observer.observe(this.host.nativeElement);
  }

  async ngAfterViewInit() {
    this.logger.debug('Terminal component initialized');
    await this.loadXterm();

    this.logger.trace('Subscribing to terminal output/clear emitter');
    this.subscriptions.push(
      this.taskManagerService.dataEvents.subscribe((output: string) => {
        this.term()?.write(output);
      }),
    );
    this.subscriptions.push(
      this.taskManagerService.events.subscribe((output: string) => {
        if (output === 'clear') this.term()?.underlying?.clear();
      }),
    );
  }

  ngOnDestroy(): void {
    for (const sub of this.subscriptions) {
      sub.unsubscribe();
    }

    this.observer?.unobserve(this.host.nativeElement);
  }

  @HostListener('keydown', ['$event'])
  respondToKeydown(event: KeyboardEvent) {
    if (event.ctrlKey) {
      switch (event.key) {
        case 'q':
          this.visible.set(false);
          break;
        case 'c':
          void this.copyToClipboard();
          break;
        case 'x':
          this.clearCache();
          break;
        case 'e':
          void this.taskManagerService.executeTasks();
          break;
        case 's':
          void this.uploadLog();
      }
    }
  }

  /**
   * Load the xterm terminal into the terminal div, and set up the terminal.
   */
  private async loadXterm(): Promise<void> {
    const term = this.term();
    if (!term) return;

    term.underlying?.loadAddon(new WebLinksAddon());
    term.underlying?.loadAddon(this.fitAddon);
    term.underlying?.clear();

    if (this.taskManagerService.data) {
      this.logger.trace('Terminal output cleared, now writing to terminal');
      term.write(this.taskManagerService.data);
    }
  }

  /**
   * Copy the current terminal output to the clipboard.
   */
  async copyToClipboard() {
    await clear();
    await writeText(this.taskManagerService.cachedData());

    this.messageToastService.success(
      this.translocoService.translate('terminal.copyToClipboardTitle'),
      this.translocoService.translate('terminal.copyToClipboard'),
    );
  }

  /**
   * Clear the cached terminal output.
   */
  clearCache() {
    this.taskManagerService.cachedData.set('');
    this.messageToastService.success(
      this.translocoService.translate('terminal.clearCachedDataTitle'),
      this.translocoService.translate('terminal.clearCachedData'),
    );
  }

  /**
   * Upload the current terminal output to PrivateBin and copy the URL to the clipboard.
   */
  async uploadLog() {
    this.logger.trace('Uploading output to PrivateBin');
    this.loadingService.loadingOn();

    const url: string = await this.garudaBin.sendText(this.taskManagerService.cachedData());
    this.logger.info(`Uploaded to ${url}`);

    void writeText(url);
    this.messageToastService.info(
      this.translocoService.translate('diagnostics.success'),
      this.translocoService.translate('diagnostics.uploadSuccess'),
    );

    this.loadingService.loadingOff();
  }
}
