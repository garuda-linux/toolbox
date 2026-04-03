import { ChangeDetectionStrategy, Component, HostListener, inject, model, signal } from '@angular/core';
import { DialogModule } from 'primeng/dialog';
import { AutoComplete, type AutoCompleteCompleteEvent, type AutoCompleteSelectEvent } from 'primeng/autocomplete';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { MODULE_SEARCH } from '../../constants/module-search';
import { CommandPaletteService } from './command-palette.service';

@Component({
  selector: 'toolbox-command-palette',
  imports: [DialogModule, AutoComplete, FormsModule, TranslocoDirective],
  templateUrl: './command-palette.component.html',
  styleUrl: './command-palette.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class CommandPaletteComponent {
  visible = model<boolean>(false);
  selectedItem = signal<any>(null);
  suggestions = signal<any[]>([]);

  private readonly router = inject(Router);
  private readonly translocoService = inject(TranslocoService);
  private readonly commandPaletteService = inject(CommandPaletteService);

  @HostListener('document:keydown', ['$event'])
  handleKeyboardEvent(event: KeyboardEvent) {
    if ((event.ctrlKey || event.metaKey) && event.key === 'p') {
      event.preventDefault();
      this.visible.update((v) => !v);
    }
  }

  search(event: AutoCompleteCompleteEvent) {
    const query = event.query.toLowerCase();
    const results: any[] = [];

    for (const action of this.commandPaletteService.getActions()) {
      const translatedLabel = this.translocoService.translate(action.label);
      const translatedDescription = action.description ? this.translocoService.translate(action.description) : '';
      if (
        translatedLabel.toLowerCase().includes(query) ||
        translatedDescription.toLowerCase().includes(query) ||
        action.keywords.some((k) => k.toLowerCase().includes(query))
      ) {
        results.push({
          moduleName: action.label,
          description: action.description,
          routerLink: action.routerLink,
          hash: action.hash,
          icon: action.icon,
          type: 'action',
          command: action.command,
        });
      }
    }

    for (const module of MODULE_SEARCH) {
      if (
        this.translocoService.translate(module.moduleName).toLowerCase().includes(query) ||
        module.keywords.some((k) => k.toLowerCase().includes(query))
      ) {
        results.push({
          ...module,
          type: 'module',
          icon: 'pi pi-external-link',
        });
      }
    }

    this.suggestions.set(results);
  }

  onSelect(event: AutoCompleteSelectEvent) {
    const item = event.value;
    this.visible.set(false);
    this.selectedItem.set(null);

    const actualItem = typeof item === 'string' ? this.suggestions().find((s) => s.moduleName === item) : item;
    if (!actualItem) {
      return;
    }

    if (actualItem.command) {
      void actualItem.command();
    } else if (actualItem.routerLink) {
      void this.router.navigate([actualItem.routerLink], { fragment: actualItem.hash });
    }
  }
}
