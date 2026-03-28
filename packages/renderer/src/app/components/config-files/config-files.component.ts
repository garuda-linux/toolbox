import { ChangeDetectionStrategy, Component, inject, type OnInit, signal } from '@angular/core';
import { TableModule } from 'primeng/table';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { ConfigService } from '../config/config.service';
import { Button } from 'primeng/button';
import { IconFieldModule } from 'primeng/iconfield';
import { InputIconModule } from 'primeng/inputicon';
import { InputTextModule } from 'primeng/inputtext';
import { TooltipModule } from 'primeng/tooltip';
import { ElectronFsService } from '../../electron-services/electron-fs.service';
import { ElectronShellService } from '../../electron-services/electron-shell.service';

interface ConfigFile {
  name: string;
  path: string;
  description: string;
  category: string;
  wikiLink?: string;
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
  private readonly translocoService = inject(TranslocoService);
  private readonly electronFs = inject(ElectronFsService);
  protected readonly shellService = inject(ElectronShellService);

  readonly configFiles = signal<ConfigFile[]>([]);

  private readonly ALL_CONFIG_FILES: ConfigFile[] = [
    {
      name: 'pacman.conf',
      path: '/etc/pacman.conf',
      description: 'configFiles.desc.pacmanConf',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Pacman',
    },
    {
      name: 'makepkg.conf',
      path: '/etc/makepkg.conf',
      description: 'configFiles.desc.makepkgConf',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Makepkg',
    },
    {
      name: 'mirrorlist',
      path: '/etc/pacman.d/mirrorlist',
      description: 'configFiles.desc.mirrorlist',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Mirrors',
    },
    {
      name: 'chaotic-mirrorlist',
      path: '/etc/pacman.d/chaotic-mirrorlist',
      description: 'configFiles.desc.chaoticMirrorlist',
      category: 'Package Management',
      wikiLink: 'https://wiki.archlinux.org/title/Unofficial_user_repositories#chaotic-aur',
    },
    {
      name: 'fstab',
      path: '/etc/fstab',
      description: 'configFiles.desc.fstab',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Fstab',
    },
    {
      name: 'environment',
      path: '/etc/environment',
      description: 'configFiles.desc.environment',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Environment_variables',
    },
    {
      name: 'grub',
      path: '/etc/default/grub',
      description: 'configFiles.desc.grub',
      category: 'Bootloader',
      wikiLink: 'https://wiki.archlinux.org/title/GRUB',
    },
    {
      name: 'vconsole.conf',
      path: '/etc/vconsole.conf',
      description: 'configFiles.desc.vconsole',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Linux_console',
    },
    {
      name: 'locale.conf',
      path: '/etc/locale.conf',
      description: 'configFiles.desc.locale',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Locale',
    },
    {
      name: 'hostname',
      path: '/etc/hostname',
      description: 'configFiles.desc.hostname',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Network_configuration#Set_the_hostname',
    },
    {
      name: 'hosts',
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
      name: 'mkinitcpio.conf',
      path: '/etc/mkinitcpio.conf',
      description: 'configFiles.desc.mkinitcpio',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Mkinitcpio',
    },
    {
      name: 'logind.conf',
      path: '/etc/systemd/logind.conf',
      description: 'configFiles.desc.logind',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Power_management#ACPI_events',
    },
    {
      name: 'journald.conf',
      path: '/etc/systemd/journald.conf',
      description: 'configFiles.desc.journald',
      category: 'System',
      wikiLink: 'https://wiki.archlinux.org/title/Systemd/Journal',
    },
    {
      name: 'NetworkManager.conf',
      path: '/etc/NetworkManager/NetworkManager.conf',
      description: 'configFiles.desc.networkmanager',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/NetworkManager',
    },
    {
      name: 'resolv.conf',
      path: '/etc/resolv.conf',
      description: 'configFiles.desc.resolv',
      category: 'Network',
      wikiLink: 'https://wiki.archlinux.org/title/Domain_name_resolution',
    },
    {
      name: 'sshd_config',
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
      name: 'SDDM',
      path: '/etc/sddm.conf',
      description: 'configFiles.desc.sddm',
      category: 'Display & Window Managers',
      wikiLink: 'https://wiki.archlinux.org/title/SDDM',
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

      if (await this.electronFs.exists(fullPath, true)) {
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
      // For root, we ALWAYS use micro with a terminal
      const cmd = `pkexec micro "${fullPath}"`;
      await this.taskManager.executeAndWaitBashTerminal(cmd);
      return;
    }

    let editor = 'micro';
    let isVisual = false;

    // Get VISUAL or EDITOR environment variable from the user's interactive shell
    // This allows us to read overrides from ~/.bashrc, ~/.zshrc, config.fish
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
        editor = envVars.get('VISUAL')!;
        isVisual = true;
      } else if (envVars.has('EDITOR') && envVars.get('EDITOR')) {
        editor = envVars.get('EDITOR')!;
      }
    }

    const cmd = `${editor} "${fullPath}"`;

    if (isVisual) {
      // VISUAL is set, execute the GUI editor directly (no terminal wrapper)
      await this.taskManager.executeAndWaitBash(cmd);
    } else {
      // Use the terminal for CLI editors (micro, nano, vim, etc.)
      await this.taskManager.executeAndWaitBashTerminal(cmd);
    }
  }
}
