import {
  AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  OnDestroy,
  OnInit,
  type Signal,
  viewChild,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { Card } from 'primeng/card';
import { Button } from 'primeng/button';
import { TableModule } from 'primeng/table';
import { TabsModule } from 'primeng/tabs';
import { ToggleButton } from 'primeng/togglebutton';
import { InputText } from 'primeng/inputtext';
import { Tag } from 'primeng/tag';
import { InputNumber } from 'primeng/inputnumber';
import { Tooltip } from 'primeng/tooltip';
import { MultiSelectModule } from 'primeng/multiselect';
import { NgTerminal, NgTerminalModule } from 'ng-terminal';
import type { ITerminalOptions, ITheme } from '@xterm/xterm';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { NetworkAssistantService } from './network-assistant.service';
import { MessageToastService } from '@garudalinux/core';
import { ElectronShellSpawnService } from '../../electron-services/electron-shell-spawn.service';
import { ConfigService } from '../config/config.service';
import { CatppuccinXtermJs } from '../../theme';
import { DesignerService } from '../designer/designerservice';
import { LoadingService } from '../loading-indicator/loading-indicator.service';
import { Select } from 'primeng/select';
import { NetworkStatus } from './types';

@Component({
  selector: 'toolbox-network-assistant',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    TranslocoDirective,
    Card,
    Button,
    TableModule,
    TabsModule,
    ToggleButton,
    InputText,
    Tag,
    InputNumber,
    Tooltip,
    MultiSelectModule,
    NgTerminalModule,
    Select,
  ],
  templateUrl: './network-assistant.component.html',
  styleUrl: './network-assistant.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NetworkAssistantComponent implements OnInit, AfterViewInit, OnDestroy {
  protected readonly networkService = inject(NetworkAssistantService);
  private readonly toast = inject(MessageToastService);
  private readonly transloco = inject(TranslocoService);
  private readonly shellSpawn = inject(ElectronShellSpawnService);
  private readonly configService = inject(ConfigService);
  private readonly designerService = inject(DesignerService);
  private readonly loadingService = inject(LoadingService);

  private activeProcessId: string | null = null;

  readonly term = viewChild<NgTerminal>('term');

  readonly driverStatusOptions = computed(() => [
    { label: 'Loaded', value: 'loaded', severity: 'success' },
    { label: 'Unloaded', value: 'unloaded', severity: 'warn' },
    { label: 'Blacklisted', value: 'blacklisted', severity: 'danger' },
  ]);
  readonly driversForTable = computed(() =>
    this.drivers().map((driver) => ({
      ...driver,
      statusFilter: driver.status.startsWith('blacklisted') ? 'blacklisted' : driver.status,
    })),
  );
  readonly driverCategoryOptions = computed(() => [
    { label: 'Ethernet', value: 'ethernet' },
    { label: 'Wireless', value: 'wireless' },
    { label: 'Other', value: 'other' },
  ]);

  readonly status = this.networkService.status;
  readonly drivers = this.networkService.drivers;
  readonly hardware = this.networkService.hardware;

  readonly ethernetHardware = this.networkService.ethernetHardware;
  readonly wirelessHardware = this.networkService.wirelessHardware;
  readonly bluetoothHardware = this.networkService.bluetoothHardware;

  readonly pingOptions = this.networkService.pingOptions;
  readonly tracerouteOptions = this.networkService.tracerouteOptions;

  readonly xtermOptions: Signal<ITerminalOptions> = computed(() => {
    let theme: ITheme = this.configService.settings().darkMode ? CatppuccinXtermJs.dark : CatppuccinXtermJs.light;
    if (!this.configService.settings().activeTheme.includes('Catppuccin Mocha')) {
      const isDarkMode = this.configService.settings().darkMode;
      theme = this.designerService.getXtermTheme(isDarkMode);
    }
    return {
      disableStdin: true,
      scrollback: 5000,
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
    });
  }

  private revertToggle<
    K extends keyof Pick<NetworkStatus, 'wifiEnabled' | 'btEnabled' | 'airplaneEnabled' | 'gpsEnabled'>,
  >(key: K, previousValue: NetworkStatus[K]): void {
    this.networkService.status.update((current) => (current ? { ...current, [key]: previousValue } : current));
  }

  private tr(key: string): string {
    return this.transloco.translate(key);
  }

  private refreshStatusNonBlocking(): void {
    void this.networkService.refreshStatus();
  }

  ngOnInit() {
    void this.networkService.refreshStatus();
  }

  ngAfterViewInit(): void {
    this.term()?.underlying?.loadAddon(new WebLinksAddon());
  }

  ngOnDestroy(): void {
    this.killCurrentProcess();
  }

  private killCurrentProcess(): void {
    if (this.activeProcessId) {
      this.shellSpawn.killProcess(this.activeProcessId);
      this.activeProcessId = null;
    }
  }

  private writeToTerminal(text: string): void {
    this.term()?.write(text);
  }

  private clearTerminal(): void {
    this.term()?.underlying?.clear();
  }

  async runPing() {
    this.killCurrentProcess();
    const opts = this.pingOptions();
    this.clearTerminal();
    this.writeToTerminal(`$ ping -c ${opts.count} -W ${opts.timeout} ${opts.host}\r\n`);

    const { processId } = await this.shellSpawn.spawnStreaming(
      'ping',
      ['-c', opts.count.toString(), '-W', opts.timeout.toString(), opts.host],
      {
        onStdout: (data) => this.writeToTerminal(data),
        onStderr: (data) => this.writeToTerminal(data),
        onClose: () => {
          this.activeProcessId = null;
        },
      },
    );

    this.activeProcessId = processId ?? null;
  }

  async runTraceroute() {
    this.killCurrentProcess();
    const opts = this.tracerouteOptions();
    this.clearTerminal();

    this.writeToTerminal(`$ traceroute -m ${opts.maxHops} ${opts.host}\r\n`);

    const { processId } = await this.shellSpawn.spawnStreaming(
      'traceroute',
      ['-m', opts.maxHops.toString(), opts.host],
      {
        onStdout: (data) => this.writeToTerminal(data),
        onStderr: (data) => this.writeToTerminal(data),
        onClose: () => {
          this.activeProcessId = null;
        },
      },
    );

    this.activeProcessId = processId ?? null;
  }

  async toggleWifi(enabled: boolean) {
    const previousValue = this.status()?.wifiEnabled ?? !enabled;
    try {
      await this.networkService.toggleWifi(enabled);
    } catch {
      this.revertToggle('wifiEnabled', previousValue);
      this.toast.error(this.tr('networkAssistant.error'), this.tr('networkAssistant.toggleWifiFailed'));
    } finally {
      this.refreshStatusNonBlocking();
    }
  }

  async toggleBt(enabled: boolean) {
    const previousValue = this.status()?.btEnabled ?? !enabled;
    try {
      await this.networkService.toggleBt(enabled);
    } catch {
      this.revertToggle('btEnabled', previousValue);
      this.toast.error(this.tr('networkAssistant.error'), this.tr('networkAssistant.toggleBtFailed'));
    } finally {
      this.refreshStatusNonBlocking();
    }
  }

  async toggleAirplane(enabled: boolean) {
    const previousValue = this.status()?.airplaneEnabled ?? !enabled;
    try {
      await this.networkService.toggleAirplane(enabled);
    } catch {
      this.revertToggle('airplaneEnabled', previousValue);
      this.toast.error(this.tr('networkAssistant.error'), this.tr('networkAssistant.toggleAirplaneFailed'));
    } finally {
      this.refreshStatusNonBlocking();
    }
  }

  async toggleGps(enabled: boolean) {
    const previousValue = this.status()?.gpsEnabled ?? !enabled;
    try {
      await this.networkService.toggleGps(enabled);
    } catch {
      this.revertToggle('gpsEnabled', previousValue);
      this.toast.error(this.tr('networkAssistant.error'), this.tr('networkAssistant.toggleGpsFailed'));
    } finally {
      this.refreshStatusNonBlocking();
    }
  }

  async enableService(service: string) {
    try {
      await this.networkService.enableService(service);
      this.toast.success(this.tr('networkAssistant.success'), this.tr('networkAssistant.serviceEnabled'));
    } catch {
      this.toast.error(this.tr('networkAssistant.error'), this.tr('networkAssistant.enableServiceFailed'));
    } finally {
      this.refreshStatusNonBlocking();
    }
  }

  async loadDriver(name: string) {
    try {
      await this.networkService.loadDriver(name);
      this.toast.success(this.tr('networkAssistant.success'), this.tr('networkAssistant.driverLoaded'));
    } catch {
      this.toast.error(this.tr('networkAssistant.error'), this.tr('networkAssistant.loadDriverFailed'));
    } finally {
      this.refreshStatusNonBlocking();
    }
  }

  async unloadDriver(name: string) {
    try {
      await this.networkService.unloadDriver(name);
      this.toast.success(this.tr('networkAssistant.success'), this.tr('networkAssistant.driverUnloaded'));
    } catch {
      this.toast.error(this.tr('networkAssistant.error'), this.tr('networkAssistant.unloadDriverFailed'));
    } finally {
      this.refreshStatusNonBlocking();
    }
  }

  async blacklistDriver(name: string, blacklist: boolean) {
    try {
      await this.networkService.blacklistDriver(name, blacklist);
      this.toast.success(
        this.tr('networkAssistant.success'),
        this.tr(blacklist ? 'networkAssistant.driverBlacklisted' : 'networkAssistant.driverUnblacklisted'),
      );
    } catch {
      this.toast.error(this.tr('networkAssistant.error'), this.tr('networkAssistant.blacklistFailed'));
    } finally {
      this.refreshStatusNonBlocking();
    }
  }

  async openWifiHotspot() {
    this.loadingService.loadingOn();
    try {
      await this.networkService.openWifiHotspot();
    } catch {
      this.toast.error(this.tr('networkAssistant.error'), this.tr('networkAssistant.hotspotFailed'));
    } finally {
      this.refreshStatusNonBlocking();
      this.loadingService.loadingOff();
    }
  }

  copyHwItem(text: string) {
    this.networkService.copyToClipboard(text);
    this.toast.success(this.tr('networkAssistant.success'), this.tr('networkAssistant.copied'));
  }

  copyAllHwItems() {
    const items = this.hardware().map((h) => h.raw);
    this.networkService.copyAllToClipboard(items);
    this.toast.success(this.tr('networkAssistant.success'), this.tr('networkAssistant.copied'));
  }

  getHwIcon(type: string): string {
    switch (type) {
      case 'ethernet':
        return 'pi pi-server';
      case 'wireless':
        return 'pi pi-wifi';
      case 'bluetooth':
        return 'pi pi-bluetooth';
      default:
        return 'pi pi-microchip';
    }
  }
}
