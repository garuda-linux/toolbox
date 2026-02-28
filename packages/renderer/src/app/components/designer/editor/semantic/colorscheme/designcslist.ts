import { ChangeDetectionStrategy, Component, input } from '@angular/core';

import { FieldsetModule } from 'primeng/fieldset';
import { FormsModule } from '@angular/forms';
import { DesignTokenField } from '../../designtokenfield';

@Component({
  selector: 'design-cs-list',
  standalone: true,
  imports: [DesignTokenField, FormsModule, FieldsetModule],
  template: ` <p-fieldset [toggleable]="true" legend="List">
    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Option</div>
    <section class="grid grid-cols-4 mb-3 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().list.option.focusBackground"
          [type]="'color'"
          label="Focus BG"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().list.option.selectedBackground"
          [type]="'color'"
          label="Selected BG"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().list.option.selectedFocusBackground"
          [type]="'color'"
          label="Selected Focus BG"
        />
      </div>
      <div class="flex flex-col gap-1"></div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().list.option.color" [type]="'color'" label="Color" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().list.option.selectedColor"
          [type]="'color'"
          label="Selected Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().list.option.selectedFocusColor"
          [type]="'color'"
          label="Selected Focus Colo"
        />
      </div>
      <div class="flex flex-col gap-1"></div>
    </section>

    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Option Icon</div>
    <section class="grid grid-cols-4 mb-3 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().list.option.icon.color" [type]="'color'" label="Color" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().list.option.icon.focusColor"
          [type]="'color'"
          label="Focus Color"
        />
      </div>
    </section>

    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Option Group</div>
    <section class="grid grid-cols-4 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().list.optionGroup.background" [type]="'color'" label="BG" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().list.optionGroup.color" [type]="'color'" label="Color" />
      </div>
      <div class="flex flex-col gap-1"></div>
      <div class="flex flex-col gap-1"></div>
    </section>
  </p-fieldset>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DesignCSList {
  readonly colorScheme = input<any>();
}
