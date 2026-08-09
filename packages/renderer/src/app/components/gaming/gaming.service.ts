import { inject, Service, signal } from '@angular/core';
import type { FullPackageDefinition, PackageSections } from './interfaces';
import { gamingPackageLists } from './package-lists';
import { ConfigService } from '../config/config.service';
import { Logger } from '../../logging/logging';

@Service()
export class GamingService {
  packages = signal<PackageSections>(gamingPackageLists);

  private readonly configService = inject(ConfigService);
  private readonly logger = Logger.getInstance();

  constructor() {
    const availablePkgs: PackageSections = this.packages().map((section) => {
      const sections: FullPackageDefinition[] = section.sections.filter((pkg) => {
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
  }
}
