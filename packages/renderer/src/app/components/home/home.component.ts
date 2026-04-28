import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { Card } from 'primeng/card';
import { Router, RouterLink } from '@angular/router';
import { ElectronShellService } from '../../electron-services';
import type { ExternalLink, HomepageLink } from '../../interfaces';
import { ConfigService } from '../config/config.service';
import { faBluesky, faDiscord, faDiscourse, faMastodon, faTelegram } from '@fortawesome/free-brands-svg-icons';
import { faComments } from '@fortawesome/free-solid-svg-icons';
import { NgOptimizedImage } from '@angular/common';
import { SystemStatusComponent } from '../system-status/system-status.component';
import { MessageToastService } from '@garudalinux/core';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { OsInteractService } from '../task-manager/os-interact.service';
import { FaIconComponent } from '@fortawesome/angular-fontawesome';

@Component({
  selector: 'toolbox-home',
  imports: [TranslocoDirective, Card, RouterLink, NgOptimizedImage, SystemStatusComponent, FaIconComponent],
  templateUrl: './home.component.html',
  styleUrl: './home.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class HomeComponent {
  webLinks: ExternalLink[] = [
    // {
    //   title: 'Garuda Wiki',
    //   subTitle: 'Inform yourself about various Garuda Linux related things',
    //   externalLink: 'https://wiki.garudalinux.org',
    //   icon: 'pi pi-book',
    // },
    {
      title: 'welcome.gitlab',
      subTitle: 'welcome.gitlabSub',
      externalLink: 'https://gitlab.com/garuda-linux',
      icon: 'gitlab.png',
    },
    {
      title: 'welcome.chaotic',
      subTitle: 'welcome.chaoticSub',
      externalLink: 'https://aur.chaotic.cx',
      icon: 'chaotic-aur.png',
    },
    {
      title: 'welcome.searxng',
      subTitle: 'welcome.searxngSub',
      externalLink: 'https://searx.garudalinux.org',
      icon: 'searxng.svg',
    },
    {
      title: 'welcome.whoogle',
      subTitle: 'welcome.whoogleSub',
      externalLink: 'https://search.garudalinux.org',
      icon: 'whoogle.svg',
    },
    {
      title: 'welcome.vaultwarden',
      subTitle: 'welcome.vaultwardenSub',
      externalLink: 'https://bitwarden.garudalinux.org',
      icon: 'vaultwarden.svg',
    },
    {
      title: 'welcome.donate',
      subTitle: 'welcome.donateSub',
      externalLink: 'https://garudalinux.org/donate',
      icon: 'garuda-orange.webp',
    },
  ];
  contactLinks: ExternalLink[] = [
    {
      title: 'Forum',
      subTitle: 'Visit the Garuda Linux Forum',
      externalLink: 'https://forum.garudalinux.org',
      icon: faDiscourse,
    },
    {
      title: 'Telegram',
      subTitle: 'Visit the Telegram channel',
      externalLink: 'https://garudalinux.org/telegram',
      icon: faTelegram,
    },
    {
      title: 'Discord',
      subTitle: 'Visit the Telegram channel',
      externalLink: 'https://garudalinux.org/discord',
      icon: faDiscord,
    },
    {
      title: 'Mastodon',
      subTitle: 'Visit the Telegram channel',
      externalLink: 'https://social.garudalinux.org',
      icon: faMastodon,
    },
    {
      title: 'BlueSky',
      subTitle: 'Visit the Telegram channel',
      externalLink: 'https://bsky.app/profile/garudalinux.org',
      icon: faBluesky,
    },
    {
      title: 'IRC',
      subTitle: 'Join the interconnected IRC channel',
      externalLink: 'https://irc.garudalinux.org',
      icon: faComments,
    },
  ];

  protected readonly configService = inject(ConfigService);
  private readonly messageToastService = inject(MessageToastService);
  private readonly translocoService = inject(TranslocoService);
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly osInteractService = inject(OsInteractService);
  private readonly shellService = inject(ElectronShellService);
  private readonly router = inject(Router);

  mainLinks: HomepageLink[] = [
    {
      title: 'welcome.maintenance',
      subTitle: 'welcome.maintenanceSub',
      routerLink: '/maintenance',
      icon: 'pi pi-desktop',
    },
    {
      title: 'welcome.systemTools',
      subTitle: 'welcome.systemToolsSub',
      routerLink: '/system-tools',
      icon: 'pi pi-microchip',
    },
    {
      title: 'welcome.gaming',
      subTitle: 'welcome.gamingSub',
      routerLink: '/gaming',
      icon: 'pi pi-play-circle',
    },
    {
      title: 'welcome.bootTools',
      subTitle: 'welcome.bootToolsSub',
      routerLink: '/boot-options',
      icon: 'pi pi-hammer',
    },
    {
      title: 'welcome.diagnostics',
      subTitle: 'welcome.diagnosticsSub',
      routerLink: '/diagnostics',
      icon: 'pi pi-info-circle',
    },
    {
      title: 'welcome.network',
      subTitle: 'welcome.networkSub',
      routerLink: '/network-assistant',
      icon: 'pi pi-globe',
    },
    {
      title: 'welcome.install',
      subTitle: 'welcome.installSub',
      command: async () => {
        const result = await this.taskManagerService.executeAndWaitBash('/usr/lib/garuda/pkexec-gui calamares');
        if ('code' in result && result.code !== 0) {
          this.messageToastService.error(this.translocoService.translate('welcome.error'), result.stderr as string);
        }
      },
      icon: 'pi pi-download',
      condition: () => this.configService.state().isLiveSystem === true,
    },
    {
      title: 'welcome.chroot',
      subTitle: 'welcome.chrootSub',
      command: async () => {
        const result = await this.shellService.execute('/usr/lib/garuda/launch-terminal', [
          "pkexec garuda-chroot -a; read -p 'Press enter to exit'",
        ]);
        if ('code' in result && result.code !== 0) {
          this.messageToastService.error(this.translocoService.translate('welcome.error'), result.stderr as string);
        }
      },
      icon: 'pi pi-refresh',
      condition: () => this.configService.state().isLiveSystem === true,
    },
    {
      title: 'welcome.setupAssistant',
      subTitle: 'welcome.setupAssistantSub',
      command: () => void this.router.navigate(['/setup-wizard']),
      icon: 'pi pi-download',
      condition: () => this.configService.state().isLiveSystem === false,
    },
    {
      title: 'welcome.startpage',
      subTitle: 'welcome.startpageSub',
      routerLink: '/update',
      command: () => void open('https://start.garudalinux.org'),
      icon: 'pi pi-refresh',
      condition: () => this.configService.state().isLiveSystem === false,
    },
  ];

  /**
   * Respond to a click on an item.
   * @param item The item that was clicked.
   */
  respondClick(item: any) {
    if (item.command) {
      void item.command();
    } else if (item.externalLink) {
      void this.shellService.open(item.externalLink);
    }
  }
}
