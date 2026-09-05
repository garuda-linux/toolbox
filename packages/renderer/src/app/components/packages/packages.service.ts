import { inject, Service, signal } from '@angular/core';
import type { PackageSections } from '../gaming/interfaces';
import { LoadingService } from '../loading-indicator/loading-indicator.service';
import { OsInteractService } from '../task-manager/os-interact.service';
import { ConfigService } from '../config/config.service';
import { Logger } from '../../logging/logging';
import { ElectronFsService, resolveResource } from '../../electron-services';

@Service()
export class PackagesService {
  readonly loading = signal<boolean>(true);
  readonly packages = signal<PackageSections>([
    {
      name: 'packages.documents',
      sections: [],
    },
    {
      name: 'packages.internet',
      sections: [],
    },
    {
      name: 'packages.multimedia',
      sections: [],
    },
    {
      name: 'packages.other',
      sections: [],
    },
    {
      name: 'packages.science',
      sections: [],
    },
    {
      name: 'packages.security',
      sections: [],
    },
  ]);

  loadingService = inject(LoadingService);
  osInteractService = inject(OsInteractService);

  protected readonly configService = inject(ConfigService);
  private readonly fsService = new ElectronFsService();

  private readonly logger = Logger.getInstance();

  constructor() {
    void this.init();
  }

  async init() {
    this.loadingService.loadingOn();

    const sections = this.packages();
    for (const section of sections) {
      const path = `../../assets/parsed/${section.name.replace('packages.', '')}-repo.json`;
      try {
        const resourcePath: string = await resolveResource(path);

        // Use the new safe JSON reading method with default empty array
        section.sections = await this.fsService.safeReadJsonFile(resourcePath, [] as any[]);
        this.logger.debug(`Loaded section ${section.name} with ${section.sections.length} packages`);
      } catch (error) {
        this.logger.error(`Failed to load section ${section.name}: ${error}`);
        section.sections = [];
      }
    }

    const availablePkgs: PackageSections = sections.map((section) => {
      const sections: any[] = section.sections.filter((pkg) => {
        const pkgname: string = pkg.pkgname[0];
        const isAvailable: boolean = this.configService.state().availablePkgs.get(pkgname) === true;
        if (!isAvailable) {
          this.logger.warn(`Package ${pkgname} is not available, removing from list`);
        }
        return isAvailable;
      });
      return { ...section, sections };
    });

    this.packages.set(availablePkgs);
    this.loading.set(false);
    this.loadingService.loadingOff();
  }
}
