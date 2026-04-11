import { computed, effect, inject, Injectable, signal, untracked } from '@angular/core';
import { type Task, TaskManagerService } from './task-manager.service';
import { ConfigService } from '../config/config.service';
import {
  defaultDnsProvider,
  type DnsProvider,
  type DnsProviderEntry,
  dnsProviders,
  type ShellEntry,
  shells,
} from '../system-settings/types';
import type { ChildProcess } from '../../types/shell';
import { Logger } from '../../logging/logging';
import { CONFIGS } from '../config-entries/configs';
import {
  exists,
  homeConfigExists,
  pathResolve,
  readHomeConfig,
  readTextFile,
} from '../../electron-services/electron-api-utils';

@Injectable({
  providedIn: 'root',
})
export class OsInteractService {
  private readonly configService = inject(ConfigService);
  private readonly logger = Logger.getInstance();
  private readonly taskManagerService = inject(TaskManagerService);

  private chrootPath = '';

  // A list of all currently installed packages
  private readonly installedPackages = signal<Map<string, boolean>>(new Map());
  private readonly currentLocales = signal<Map<string, boolean>>(new Map());
  private readonly currentServices = signal<Map<string, boolean>>(new Map());
  private readonly currentServicesUser = signal<Map<string, boolean>>(new Map());
  private readonly currentGroups = signal<Map<string, boolean>>(new Map());

  private readonly currentDNS = signal<DnsProvider>(defaultDnsProvider);
  private readonly currentShell = signal<ShellEntry | null>(null);
  private readonly currentHblock = signal<boolean>(false);
  private readonly currentIwd = signal<boolean>(false);
  private readonly currentConfigs = signal<Map<string, boolean>>(new Map());
  private readonly currentGrub = signal<Map<string, string>>(new Map());
  private readonly currentPlymouthTheme = signal<string | null>(null);
  private readonly isInitialized = signal<boolean>(false);

  private readonly wantedPackages = signal<Map<string, boolean>>(new Map());
  private readonly wantedLocales = signal<Map<string, boolean>>(new Map());
  private readonly wantedPackagesAur = signal<Map<string, boolean>>(new Map());
  private readonly wantedServices = signal<Map<string, boolean>>(new Map());
  private readonly wantedServicesUser = signal<Map<string, boolean>>(new Map());
  private readonly wantedGroups = signal<Map<string, boolean>>(new Map());
  private readonly wantedConfigs = signal<Map<string, boolean>>(new Map());
  private readonly wantedGrub = signal<Map<string, string>>(new Map());
  private readonly wantedPlymouthTheme = signal<string | null>(null);
  private readonly wantedBootScript = signal<string | null>(null);

  readonly wantedDns = signal<DnsProvider | null>(null);
  readonly wantedShell = signal<ShellEntry | null>(null);
  readonly wantedHblock = signal<boolean | null>(null);
  readonly wantedIwd = signal<boolean | null>(null);

  readonly packages = computed(() => {
    return new Map([...this.installedPackages(), ...this.wantedPackages(), ...this.wantedPackagesAur()]);
  });
  readonly locales = computed(() => {
    return new Map([...this.currentLocales(), ...this.wantedLocales()]);
  });
  readonly services = computed(() => {
    return new Map([...this.currentServices(), ...this.wantedServices()]);
  });
  readonly servicesUser = computed(() => {
    return new Map([...this.currentServicesUser(), ...this.wantedServicesUser()]);
  });
  readonly groups = computed(() => {
    return new Map([...this.currentGroups(), ...this.wantedGroups()]);
  });

  readonly dns = computed(() => {
    return this.wantedDns() ?? this.currentDNS();
  });
  readonly shell = computed(() => {
    return this.wantedShell() ?? this.currentShell();
  });
  readonly hblock = computed(() => {
    return this.wantedHblock() ?? this.currentHblock();
  });
  readonly iwd = computed(() => {
    return this.wantedIwd() ?? this.currentIwd();
  });
  readonly configs = computed(() => {
    return new Map([...this.currentConfigs(), ...this.wantedConfigs()]);
  });

  constructor() {
    effect(() => {
      if (!this.taskManagerService.running()) {
        void untracked(async () => await this.update());
      }
    });
    effect(() => {
      this.wantedPackages.set(this.wantedPrune(untracked(this.wantedPackages), this.installedPackages()));
      this.wantedPackagesAur.set(this.wantedPrune(untracked(this.wantedPackagesAur), this.installedPackages()));
      this.wantedServices.set(this.wantedPrune(untracked(this.wantedServices), this.currentServices()));
      this.wantedServicesUser.set(this.wantedPrune(untracked(this.wantedServicesUser), this.currentServicesUser()));
      this.wantedGroups.set(this.wantedPrune(untracked(this.wantedGroups), this.currentGroups()));
      this.wantedLocales.set(this.wantedPrune(untracked(this.wantedLocales), this.currentLocales()));

      const cGrub = this.currentGrub();
      this.wantedGrub.update((w) => {
        const pruned = new Map(w);
        for (const [key, val] of w) {
          if (cGrub.get(key) === val) {
            pruned.delete(key);
          }
        }
        return pruned;
      });

      const cPly = this.currentPlymouthTheme();
      this.wantedPlymouthTheme.update((w) => {
        if (w !== null && w === cPly) return null;
        return w;
      });
    });
    effect(() => this.generateTasks());
  }

  /**
   * Generate the tasks to be executed based on the wanted state and the current state.
   * @private
   */
  private generateTasks(): void {
    const install: string[] = [];
    const uninstall: string[] = [];
    const installAur: string[] = [];
    const enable: string[] = [];
    const disable: string[] = [];
    const enableUser: string[] = [];
    const disableUser: string[] = [];
    const groupAdd: string[] = [];
    const groupRemove: string[] = [];
    const localeAdd: string[] = [];
    const localeRemove: string[] = [];

    for (const [pkg, wanted] of this.wantedPackages()) {
      if (wanted) {
        install.push(pkg);
      } else {
        uninstall.push(pkg);
      }
    }

    for (const [pkg, wanted] of this.wantedPackagesAur()) {
      if (wanted) {
        installAur.push(pkg);
      } else {
        uninstall.push(pkg);
      }
    }

    for (const [service, wanted] of this.wantedServices()) {
      if (wanted) {
        enable.push(service);
      } else {
        disable.push(service);
      }
    }

    for (const [service, wanted] of this.wantedServicesUser()) {
      if (wanted) {
        enableUser.push(service);
      } else {
        disableUser.push(service);
      }
    }

    for (const [group, wanted] of this.wantedGroups()) {
      if (wanted) {
        groupAdd.push(group);
      } else {
        groupRemove.push(group);
      }
    }

    for (const [locale, wanted] of this.wantedLocales()) {
      this.logger.trace(`Locale ${locale} wanted: ${wanted}`);
      if (wanted) {
        localeAdd.push(locale);
      } else {
        localeRemove.push(locale);
      }
    }

    let script_packages = '';
    if (uninstall.length > 0) {
      script_packages += `pacman --noconfirm -R ${uninstall.join(' ')}\n`;
    }
    if (install.length > 0) {
      script_packages += `pacman --noconfirm -S ${install.join(' ')}\n`;
    }

    let script_packages_aur = '';
    if (installAur.length > 0) {
      script_packages_aur += `paru --noconfirm -S ${install.join(' ')}`;
    }

    let script_services = '';
    if (enable.length > 0) {
      script_services += `systemctl enable --now ${enable.join(' ')}\n`;
    }
    if (disable.length > 0) {
      script_services += `systemctl disable --now ${disable.join(' ')}\n`;
    }

    if (this.dns() !== this.currentDNS()) {
      const file = '/etc/NetworkManager/conf.d/10-garuda-assistant-dns.conf';
      if (this.dns().ips[0] === '0.0.0.0') {
        script_services += `
            set -e
            rm -f "${file}"
            nmcli general reload
            echo "DNS settings changed."
            `;
      } else {
        script_services += `
            set -e
            echo -e "[global-dns-domain-*]\\nservers=${this.dns().ips[0]}" > "${file}"
            nmcli general reload
            echo "DNS settings changed."
            `;
      }
    }

    const shell = this.shell();
    if (shell !== null && shell !== this.currentShell()) {
      script_services += `
            set -e
            chsh -s ${shell.path ?? '$(which ${shell.name})'} ${this.configService.state().user}
            echo "Shell changed."
            `;
    }

    if (this.hblock() !== this.currentHblock()) {
      if (this.hblock()) {
        script_services += `systemctl enable --now hblock.timer && hblock\n`;
      } else {
        script_services += `systemctl disable --now hblock.timer && hblock -S none -D none\n`;
      }
    }

    if (this.iwd() !== this.currentIwd()) {
      if (this.iwd()) {
        script_services += `
          set -e
          systemctl stop NetworkManager
          systemctl disable --now wpa_supplicant.service
          systemctl mask wpa_supplicant
          echo -e "[device]\\nwifi.backend=iwd" > /etc/NetworkManager/conf.d/20_wifi_backend_toolbox.conf
          systemctl daemon-reload
          systemctl start NetworkManager
          echo "Changed NetworkManager backend to iwd."\n
        `;
      } else {
        script_services += `
          set -e
          systemctl stop NetworkManager
          rm /etc/NetworkManager/conf.d/20_wifi_backend_{rani,toolbox}.conf
          systemctl unmask wpa_supplicant
          systemctl enable --now wpa_supplicant
          systemctl daemon-reload
          systemctl start NetworkManager
          echo "Changed NetworkManager backend to wpa_supplicant."\n
        `;
      }
    }

    let script_configs_user = '';
    let script_configs_sudo = '';

    for (const config of CONFIGS) {
      const wanted = this.wantedConfigs().get(config.key);
      const current = this.currentConfigs().get(config.key);
      if (wanted !== undefined && wanted !== current) {
        const prefix = config.sudo ? '' : `${this.configService.state().userHome}/`;
        let newContent = '';

        if (config.type === 'file') {
          if (wanted) {
            newContent += `mkdir -p "$(dirname "${prefix}${config.path}")"\n`;
            newContent += `cat > "${prefix}${config.path}" << 'EOFCONFIG'\n${config.content}EOFCONFIG\n`;
          } else {
            newContent += `rm -f "${prefix}${config.path}"\n`;
          }
        } else if (config.type === 'regex' && config.regex) {
          const replacement = wanted ? config.regex.enableReplacement : config.regex.disableReplacement;
          newContent += `mkdir -p "$(dirname "${prefix}${config.regex.file}")"\n`;
          newContent += `sed -i 's/${config.regex.pattern}/${replacement}/g' "${prefix}${config.regex.file}"\n`;
        }

        if (config.sudo) {
          script_configs_sudo += newContent;
        } else {
          script_configs_user += newContent;
        }
      }
    }

    let script_services_user = '';
    if (enableUser.length > 0) {
      script_services_user += `systemctl --user enable --now ${enableUser.join(' ')}\n`;
    }
    if (disableUser.length > 0) {
      script_services_user += `systemctl --user disable --now ${disableUser.join(' ')}\n`;
    }

    let script_groups = '';
    if (groupAdd.length > 0) {
      for (const group of groupAdd) {
        script_groups += `gpasswd -a ${this.configService.state().user} ${group}\n`;
      }
    }
    if (groupRemove.length > 0) {
      for (const group of groupRemove) {
        script_groups += `gpasswd -d ${this.configService.state().user} ${group}\n`;
      }
    }

    const sedExpressions: string[] = [];
    let script_locales = '';
    if (localeAdd.length > 0) {
      for (const locale of localeAdd) {
        sedExpressions.push(`-e 's/#${locale}/${locale}/'`);
      }
    }
    if (localeRemove.length > 0) {
      for (const locale of localeRemove) {
        sedExpressions.push(`-e 's/${locale}/#${locale}/'`);
      }
    }
    if (sedExpressions.length > 0) script_locales = `sed -i ${sedExpressions.join(' ')} /etc/locale.gen\nlocale-gen\n`;

    untracked(() => {
      [
        this.taskManagerService.findTaskById('os-interact-packages'),
        this.taskManagerService.findTaskById('os-interact-packages-aur'),
        this.taskManagerService.findTaskById('os-interact-services'),
        this.taskManagerService.findTaskById('os-interact-services-user'),
        this.taskManagerService.findTaskById('os-interact-groups'),
        this.taskManagerService.findTaskById('os-interact-locales'),
        this.taskManagerService.findTaskById('os-interact-configs-user'),
        this.taskManagerService.findTaskById('os-interact-configs-sudo'),
        this.taskManagerService.findTaskById('os-interact-boot-options'),
      ].forEach((task) => {
        if (task !== null) {
          this.taskManagerService.removeTask(task);
        }
      });
    });

    const tasks: any[] = [];

    if (script_packages !== '')
      tasks.push(
        this.taskManagerService.createTask(
          8,
          'os-interact-packages',
          true,
          'os-interact.packages',
          'pi pi-box',
          script_packages,
        ),
      );
    if (script_packages_aur !== '')
      tasks.push(
        this.taskManagerService.createTask(
          9,
          'os-interact-packages-aur',
          true,
          'os-interact.packages-aur',
          'pi pi-box',
          script_packages_aur,
        ),
      );
    if (script_groups !== '')
      tasks.push(
        this.taskManagerService.createTask(
          11,
          'os-interact-groups',
          true,
          'os-interact.groups',
          'pi pi-users',
          script_groups,
        ),
      );
    if (script_services !== '')
      tasks.push(
        this.taskManagerService.createTask(
          12,
          'os-interact-services',
          true,
          'os-interact.services',
          'pi pi-receipt',
          script_services,
        ),
      );
    if (script_services_user !== '')
      tasks.push(
        this.taskManagerService.createTask(
          12,
          'os-interact-services-user',
          false,
          'os-interact.services-user',
          'pi pi-receipt',
          script_services_user,
        ),
      );
    if (script_locales !== '')
      tasks.push(
        this.taskManagerService.createTask(
          13,
          'os-interact-locales',
          true,
          'os-interact.locales',
          'pi pi-languages',
          script_locales,
        ),
      );
    if (script_configs_user !== '') {
      this.logger.debug(`Creating user configs task with script: ${script_configs_user}`);
      tasks.push(
        this.taskManagerService.createTask(
          14,
          'os-interact-configs-user',
          false,
          'os-interact.configs-user',
          'pi pi-cog',
          script_configs_user,
        ),
      );
    }
    if (script_configs_sudo !== '') {
      this.logger.debug(`Creating sudo configs task with script: ${script_configs_sudo}`);
      tasks.push(
        this.taskManagerService.createTask(
          14,
          'os-interact-configs-sudo',
          true,
          'os-interact.configs-system',
          'pi pi-cog',
          script_configs_sudo,
        ),
      );
    }

    const wGrub = this.wantedGrub();
    const wPlyTheme = this.wantedPlymouthTheme();
    const wScript = this.wantedBootScript();

    if (wGrub.size > 0 || wPlyTheme !== null || wScript !== null) {
      let script = '';
      if (wScript) {
        script = wScript;
      } else {
        const prefix = this.chrootPath || '';
        if (prefix) {
          script += `ROOT_PATH="${prefix}"\nmount /dev/$(lsblk -no NAME -p | grep $(basename $ROOT_PATH)) $ROOT_PATH || true\nmount -o bind /dev $ROOT_PATH/dev\nmount -o bind /sys $ROOT_PATH/sys\nmount -o bind /proc $ROOT_PATH/proc\n`;
        }
        const grubFile = prefix ? `${prefix}/etc/default/grub` : '/etc/default/grub';
        const updateGrubCmd = prefix ? `chroot ${prefix} update-grub` : 'update-grub';
        const plymouthCmd = (theme: string) =>
          prefix ? `chroot ${prefix} plymouth-set-default-theme -R ${theme}` : `plymouth-set-default-theme -R ${theme}`;

        script += `GRUB_FILE="${grubFile}"\ntemp_grub=$(mktemp)\ncp "$GRUB_FILE" "$temp_grub"\n`;
        for (const [key, value] of wGrub) {
          script += `if grep -q "^${key}=" "$temp_grub"; then\n  sed -i 's|^${key}=.*|${key}="${value}"|' "$temp_grub"\nelse\n  echo '${key}="${value}"' >> "$temp_grub"\nfi\n`;
        }
        script += `cp "$temp_grub" "$GRUB_FILE"\nrm "$temp_grub"\n`;
        if (wPlyTheme) script += `\n${plymouthCmd(wPlyTheme)}`;
        script += `\n${updateGrubCmd}`;
        if (prefix) script += `\numount $ROOT_PATH/proc $ROOT_PATH/sys $ROOT_PATH/dev\numount $ROOT_PATH\n`;
      }
      tasks.push(
        this.taskManagerService.createTask(
          15,
          'os-interact-boot-options',
          true,
          'os-interact.boot-options',
          'pi pi-hammer',
          script,
        ),
      );
    }

    tasks.forEach((task) => {
      this.taskManagerService.scheduleTask(task);
    });
  }

  /**
   * Prune the wanted map based on the current state.
   * @param wanted The wanted map.
   * @param current The current map.
   * @private
   */
  private wantedPrune(wanted: Map<string, boolean>, current: Map<string, boolean>): Map<string, boolean> {
    return new Map(
      [...wanted].filter(([key, value]) => {
        if (!current.has(key)) {
          return value;
        } else {
          return current.get(key) !== value;
        }
      }),
    );
  }

  setChroot(path: string) {
    this.chrootPath = path;
  }

  getChroot() {
    return this.chrootPath;
  }

  private wrapCommand(command: string): string {
    if (!this.chrootPath) return command;
    return `chroot ${this.chrootPath} ${command}`;
  }

  async readPrivilegedFile(virtualPath: string): Promise<string> {
    const hostPath = this.chrootPath ? await pathResolve(this.chrootPath, virtualPath) : virtualPath;
    const cmd = `pkexec cat "${hostPath}"`;
    const result = await this.taskManagerService.executeAndWaitBash(cmd);
    if (result.code === 0) return result.stdout;
    throw new Error(`Failed to read privileged file ${hostPath}: ${result.stderr || 'Unknown error'}`);
  }

  /**
   * Update the current state of the system asynchronously.
   */
  async update(): Promise<void> {
    await Promise.all([
      this.getServices().then((res) => this.currentServices.set(res)),
      this.getUserServices().then((res) => this.currentServicesUser.set(res)),
      this.getInstalledPackages().then((res) => this.installedPackages.set(res)),
      this.getGroups().then((res) => this.currentGroups.set(res)),
      this.getDNS().then((res) => this.currentDNS.set(res)),
      this.getShell().then((res) => this.currentShell.set(res)),
      this.getHblock().then((res) => this.currentHblock.set(res)),
      this.getLocales().then((res) => this.currentLocales.set(res)),
      this.getIwd().then((res) => this.currentIwd.set(res)),
      this.getConfigs().then((res) => this.currentConfigs.set(res)),
      this.getGrubSettings().then((res) => this.currentGrub.set(res)),
      this.getPlymouthInfo().then((res) => this.currentPlymouthTheme.set(res.currentTheme || null)),
    ]);
    this.isInitialized.set(true);
  }

  async getGrubSettings(): Promise<Map<string, string>> {
    try {
      const content = await this.readPrivilegedFile('/etc/default/grub');
      const map = new Map<string, string>();
      content.split('\n').forEach((line) => {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith('#') || !trimmed.includes('=')) return;
        const [k, ...v] = trimmed.split('=');
        let val = v.join('=').trim();
        if ((val.startsWith('"') && val.endsWith('"')) || (val.startsWith("'") && val.endsWith("'")))
          val = val.substring(1, val.length - 1);
        map.set(k.trim(), val);
      });
      return map;
    } catch (e) {
      this.logger.error(`Failed to fetch GRUB settings: ${e}`);
      return new Map();
    }
  }

  async getPlymouthInfo(): Promise<{
    themes: string[];
    currentTheme: string;
    isInstalled: boolean;
  }> {
    try {
      const inst = await this.taskManagerService.executeAndWaitBash(this.wrapCommand('pacman -Qq plymouth'));
      const isInstalled = inst.code === 0;
      if (!isInstalled) return { themes: [], currentTheme: '', isInstalled: false };
      const list = await this.taskManagerService.executeAndWaitBash(this.wrapCommand('plymouth-set-default-theme -l'));
      const curr = await this.taskManagerService.executeAndWaitBash(this.wrapCommand('plymouth-set-default-theme'));
      return {
        themes: list.stdout
          .trim()
          .split('\n')
          .filter((t) => t.trim()),
        currentTheme: curr.stdout.trim(),
        isInstalled: true,
      };
    } catch (e) {
      this.logger.error(`Failed to fetch Plymouth info: ${e}`);
      return { themes: [], currentTheme: '', isInstalled: false };
    }
  }

  /**
   * Get the currently installed packages.
   * @returns A map of the installed packages.
   * @private
   */
  private async getInstalledPackages(): Promise<Map<string, boolean>> {
    const cmd = 'pacman -Qq';
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);
    if (result.code !== 0) {
      return new Map<string, boolean>();
    }

    return result.stdout
      .trim()
      .split('\n')
      .reduce((map: Map<string, boolean>, pkg: string) => {
        map.set(pkg, true);
        return map;
      }, new Map<string, boolean>());
  }

  /**
   * Get the current services.
   * @returns A map of the services.
   * @private
   */
  private async getServices(): Promise<Map<string, boolean>> {
    const cmd = 'systemctl list-units --type service,socket --full --output json --no-pager';
    const commandoutput: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);
    const result = JSON.parse(commandoutput.stdout.trim()) as any[];

    const services = new Map<string, boolean>();
    for (const service of result) {
      services.set(service['unit'], service['active'] === 'active');
    }

    return services;
  }

  /**
   * Get the current groups of the user.
   * @returns A map of the groups.
   * @private
   */
  private async getUserServices(): Promise<Map<string, boolean>> {
    const cmd = 'systemctl --user list-units --type service,socket --full --output json --no-pager';
    const commandoutput: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);
    const result = JSON.parse(commandoutput.stdout.trim()) as any[];

    const services = new Map<string, boolean>();
    for (const service of result) {
      services.set(service['unit'], service['active'] === 'active');
    }

    return services;
  }

  /**
   * Get the current groups of the user.
   * @returns A map of the groups.
   * @private
   */
  private async getGroups(): Promise<Map<string, boolean>> {
    const cmd = `groups ${this.configService.state().user} | cut -d ' ' -f 3-`;
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);
    if (result.code !== 0) {
      return new Map<string, boolean>();
    }

    return result.stdout
      .trim()
      .split(' ')
      .reduce((map: Map<string, boolean>, group: string) => {
        map.set(group, true);
        return map;
      }, new Map<string, boolean>());
  }

  /**
   * Get the current DNS provider.
   * @returns The current DNS provider.
   * @private
   */
  private async getDNS(): Promise<DnsProvider> {
    const cmd = 'cat /etc/resolv.conf | grep nameserver | head -n 1 | cut -d " " -f 2';
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);
    if (result.code !== 0) return defaultDnsProvider;

    const ip: string = result.stdout.trim();
    const provider: DnsProviderEntry | undefined = dnsProviders.find((provider) => provider.ips.includes(ip));
    if (provider) {
      return provider;
    }
    return defaultDnsProvider;
  }

  /**
   * Get the current shell of the user.
   * @returns The shell entry or null if not found.
   * @private
   */
  private async getShell(): Promise<ShellEntry | null> {
    const cmd = `basename $(/usr/bin/getent passwd $USER | awk -F':' '{print $7}')`;
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);
    if (result.code !== 0) {
      return null;
    }
    const shell: string = result.stdout.trim();
    return shells.find((entry) => entry.name === shell) ?? null;
  }

  /**
   * Get the current status of the iwd backend for NetworkManager.
   * @returns True or false depending on the status.
   * @private
   */
  private async getIwd(): Promise<boolean> {
    const cmd = `grep -q wifi.backend=iwd /etc/NetworkManager/conf.d/*.conf`;
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);
    return result.code === 0;
  }

  /**
   * Get the current state of hblock.
   * @returns Whether hblock is enabled.
   * @private
   */
  private async getHblock(): Promise<boolean> {
    const cmd = 'cat /etc/hosts | grep -A1 "Blocked domains" | awk \'/Blocked domains/ { print $NF }\'';
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);
    if (result.code !== 0) {
      return false;
    }
    return parseInt(result.stdout.trim()) > 0;
  }

  /**
   * Get the current state of all configs.
   * @returns A map of config keys to enabled state.
   * @private
   */
  private async getConfigs(): Promise<Map<string, boolean>> {
    const configs = new Map<string, boolean>();
    for (const config of CONFIGS) {
      if (config.type === 'file') {
        const result = config.sudo
          ? { exists: await exists(config.path as string) }
          : await homeConfigExists(config.path as string);
        configs.set(config.key, result.exists);
      } else if (config.type === 'regex' && config.regex) {
        let content = '';
        let success = false;
        if (config.sudo) {
          try {
            content = await readTextFile(config.regex.file);
            success = true;
          } catch (e) {
            this.logger.error(`Failed to read sudo config ${config.regex.file}: ${e}`);
          }
        } else {
          const result = await readHomeConfig(config.regex.file);
          content = result.content || '';
          success = result.success;
        }

        if (success && content) {
          const key = config.regex.enableReplacement.split('=')[0];
          const regex = new RegExp(`^\\s*${key.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\s*=\\s*true\\s*$`, 'm');
          const matches = regex.test(content);
          this.logger.debug(`Config ${config.key} (regex): key="${key}", matches=${matches}`);
          configs.set(config.key, matches);
        } else {
          this.logger.debug(`Config ${config.key} (regex): no content or failed read, defaulting to false`);
          configs.set(config.key, false);
        }
      }
    }

    this.logger.debug(`getConfigs result: ${JSON.stringify(Object.fromEntries(configs))}`);
    return configs;
  }

  setWantedConfig(key: string, value: boolean): void {
    const newMap = new Map(this.wantedConfigs());
    newMap.set(key, value);
    this.wantedConfigs.set(newMap);
  }

  toggleConfigState(key: string): void {
    const current = this.configs().get(key) ?? false;
    this.setWantedConfig(key, !current);
  }

  setGrubSetting(key: string, value: string): void {
    this.wantedGrub.update((m) => {
      const nm = new Map(m);
      const current = this.currentGrub().get(key);
      const defaults: Record<string, string> = {
        GRUB_DISABLE_SUBMENU: 'n',
        GRUB_SAVEDEFAULT: 'false',
        GRUB_TIMEOUT: '5',
        GRUB_DEFAULT: '0',
      };
      if (key === 'GRUB_CMDLINE_LINUX_DEFAULT') {
        const norm = (s: string) => (s || '').split(/\s+/).filter(Boolean).sort().join(' ');
        if (norm(current || '') === norm(value)) {
          nm.delete(key);
          return nm;
        }
      }
      if (current === undefined && (defaults[key] === value || value === '')) {
        nm.delete(key);
        return nm;
      }
      if (key === 'GRUB_SAVEDEFAULT' && value === 'false') {
        const def = nm.get('GRUB_DEFAULT') || this.currentGrub().get('GRUB_DEFAULT');
        if (def !== 'saved' && current === undefined) {
          nm.delete(key);
          return nm;
        }
      }
      if (current === value) {
        nm.delete(key);
        return nm;
      }
      nm.set(key, value);
      return nm;
    });
  }

  setWantedPlymouthTheme(theme: string | null): void {
    const current = this.currentPlymouthTheme();
    const nT = theme === '' ? null : theme;
    const nC = current === '' ? null : current;
    this.wantedPlymouthTheme.set(nC === nT ? null : nT);
  }

  setWantedBootScript(script: string | null): void {
    this.wantedBootScript.set(script);
  }

  /**
   * Set a package state directly.
   * @param pkg The package to toggle.
   * @param state The new package state.
   */
  setPackage(pkg: string, state: boolean): void {
    const arrow = (wanted: Map<string, boolean>) => {
      const newMap = new Map<string, boolean>(wanted);
      newMap.set(pkg, state);
      return this.wantedPrune(newMap, this.installedPackages());
    };
    this.wantedPackages.update(arrow);
  }

  /**
   * Toggle a package.
   * @param pkg The package to toggle.
   * @param remove Whether to remove the package.
   */
  togglePackage(pkg: string, remove = false): void {
    const arrow = (wanted: Map<string, boolean>) => {
      if (remove) {
        if (wanted.has(pkg)) {
          const newMap = new Map<string, boolean>(wanted);
          newMap.delete(pkg);
          return newMap;
        }
        return wanted;
      }
      const newMap = new Map<string, boolean>(wanted);
      newMap.set(pkg, this.packages().has(pkg) ? !this.packages().get(pkg) : true);
      return this.wantedPrune(newMap, this.installedPackages());
    };
    this.wantedPackages.update(arrow);
  }

  /**
   * Toggle a system service.
   * @param service The service to toggle.
   * @param remove Whether to remove the service.
   */
  toggleService(service: string, remove = false): void {
    this.wantedServices.update((wanted) => {
      // If already in the map, remove it
      if (wanted.has(service)) {
        const newMap = new Map<string, boolean>(wanted);
        newMap.delete(service);
        return newMap;
      } else if (!remove) {
        // Otherwise, add it
        const newMap = new Map<string, boolean>(wanted);
        newMap.set(service, this.services().has(service) ? !this.services().get(service) : true);
        return newMap;
      }
      return wanted;
    });
  }

  /**
   * Toggle a service for the current user.
   * @param service The service to toggle.
   * @param remove Whether to remove the service.
   */
  toggleServiceUser(service: string, remove = false): void {
    this.wantedServicesUser.update((wanted) => {
      // If already in the map, remove it
      if (wanted.has(service)) {
        const newMap = new Map<string, boolean>(wanted);
        newMap.delete(service);
        return newMap;
      } else if (!remove) {
        // Otherwise, add it
        const newMap = new Map<string, boolean>(wanted);
        newMap.set(service, this.servicesUser().has(service) ? !this.servicesUser().get(service) : true);
        return newMap;
      }
      return wanted;
    });
  }

  /**
   * Toggle a group.
   * @param group The group to toggle.
   * @param remove Whether to remove the group.
   * @private
   */
  toggleGroup(group: string, remove = false): void {
    this.wantedGroups.update((wanted) => {
      // If already in the map, remove it
      if (wanted.has(group)) {
        const newMap = new Map<string, boolean>(wanted);
        newMap.delete(group);
        return newMap;
      } else if (!remove) {
        // Otherwise, add it
        const newMap = new Map<string, boolean>(wanted);
        newMap.set(group, !this.groups().has(group));
        return newMap;
      }
      return wanted;
    });
  }

  /**
   * Toggle a system tool entry.
   * @param name The name of the entry to toggle.
   * @param type The type of the entry (pkg, service, serviceUser, group, config).
   * @param remove Whether to remove the entry.
   */
  toggle(name: string, type: string, remove = false): void {
    switch (type) {
      case 'pkg':
        this.togglePackage(name, remove);
        break;
      case 'service':
        this.toggleService(name, remove);
        break;
      case 'serviceUser':
        this.toggleServiceUser(name, remove);
        break;
      case 'group':
        this.toggleGroup(name, remove);
        break;
      case 'config':
        this.toggleConfigState(name);
        break;
    }
  }

  /**
   * Check if a package is considered to be "active" for the purpose of the application.
   * This does not necessarily mean that the package is installed, but may mean that the package is meant for installation.
   * When current is set to true, it will check if the package is currently actually installed. Usage of this should be avoided.
   * Think twice, why would you want to check if a package is actually installed? If a package is active, it will be installed if it is not installed anyway.
   * @param name The name of the package/service/group to check.
   * @param type The type of the entry to check (pkg, service, serviceUser, group).
   * @param current Whether to check if the package is currently installed. (Do not use this unless you have a very good reason to do so)
   * @returns Whether the package is active/installed as a boolean.
   */
  check(name: string, type: string, current = false): boolean {
    switch (type) {
      case 'pkg':
        return current ? this.installedPackages().get(name) == true : this.packages().get(name) == true;
      case 'service':
        return current ? this.currentServices().get(name) == true : this.services().get(name) == true;
      case 'serviceUser':
        return current ? this.currentServicesUser().get(name) == true : this.servicesUser().get(name) == true;
      case 'group':
        return current ? this.currentGroups().get(name) == true : this.groups().get(name) == true;
      case 'config':
        return current ? this.currentConfigs().get(name) == true : this.configs().get(name) == true;
    }
    return false;
  }

  /**
   * Check if a package is installed already.
   * @param pkg The name of the package to check.
   * @returns Whether the package is installed as a boolean.
   * @private
   */
  private async isPackageInstalledArchlinux(pkg: string): Promise<boolean> {
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(`pacman -Qq ${pkg}`);
    return result.code === 0;
  }

  /**
   * Ensure that a package/command is installed
   * @param pkg The name of the package that will be installed if the executable is not found.
   * @returns Whether the package is installed as a boolean.
   */
  async ensurePackageArchlinux(pkg: string): Promise<boolean> {
    if (this.packages().get(pkg) !== true) {
      const task: Task = this.taskManagerService.createTask(
        0,
        'install-' + pkg,
        true,
        `Install ${pkg}`,
        'pi pi-box',
        `pacman -S --noconfirm --needed ${pkg}`,
      );
      await this.taskManagerService.executeTask(task);
      return await this.isPackageInstalledArchlinux(pkg);
    }
    return true;
  }

  /**
   * Get the available language packs from the system and process them.
   * @returns A promise that resolves when the language packs are processed
   */
  private async getLocales(): Promise<Map<string, boolean>> {
    const cmd = 'localectl list-locales';
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);
    if (result.code !== 0) {
      return new Map<string, boolean>();
    }

    return result.stdout
      .trim()
      .split('\n')
      .reduce((map: any, locale: any) => {
        map.set(locale, true);
        return map;
      }, new Map<string, boolean>());
  }

  /**
   * Toggle a locale.
   * @param locale The locale to toggle.
   * @param remove Whether to remove the locale.
   */
  toggleLocale(locale: string, remove = false): void {
    this.wantedLocales.update((wanted) => {
      if (wanted.has(locale)) {
        const newMap = new Map<string, boolean>(wanted);
        newMap.delete(locale);
        return newMap;
      } else if (!remove) {
        const newMap = new Map<string, boolean>(wanted);
        newMap.set(locale, this.locales().has(locale) ? !this.locales().get(locale) : true);
        return newMap;
      }
      return wanted;
    });
  }
}
