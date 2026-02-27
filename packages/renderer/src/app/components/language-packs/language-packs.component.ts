import { ChangeDetectionStrategy, Component, effect, inject, model } from '@angular/core';
import { OsInteractService } from '../task-manager/os-interact.service';
import { TableModule } from 'primeng/table';
import { TranslocoDirective } from '@jsverse/transloco';
import { FormsModule } from '@angular/forms';
import { StatefulPackage } from '../gaming/interfaces';
import { LanguagePacks } from './types';
import { NgClass, NgIf } from '@angular/common';
import { Checkbox } from 'primeng/checkbox';
import { LanguagePacksService } from './language-packs.service';
import { LocalePipe } from '../lang-pipe/locale.pipe';

@Component({
  selector: 'toolbox-language-packs',
  imports: [TableModule, TranslocoDirective, FormsModule, NgIf, Checkbox, NgClass, LocalePipe],
  templateUrl: './language-packs.component.html',
  styleUrl: './language-packs.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class LanguagePacksComponent {
  selectedPacks = model<LanguagePacks>([]);

  protected readonly languagePacksService = inject(LanguagePacksService);
  private readonly osInteractService = inject(OsInteractService);

  constructor() {
    effect(() => {
      const _packages: Map<string, boolean> = this.osInteractService.packages();
      if (!this.languagePacksService.loading()) {
        this.updateUi();
      }
    });
  }

  /**
   * Toggles the selected state of a package.
   * @param item The package to toggle.
   */
  togglePackage(item: StatefulPackage): void {
    for (const pkgname of item.pkgname) {
      this.osInteractService.togglePackage(pkgname);
    }
  }

  /**
   * Update the state of the UI based on the installed packages.
   */
  updateUi(): void {
    const installedPackages: Map<string, boolean> = this.osInteractService.packages();
    this.languagePacksService.languagePacks.update((languagePacks) => {
      for (const pack of languagePacks) {
        pack.selected = installedPackages.get(pack.pkgname[0]) === true;
      }
      return languagePacks;
    });
  }
}
