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
import { ElectronFsService, ElectronShellService } from '../../electron-services';

interface ConfigFile {
  name: string;
  path: string;
  description: string;
  category: string;
  wikiLink?: string;
  alwaysShow?: boolean;
}

@Component({
  selector: 'toolbox-config-files',
  imports: [TableModule, TranslocoDirective, Button, IconFieldModule, InputIconModule, InputTextModule, TooltipModule],
  templateUrl: './config-files.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ConfigFilesComponent implements OnInit {
  private readonly taskManager = inject(TaskManagerService);
  private readonly configService = inject(ConfigService);
  private readonly electronFs = inject(ElectronFsService);
  protected readonly shellService = inject(ElectronShellService);

  readonly configFiles = signal<ConfigFile[]>([]);

  private readonly ALL_CONFIG_FILES: ConfigFile[] = [
    {
      name: 'Pacman Config',
      path: '/etc/pacman.conf',
      description: 'configFiles.desc.pacmanConf',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Pacman',
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
      name: 'Chaotic Mirrorlist',
      path: '/etc/pacman.d/chaotic-mirrorlist',
      description: 'configFiles.desc.chaoticMirrorlist',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Unofficial_user_repositories#chaotic-aur',
    },
    {
      name: 'Fstab',
      path: '/etc/fstab',
      description: 'configFiles.desc.fstab',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Fstab',
    },
    {
      name: 'Environment',
      path: '/etc/environment',
      description: 'configFiles.desc.environment',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Environment_variables',
    },
    {
      name: 'GRUB Default',
      path: '/etc/default/grub',
      description: 'configFiles.desc.grub',
      category: 'Bootloader',
      wikiLink: 'https://wiki.archlinux.org/title/GRUB',
    },
    {
      name: 'VConsole Config',
      path: '/etc/vconsole.conf',
      description: 'configFiles.desc.vconsole',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Linux_console',
    },
    {
      name: 'Locale Config',
      path: '/etc/locale.conf',
      description: 'configFiles.desc.locale',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Locale',
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
      name: 'Bash',
      path: '~/.bashrc',
      description: 'configFiles.desc.bashrc',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Bash',
    },
    {
      name: 'Fish',
      path: '~/.config/fish/config.fish',
      description: 'configFiles.desc.fish',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Fish',
    },
    {
      name: 'ZSH',
      path: '~/.zshrc',
      description: 'configFiles.desc.zsh',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Zsh',
    },
    {
      name: 'Starship',
      path: '~/.config/starship.toml',
      description: 'configFiles.desc.starship',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Bash_prompt#Starship',
    },
    {
      name: 'Alacritty',
      path: '~/.config/alacritty/alacritty.toml',
      description: 'configFiles.desc.alacritty',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Alacritty',
    },
    {
      name: 'Kitty',
      path: '~/.config/kitty/kitty.conf',
      description: 'configFiles.desc.kitty',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Kitty',
    },
    {
      name: 'Fastfetch',
      path: '~/.config/fastfetch/config.jsonc',
      description: 'configFiles.desc.fastfetch',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Fastfetch',
    },

    {
      name: 'Mkinitcpio Config',
      path: '/etc/mkinitcpio.conf',
      description: 'configFiles.desc.mkinitcpio',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Mkinitcpio',
    },
    {
      name: 'Logind Config',
      path: '/etc/systemd/logind.conf',
      description: 'configFiles.desc.logind',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Power_management#ACPI_events',
    },
    {
      name: 'Journald Config',
      path: '/etc/systemd/journald.conf',
      description: 'configFiles.desc.journald',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd/Journal',
    },
    {
      name: 'NetworkManager Config',
      path: '/etc/NetworkManager/NetworkManager.conf',
      description: 'configFiles.desc.networkmanager',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/NetworkManager',
    },
    {
      name: 'Resolv Config',
      path: '/etc/resolv.conf',
      description: 'configFiles.desc.resolv',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/Domain_name_resolution',
    },
    {
      name: 'SSHD Config',
      path: '/etc/ssh/sshd_config',
      description: 'configFiles.desc.sshd',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/OpenSSH',
    },
    {
      name: 'SSH Config',
      path: '~/.ssh/config',
      description: 'configFiles.desc.ssh_user',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/OpenSSH#Client_usage',
    },
    {
      name: 'Profile',
      path: '~/.profile',
      description: 'configFiles.desc.profile',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Environment_variables',
    },
    {
      name: 'User Dirs',
      path: '~/.config/user-dirs.dirs',
      description: 'configFiles.desc.userdirs',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/XDG_user_directories',
    },
    {
      name: 'Git Config',
      path: '~/.gitconfig',
      description: 'configFiles.desc.gitconfig',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Git',
    },
    {
      name: 'Hyprland',
      path: '~/.config/hypr/hyprland.conf',
      description: 'configFiles.desc.hyprland',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Hyprland',
    },
    {
      name: 'Sway',
      path: '~/.config/sway/config',
      description: 'configFiles.desc.sway',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Sway',
    },
    {
      name: 'Wayfire',
      path: '~/.config/wayfire.ini',
      description: 'configFiles.desc.wayfire',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Wayfire',
    },
    {
      name: 'Qtile',
      path: '~/.config/qtile/config.py',
      description: 'configFiles.desc.qtile',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/Qtile',
    },
    {
      name: 'i3',
      path: '~/.config/i3/config',
      description: 'configFiles.desc.i3',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/I3',
    },
    {
      name: 'SDDM Config',
      path: '/etc/sddm.conf',
      description: 'configFiles.desc.sddm',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/SDDM',
    },
    {
      name: 'OS Release',
      path: '/etc/os-release',
      description: 'configFiles.desc.os_release',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Os-release',
    },
    {
      name: '20-gpu.conf',
      path: '/etc/X11/xorg.conf.d/20-gpu.conf',
      description: 'configFiles.desc.xorg_gpu',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/Xorg#Configuration',
    },
    {
      name: 'Nvidia Config',
      path: '/etc/modprobe.d/nvidia.conf',
      description: 'configFiles.desc.nvidia_conf',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/NVIDIA',
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
      name: 'PipeWire Config',
      path: '~/.config/pipewire/pipewire.conf',
      description: 'configFiles.desc.pipewire_conf',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/PipeWire',
    },
    {
      name: 'client.conf (PipeWire)',
      path: '~/.config/pipewire/client.conf',
      description: 'configFiles.desc.pipewire_client',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/PipeWire',
    },
    {
      name: 'WirePlumber',
      path: '~/.config/wireplumber',
      description: 'configFiles.desc.wireplumber',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/WirePlumber',
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
      name: 'ALSA Base',
      path: '/etc/modprobe.d/alsa-base.conf',
      description: 'configFiles.desc.alsa_base',
      category: 'Audio',
      wikiLink: 'https://wiki.archlinux.org/title/Advanced_Linux_Sound_Architecture',
    },
    {
      name: 'GRUB Config',
      path: '/boot/grub/grub.cfg',
      description: 'configFiles.desc.grub_cfg',
      category: 'Bootloader',
      wikiLink: 'https://wiki.archlinux.org/title/GRUB',
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
      name: 'Group',
      path: '/etc/group',
      description: 'configFiles.desc.group',
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
      name: 'Locale Gen',
      path: '/etc/locale.gen',
      description: 'configFiles.desc.locale_gen',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Locale',
    },
    {
      name: 'Timezone',
      path: '/etc/timezone',
      description: 'configFiles.desc.timezone',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/System_time',
    },
    {
      name: 'Adjtime',
      path: '/etc/adjtime',
      description: 'configFiles.desc.adjtime',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/System_time',
    },
    {
      name: 'Rsyslog Config',
      path: '/etc/rsyslog.conf',
      description: 'configFiles.desc.rsyslog',
      category: 'Logging',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd/Journal',
    },
    {
      name: 'Xorg Config',
      path: '/etc/X11/xorg.conf',
      description: 'configFiles.desc.xorg_conf',
      category: 'Graphics and Display',
      wikiLink: 'https://wiki.archlinux.org/title/Xorg#Configuration',
    },
    {
      name: 'Bash System Config',
      path: '/etc/bash.bashrc',
      description: 'configFiles.desc.bash_bashrc',
      category: 'User Environment',
      wikiLink: 'https://wiki.archlinux.org/title/Bash',
    },
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
      name: 'Sysctl Config',
      path: '/etc/sysctl.conf',
      description: 'configFiles.desc.sysctl',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Sysctl',
    },
  ];

  async ngOnInit() {
    const userHome = this.configService.state().userHome;
    const availableFiles: ConfigFile[] = [];

    for (const file of this.ALL_CONFIG_FILES) {
      let fullPath = file.path;
      if (fullPath.startsWith('~/')) {
        fullPath = fullPath.replace('~', userHome);
      }

      if (file.alwaysShow || (await this.electronFs.exists(fullPath, true))) {
        availableFiles.push(file);
      }
    }

    this.configFiles.set(availableFiles);
  }

  async editConfig(file: ConfigFile) {
    const userHome = this.configService.state().userHome;
    let fullPath = file.path;

    if (fullPath.startsWith('~/')) {
      fullPath = fullPath.replace('~', userHome);
    }

    const needsRoot = !fullPath.startsWith(userHome);
    if (needsRoot) {
      const cmd = `pkexec micro "${fullPath}"`;
      await this.taskManager.executeAndWaitBashTerminal(cmd);
      return;
    }

    let editor = 'micro';
    let isVisual = false;

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
        if (line.startsWith('VISUAL=')) {
          envVars.set('VISUAL', line.substring(7).trim());
        } else if (line.startsWith('EDITOR=')) {
          envVars.set('EDITOR', line.substring(7).trim());
        }
      }

      if (envVars.has('VISUAL') && envVars.get('VISUAL')) {
        editor = envVars.get('VISUAL') as string;
        isVisual = true;
      } else if (envVars.has('EDITOR') && envVars.get('EDITOR')) {
        editor = envVars.get('EDITOR') as string;
      }
    }

    const cmd = `${editor} "${fullPath}"`;

    if (isVisual) {
      await this.taskManager.executeAndWaitBash(cmd);
    } else {
      await this.taskManager.executeAndWaitBashTerminal(cmd);
    }
  }
}
