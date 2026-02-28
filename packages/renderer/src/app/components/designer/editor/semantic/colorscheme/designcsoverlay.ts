import { ChangeDetectionStrategy, Component, input } from '@angular/core';

import { FieldsetModule } from 'primeng/fieldset';
import { FormsModule } from '@angular/forms';
import { DesignTokenField } from '../../designtokenfield';

@Component({
  selector: 'design-cs-overlay',
  standalone: true,
  imports: [DesignTokenField, FormsModule, FieldsetModule],
  template: ` <p-fieldset [toggleable]="true" legend="Overlay">
    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Select</div>
    <section class="grid grid-cols-4 mb-3 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().overlay.select.background" [type]="'color'" label="BG" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().overlay.select.borderColor"
          [type]="'color'"
          label="Border Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().overlay.select.color" [type]="'color'" label="Color" />
      </div>
      <div class="flex flex-col gap-1"></div>
    </section>

    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Popover</div>
    <section class="grid grid-cols-4 mb-3 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().overlay.popover.background" [type]="'color'" label="BG" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().overlay.popover.borderColor"
          [type]="'color'"
          label="Border Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().overlay.popover.color" [type]="'color'" label="Color" />
      </div>
      <div class="flex flex-col gap-1"></div>
    </section>

    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Modal</div>
    <section class="grid grid-cols-4 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().overlay.modal.background" [type]="'color'" label="BG" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().overlay.modal.borderColor"
          [type]="'color'"
          label="Border Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().overlay.modal.color" [type]="'color'" label="Color" />
      </div>
      <div class="flex flex-col gap-1"></div>
    </section>
  </p-fieldset>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DesignCSOverlay {
  readonly colorScheme = input<any>();
}
