import { ChangeDetectionStrategy, Component, inject, type OnInit, signal } from '@angular/core';
import { TableModule } from 'primeng/table';
import { TranslocoDirective } from '@jsverse/transloco';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { ConfigService } from '../config/config.service';
import { Button } from 'primeng/button';
import { IconFieldModule } from 'primeng/iconfield';
import { InputIconModule } from 'primeng/inputicon';
import { InputTextModule } from 'primeng/inputtext';
import { TooltipModule } from 'primeng/tooltip';
import { DialogModule } from 'primeng/dialog';
import { ElectronFsService, ElectronShellService } from '../../electron-services';

interface ConfigFile {
  name: string;
  path: string;
  description: string;
  category: string;
  wikiLink?: string;
  alwaysShow?: boolean;
  directory?: boolean;
}

@Component({
  selector: 'toolbox-config-files',
  imports: [
    TableModule,
    TranslocoDirective,
    Button,
    IconFieldModule,
    InputIconModule,
    InputTextModule,
    TooltipModule,
    DialogModule,
  ],
  templateUrl: './config-files.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ConfigFilesComponent implements OnInit {
  private readonly taskManager = inject(TaskManagerService);
  private readonly configService = inject(ConfigService);
  private readonly electronFs = inject(ElectronFsService);
  protected readonly shellService = inject(ElectronShellService);

  readonly configFiles = signal<ConfigFile[]>([]);
  readonly directoryDialogVisible = signal(false);
  readonly selectedDirectory = signal<ConfigFile | null>(null);
  readonly directoryFiles = signal<ConfigFile[]>([]);

  private sortConfigFiles(files: ConfigFile[]): ConfigFile[] {
    const categoryOrder = new Map<string, number>();
    for (const file of this.ALL_CONFIG_FILES) {
      if (!categoryOrder.has(file.category)) {
        categoryOrder.set(file.category, categoryOrder.size);
      }
    }

    return [...files].sort((a, b) => {
      if (a.category !== b.category) {
        return (
          (categoryOrder.get(a.category) ?? Number.MAX_SAFE_INTEGER) -
          (categoryOrder.get(b.category) ?? Number.MAX_SAFE_INTEGER)
        );
      }

      return a.name.localeCompare(b.name);
    });
  }

  private resolvePath(path: string): string {
    const userHome = this.configService.state().userHome;
    if (path.startsWith('~/')) {
      return path.replace('~', userHome);
    }
    return path;
  }

  private quoteForShell(value: string): string {
    return `'${value.replaceAll("'", `'"'"'`)}'`;
  }

  private sanitizeEditor(editor: string): string {
    return editor.replace(/^["']+|["']+$/g, '').trim();
  }

  private async listDirectoryFiles(parent: ConfigFile): Promise<ConfigFile[]> {
    const fullPath = this.resolvePath(parent.path);
    const cmd =
      `if [ -d ${this.quoteForShell(fullPath)} ]; then ` +
      `find ${this.quoteForShell(fullPath)} -mindepth 1 -maxdepth 1 -type f -printf '%f\\n' | sort; fi`;
    const res = await this.taskManager.executeAndWaitBash(cmd);
    if (res.code !== 0) {
      return [];
    }

    return res.stdout
      .split('\n')
      .map((line) => line.trim())
      .filter((line) => line.length > 0)
      .map((name) => ({
        name,
        path: `${parent.path.replace(/\/$/, '')}/${name}`,
        description: parent.description,
        category: parent.category,
        wikiLink: parent.wikiLink,
      }));
  }

  private readonly ALL_CONFIG_FILES: ConfigFile[] = [
    /* Audio */
    {
      name: 'ALSA Base',
      path: '/etc/modprobe.d/alsa-base.conf',
      description: 'configFiles.desc.alsa_base',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/Advanced_Linux_Sound_Architecture',
    },
    {
      name: 'client.conf (PipeWire)',
      path: '~/.config/pipewire/client.conf',
      description: 'configFiles.desc.pipewire_client',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/PipeWire',
    },
    {
      name: 'daemon.conf (Pulse)',
      path: '~/.config/pulse/daemon.conf',
      description: 'configFiles.desc.pulse_daemon',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/PulseAudio',
    },
    {
      name: 'default.pa (Pulse)',
      path: '~/.config/pulse/default.pa',
      description: 'configFiles.desc.pulse_default',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/PulseAudio',
    },
    {
      name: 'main.conf (Bluetooth)',
      path: '/etc/bluetooth/main.conf',
      description: 'configFiles.desc.bluetooth_main',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/Bluetooth',
    },
    {
      name: 'PipeWire Config',
      path: '~/.config/pipewire/pipewire.conf',
      description: 'configFiles.desc.pipewire_conf',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/PipeWire',
    },
    {
      name: 'System ALSA',
      path: '/etc/asound.conf',
      description: 'configFiles.desc.asound_conf',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/Advanced_Linux_Sound_Architecture',
    },
    {
      name: 'User ALSA',
      path: '~/.asoundrc',
      description: 'configFiles.desc.asoundrc',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/Advanced_Linux_Sound_Architecture',
    },
    {
      name: 'WirePlumber',
      path: '~/.config/wireplumber',
      description: 'configFiles.desc.wireplumber',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/WirePlumber',
    },

    /* Bootloader */
    {
      name: 'GRUB Config',
      path: '/boot/grub/grub.cfg',
      description: 'configFiles.desc.grub_cfg',
      category: 'Bootloader',
      wikiLink: 'https://wiki.archlinux.org/title/GRUB',
    },
    {
      name: 'GRUB Default',
      path: '/etc/default/grub',
      description: 'configFiles.desc.grub',
      category: 'Bootloader',
      wikiLink: 'https://wiki.archlinux.org/title/GRUB',
    },

    /* Display & Window Managers */
    {
      name: 'GDM Config',
      path: '/etc/gdm/custom.conf',
      description: 'configFiles.desc.gdm',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/GDM',
    },
    {
      name: 'GNOME Settings (dconf)',
      path: '~/.config/dconf/user',
      description: 'configFiles.desc.dconfUser',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/GNOME#Configuration',
    },
    {
      name: 'Hyprland',
      path: '~/.config/hypr/hyprland.conf',
      description: 'configFiles.desc.hyprland',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Hyprland',
    },
    {
      name: 'Hyprpaper',
      path: '~/.config/hypr/hyprpaper.conf',
      description: 'configFiles.desc.hyprpaper',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.hyprland.org/Hypr-Ecosystem/hyprpaper/',
    },
    {
      name: 'i3',
      path: '~/.config/i3/config',
      description: 'configFiles.desc.i3',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/I3',
    },
    {
      name: 'KDE Globals',
      path: '~/.config/kdeglobals',
      description: 'configFiles.desc.kdeglobals',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/KDE',
    },
    {
      name: 'KWin Config',
      path: '~/.config/kwinrc',
      description: 'configFiles.desc.kwinrc',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/KWin',
    },
    {
      name: 'LightDM Config',
      path: '/etc/lightdm/lightdm.conf',
      description: 'configFiles.desc.lightdm',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/LightDM',
    },
    {
      name: 'Plasma Desktop Applets',
      path: '~/.config/plasma-org.kde.plasma.desktop-appletsrc',
      description: 'configFiles.desc.plasmaApplets',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/KDE',
    },
    {
      name: 'Plasma Workspace',
      path: '~/.config/plasmarc',
      description: 'configFiles.desc.plasmarc',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/KDE',
    },
    {
      name: 'Qtile',
      path: '~/.config/qtile/config.py',
      description: 'configFiles.desc.qtile',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Qtile',
    },
    {
      name: 'SDDM Config',
      path: '/etc/sddm.conf',
      description: 'configFiles.desc.sddm',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/SDDM',
    },
    {
      name: 'Sway',
      path: '~/.config/sway/config',
      description: 'configFiles.desc.sway',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Sway',
    },
    {
      name: 'Waybar Config',
      path: '~/.config/waybar/config',
      description: 'configFiles.desc.waybar',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Waybar',
    },
    {
      name: 'Waybar Style',
      path: '~/.config/waybar/style.css',
      description: 'configFiles.desc.waybarStyle',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Waybar',
    },
    {
      name: 'Wayfire',
      path: '~/.config/wayfire.ini',
      description: 'configFiles.desc.wayfire',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Wayfire',
    },

    /* Gaming */
    {
      name: 'GameMode Config',
      path: '/etc/gamemode.ini',
      description: 'configFiles.desc.gamemode',
      category: 'Gaming',
      wikiLink: 'https://wiki.archlinux.org/title/Gamemode',
    },
    {
      name: 'MangoHud Config',
      path: '~/.config/MangoHud/MangoHud.conf',
      description: 'configFiles.desc.mangohud',
      category: 'Gaming',
      wikiLink: 'https://wiki.archlinux.org/title/MangoHud',
    },

    /* Garuda Tools */
    {
      name: 'BTRFS Assistant Config',
      path: '/etc/btrfs-assistant.conf',
      description: 'configFiles.desc.btrfsAssistant',
      category: 'Garuda Tools',
      wikiLink: 'https://gitlab.com/garuda-linux/applications/btrfs-assistant',
    },
    {
      name: 'cpupower Config',
      path: '/etc/default/cpupower',
      description: 'configFiles.desc.cpupower',
      category: 'Garuda Tools',
      wikiLink: 'https://wiki.archlinux.org/title/CPU_frequency_scaling',
    },
    {
      name: 'Garuda Update Config',
      path: '/etc/garuda/garuda-update/config',
      description: 'configFiles.desc.garudaUpdateConfig',
      category: 'Garuda Tools',
      wikiLink: 'https://wiki.garudalinux.org/garuda-update',
    },
    {
      name: 'Snapper Root Config',
      path: '/etc/snapper/configs/root',
      description: 'configFiles.desc.snapperRoot',
      category: 'Garuda Tools',
      wikiLink: 'https://wiki.archlinux.org/title/Snapper',
    },
    {
      name: 'ZRAM generator',
      path: '/etc/systemd/zram-generator.conf',
      description: 'configFiles.desc.zramGenerator',
      category: 'Garuda Tools',
      wikiLink: 'https://wiki.archlinux.org/title/Zram',
    },

    /* Graphics and Display */
    {
      name: '20-gpu.conf',
      path: '/etc/X11/xorg.conf.d/20-gpu.conf',
      description: 'configFiles.desc.xorg_gpu',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/Xorg#Configuration',
    },
    {
      name: 'AMDGPU Config',
      path: '/etc/modprobe.d/amdgpu.conf',
      description: 'configFiles.desc.amdgpu_conf',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/AMDGPU',
    },
    {
      name: 'Blacklist Nouveau',
      path: '/etc/modprobe.d/blacklist-nouveau.conf',
      description: 'configFiles.desc.blacklist_nouveau',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/Nouveau',
    },
    {
      name: 'Monitor Layout',
      path: '/etc/X11/xorg.conf.d/10-monitor.conf',
      description: 'configFiles.desc.xorg_monitor',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/Multihead',
    },
    {
      name: 'Nvidia Config',
      path: '/etc/modprobe.d/nvidia.conf',
      description: 'configFiles.desc.nvidia_conf',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/NVIDIA',
    },
    {
      name: 'Xorg Config',
      path: '/etc/X11/xorg.conf',
      description: 'configFiles.desc.xorg_conf',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/Xorg#Configuration',
    },
    {
      name: 'Xorg Config',
      path: '/etc/X11/xorg.conf.d',
      description: 'configFiles.desc.xorg_conf_d',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/Xorg#Configuration',
      directory: true,
    },

    /* Logging */
    {
      name: 'Rsyslog Config',
      path: '/etc/rsyslog.conf',
      description: 'configFiles.desc.rsyslog',
      category: 'Logging',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd/Journal',
    },

    /* Network */
    {
      name: 'Hosts Allow',
      path: '/etc/hosts.allow',
      description: 'configFiles.desc.hosts_allow',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/Tcpwrappers',
    },
    {
      name: 'Hosts Deny',
      path: '/etc/hosts.deny',
      description: 'configFiles.desc.hosts_deny',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/Tcpwrappers',
    },
    {
      name: 'NetworkManager Config',
      path: '/etc/NetworkManager/NetworkManager.conf',
      description: 'configFiles.desc.networkmanager',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/NetworkManager',
    },
    {
      name: 'systemd-networkd',
      path: '/etc/systemd/network',
      description: 'configFiles.desc.systemd_network',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd-networkd',
      directory: true,
    },
    {
      name: 'nftables Config',
      path: '/etc/nftables.conf',
      description: 'configFiles.desc.nftables',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/Nftables',
    },
    {
      name: 'Resolv Config',
      path: '/etc/resolv.conf',
      description: 'configFiles.desc.resolv',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/Domain_name_resolution',
    },
    {
      name: 'SSH Config',
      path: '~/.ssh/config',
      description: 'configFiles.desc.ssh_user',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/OpenSSH#Client_usage',
    },
    {
      name: 'SSHD Config',
      path: '/etc/ssh/sshd_config',
      description: 'configFiles.desc.sshd',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/OpenSSH',
    },

    /* Package Management */
    {
      name: 'Chaotic Mirrorlist',
      path: '/etc/pacman.d/chaotic-mirrorlist',
      description: 'configFiles.desc.chaoticMirrorlist',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Unofficial_user_repositories#chaotic-aur',
    },
    {
      name: 'Makepkg Config',
      path: '/etc/makepkg.conf',
      description: 'configFiles.desc.makepkgConf',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Makepkg',
    },
    {
      name: 'Mirrorlist',
      path: '/etc/pacman.d/mirrorlist',
      description: 'configFiles.desc.mirrorlist',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Mirrors',
    },
    {
      name: 'Pacman Hooks',
      path: '/etc/pacman.d/hooks',
      description: 'configFiles.desc.pacman_hooks',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Pacman#Hooks',
      directory: true,
    },
    {
      name: 'Pacman Config',
      path: '/etc/pacman.conf',
      description: 'configFiles.desc.pacmanConf',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Pacman',
    },

    /* System */
    {
      name: 'Adjtime',
      path: '/etc/adjtime',
      description: 'configFiles.desc.adjtime',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/System_time',
    },
    {
      name: 'Environment',
      path: '/etc/environment',
      description: 'configFiles.desc.environment',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Environment_variables',
    },
    {
      name: 'Fstab',
      path: '/etc/fstab',
      description: 'configFiles.desc.fstab',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Fstab',
    },
    {
      name: 'Hostname',
      path: '/etc/hostname',
      description: 'configFiles.desc.hostname',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Network_configuration#Set_the_hostname',
    },
    {
      name: 'Hosts',
      path: '/etc/hosts',
      description: 'configFiles.desc.hosts',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Network_configuration#Local_network_hostname_resolution',
    },
    {
      name: 'Journald Config',
      path: '/etc/systemd/journald.conf',
      description: 'configFiles.desc.journald',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd/Journal',
    },
    {
      name: 'Locale Config',
      path: '/etc/locale.conf',
      description: 'configFiles.desc.locale',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Locale',
    },
    {
      name: 'Locale Gen',
      path: '/etc/locale.gen',
      description: 'configFiles.desc.locale_gen',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Locale',
    },
    {
      name: 'Logind Config',
      path: '/etc/systemd/logind.conf',
      description: 'configFiles.desc.logind',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Power_management#ACPI_events',
    },
    {
      name: 'Mkinitcpio Config',
      path: '/etc/mkinitcpio.conf',
      description: 'configFiles.desc.mkinitcpio',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Mkinitcpio',
    },
    {
      name: 'OS Release',
      path: '/etc/os-release',
      description: 'configFiles.desc.os_release',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Os-release',
    },
    {
      name: 'Sysctl Config',
      path: '/etc/sysctl.conf',
      description: 'configFiles.desc.sysctl',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Sysctl',
    },
    {
      name: 'Systemd System Config',
      path: '/etc/systemd/system.conf',
      description: 'configFiles.desc.systemdSystem',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd',
    },
    {
      name: 'Systemd User Config',
      path: '/etc/systemd/user.conf',
      description: 'configFiles.desc.systemdUser',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd/User',
    },
    {
      name: 'Systemd Services',
      path: '/etc/systemd/system',
      description: 'configFiles.desc.systemd_services',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd',
      directory: true,
    },
    {
      name: 'Systemd User Units',
      path: '/etc/systemd/user',
      description: 'configFiles.desc.systemd_services',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd/User',
      directory: true,
    },
    {
      name: 'Timezone',
      path: '/etc/timezone',
      description: 'configFiles.desc.timezone',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/System_time',
    },
    {
      name: 'VConsole Config',
      path: '/etc/vconsole.conf',
      description: 'configFiles.desc.vconsole',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Linux_console',
    },
    {
      name: 'modprobe.d',
      path: '/etc/modprobe.d',
      description: 'configFiles.desc.modprobe_d',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Kernel_module',
      directory: true,
    },
    {
      name: 'udev Rules',
      path: '/etc/udev/rules.d',
      description: 'configFiles.desc.udev_rules',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Udev',
      directory: true,
    },

    /* Theming */
    {
      name: 'Cursor Theme (System)',
      path: '/usr/share/icons/default/index.theme',
      description: 'configFiles.desc.cursorThemeSystem',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/Cursor_themes',
    },
    {
      name: 'Cursor Theme (User)',
      path: '~/.icons/default/index.theme',
      description: 'configFiles.desc.cursorThemeUser',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/Cursor_themes',
    },
    {
      name: 'Fontconfig System',
      path: '/etc/fonts/local.conf',
      description: 'configFiles.desc.fontconfigSystem',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/Font_configuration',
    },
    {
      name: 'Fontconfig User',
      path: '~/.config/fontconfig/fonts.conf',
      description: 'configFiles.desc.fontconfigUser',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/Font_configuration',
    },
    {
      name: 'GRUB Theme',
      path: '/usr/share/grub/themes/garuda-dr460nized/theme.txt',
      description: 'configFiles.desc.grubTheme',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/GRUB/Tips_and_tricks#Visual_configuration',
    },
    {
      name: 'GTK 2 Config',
      path: '~/.gtkrc-2.0',
      description: 'configFiles.desc.gtkrc2',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/GTK#GTK_2',
    },
    {
      name: 'GTK 3 Settings',
      path: '~/.config/gtk-3.0/settings.ini',
      description: 'configFiles.desc.gtk3Settings',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/GTK#Basic_theme_configuration',
    },
    {
      name: 'GTK 4 Custom CSS',
      path: '~/.config/gtk-4.0/gtk.css',
      description: 'configFiles.desc.gtk4Css',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/GTK#Basic_theme_configuration',
    },
    {
      name: 'GTK 4 Settings',
      path: '~/.config/gtk-4.0/settings.ini',
      description: 'configFiles.desc.gtk4Settings',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/GTK#Basic_theme_configuration',
    },
    {
      name: 'Icon Theme Config',
      path: '~/.config/kdeglobals',
      description: 'configFiles.desc.iconTheme',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/Icons',
    },
    {
      name: 'KDE Font Config',
      path: '~/.config/kcmfonts',
      description: 'configFiles.desc.kcmfonts',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/Fonts#Fontconfig',
    },
    {
      name: 'KDE Look and Feel',
      path: '~/.config/kdedefaults/package',
      description: 'configFiles.desc.kdeLookandFeel',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/KDE#Themes',
    },
    {
      name: 'Kvantum Config',
      path: '~/.config/Kvantum/kvantum.kvconfig',
      description: 'configFiles.desc.kvantumConfig',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/Kvantum',
    },
    {
      name: 'Plymouth Config',
      path: '/etc/plymouth/plymouthd.conf',
      description: 'configFiles.desc.plymouthConf',
      category: 'Theming',
      wikiLink: 'https://wiki.archlinux.org/title/Plymouth',
    },

    /* User Environment */
    {
      name: 'Alacritty',
      path: '~/.config/alacritty/alacritty.toml',
      description: 'configFiles.desc.alacritty',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Alacritty',
    },
    {
      name: 'Bash',
      path: '~/.bashrc',
      description: 'configFiles.desc.bashrc',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Bash',
    },
    {
      name: 'Bash System Config',
      path: '/etc/bash.bashrc',
      description: 'configFiles.desc.bash_bashrc',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Bash',
    },
    {
      name: 'Dunst Config',
      path: '~/.config/dunst/dunstrc',
      description: 'configFiles.desc.dunst',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Dunst',
    },
    {
      name: 'Fastfetch',
      path: '~/.config/fastfetch/config.jsonc',
      description: 'configFiles.desc.fastfetch',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Fastfetch',
    },
    {
      name: 'Fish',
      path: '~/.config/fish/config.fish',
      description: 'configFiles.desc.fish',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Fish',
    },
    {
      name: 'Git Config',
      path: '~/.gitconfig',
      description: 'configFiles.desc.gitconfig',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Git',
    },
    {
      name: 'Kitty',
      path: '~/.config/kitty/kitty.conf',
      description: 'configFiles.desc.kitty',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Kitty',
    },
    {
      name: 'Mako Config',
      path: '~/.config/mako/config',
      description: 'configFiles.desc.mako',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Mako',
    },
    {
      name: 'Profile',
      path: '~/.profile',
      description: 'configFiles.desc.profile',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Environment_variables',
    },
    {
      name: 'Rofi Config',
      path: '~/.config/rofi/config.rasi',
      description: 'configFiles.desc.rofi',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Rofi',
    },
    {
      name: 'Starship',
      path: '~/.config/starship.toml',
      description: 'configFiles.desc.starship',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Bash_prompt#Starship',
    },
    {
      name: 'User Dirs',
      path: '~/.config/user-dirs.dirs',
      description: 'configFiles.desc.userdirs',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/XDG_user_directories',
    },
    {
      name: 'Wofi Config',
      path: '~/.config/wofi/config',
      description: 'configFiles.desc.wofi',
      category: 'User Environment',
      wikiLink: 'https://hg.sr.ht/~scoopta/wofi',
    },
    {
      name: 'Wayland Config',
      path: '~/.config/wayland',
      description: 'configFiles.desc.wayland_conf',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Wayland',
      directory: true,
    },
    {
      name: 'User systemd Units',
      path: '~/.config/systemd/user',
      description: 'configFiles.desc.systemd_services',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd/User',
      directory: true,
    },
    {
      name: 'ZSH',
      path: '~/.zshrc',
      description: 'configFiles.desc.zsh',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Zsh',
    },

    /* Users and Authentication */
    {
      name: 'Group',
      path: '/etc/group',
      description: 'configFiles.desc.group',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/Users_and_groups',
    },
    {
      name: 'Passwd',
      path: '/etc/passwd',
      description: 'configFiles.desc.passwd',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/Users_and_groups',
    },
    {
      name: 'Shadow',
      path: '/etc/shadow',
      description: 'configFiles.desc.shadow',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/Users_and_groups',
    },
    {
      name: 'Sudoers',
      path: '/etc/sudoers',
      description: 'configFiles.desc.sudoers',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/Sudo',
    },
    {
      name: 'PAM',
      path: '/etc/pam.d',
      description: 'configFiles.desc.pam_d',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/PAM',
      directory: true,
    },
    {
      name: 'cron.*',
      path: '/etc/cron.d',
      description: 'configFiles.desc.cron',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/Cron',
      directory: true,
    },
    {
      name: 'cron.daily',
      path: '/etc/cron.daily',
      description: 'configFiles.desc.cron',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/Cron',
      directory: true,
    },
    {
      name: 'cron.hourly',
      path: '/etc/cron.hourly',
      description: 'configFiles.desc.cron',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/Cron',
      directory: true,
    },
    {
      name: 'cron.monthly',
      path: '/etc/cron.monthly',
      description: 'configFiles.desc.cron',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/Cron',
      directory: true,
    },
    {
      name: 'cron.weekly',
      path: '/etc/cron.weekly',
      description: 'configFiles.desc.cron',
      category: 'Users and Authentication',
      wikiLink: 'https://wiki.archlinux.org/title/Cron',
      directory: true,
    },
  ];

  async ngOnInit() {
    const availableFiles: ConfigFile[] = [];

    for (const file of this.ALL_CONFIG_FILES) {
      const fullPath = this.resolvePath(file.path);

      if (file.alwaysShow || (await this.electronFs.exists(fullPath, true))) {
        availableFiles.push(file);
      }
    }

    this.configFiles.set(this.sortConfigFiles(availableFiles));
  }

  async editConfig(file: ConfigFile) {
    if (file.directory) {
      const files = await this.listDirectoryFiles(file);
      this.selectedDirectory.set(file);
      this.directoryFiles.set(files);
      this.directoryDialogVisible.set(true);
      return;
    }

    let editor = 'micro';
    let isVisual = false;

    const fullPath = this.resolvePath(file.path);
    const userHome = this.configService.state().userHome;
    const needsRoot = !fullPath.startsWith(userHome);

    const envScript = `
      USER_SHELL=$(getent passwd "$USER" | cut -d: -f7)
      if [ -x "$USER_SHELL" ]; then
        "$USER_SHELL" -ic 'env 2>/dev/null' | grep -E '^(VISUAL|EDITOR)=' || echo ""
      else
        env 2>/dev/null | grep -E '^(VISUAL|EDITOR)=' || echo ""
      fi
    `;

    const envRes = await this.taskManager.executeAndWaitBash(envScript);
    if (envRes && envRes.code === 0 && envRes.stdout) {
      const lines = envRes.stdout.split('\n');
      const envVars = new Map<string, string>();
      for (const line of lines) {
        const [key, value] = line.split('=');
        if (key && value) {
          envVars.set(key.trim(), value.trim());
        }
      }

      if (!needsRoot && envVars.has('VISUAL') && envVars.get('VISUAL')) {
        editor = this.sanitizeEditor(envVars.get('VISUAL') as string);
        isVisual = true;
      } else if (envVars.has('EDITOR') && envVars.get('EDITOR')) {
        editor = this.sanitizeEditor(envVars.get('EDITOR') as string);
      }
    }

    if (needsRoot) {
      const cmd = `pkexec ${editor} "${fullPath}"`;
      await this.taskManager.executeAndWaitBashTerminal(cmd);
      return;
    }

    const cmd = `${editor} "${fullPath}"`;
    if (isVisual) {
      await this.taskManager.executeAndWaitBash(cmd);
    } else {
      await this.taskManager.executeAndWaitBashTerminal(cmd);
    }
  }

  closeDirectoryDialog() {
    this.directoryDialogVisible.set(false);
    this.selectedDirectory.set(null);
    this.directoryFiles.set([]);
  }
}
