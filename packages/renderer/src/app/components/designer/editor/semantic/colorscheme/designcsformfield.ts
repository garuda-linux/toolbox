import { ChangeDetectionStrategy, Component, input } from '@angular/core';

import { FieldsetModule } from 'primeng/fieldset';
import { FormsModule } from '@angular/forms';
import { DesignTokenField } from '../../designtokenfield';

@Component({
  selector: 'design-cs-form-field',
  standalone: true,
  imports: [DesignTokenField, FormsModule, FieldsetModule],
  template: ` <p-fieldset [toggleable]="true" legend="Form Field">
    <section class="grid grid-cols-4 mb-3 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().formField.background" [type]="'color'" label="BG" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().formField.color" [type]="'color'" label="Color" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().formField.iconColor" [type]="'color'" label="Icon Color" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().formField.shadow" label="Shadow" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.borderColor"
          [type]="'color'"
          label="Border Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.hoverBorderColor"
          [type]="'color'"
          label="Hover Border Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.focusBorderColor"
          [type]="'color'"
          label="Focus Border Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.invalidBorderColor"
          [type]="'color'"
          label="Invalid Border Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.disabledBackground"
          [type]="'color'"
          label="Disabled BG"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.disabledColor"
          [type]="'color'"
          label="Disabled Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.placeholderColor"
          [type]="'color'"
          label="Placeholder"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.invalidPlaceholderColor"
          [type]="'color'"
          label="Invalid Placeholder"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.filledBackground"
          [type]="'color'"
          label="Filled BG"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.filledHoverBackground"
          [type]="'color'"
          label="Filled Hover BG"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.filledFocusBackground"
          [type]="'color'"
          label="Filled Focus BG"
        />
      </div>
      <div class="flex flex-col gap-1"></div>
    </section>

    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Float Label</div>
    <section class="grid grid-cols-4 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().formField.floatLabelColor" [type]="'color'" label="Color" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.floatLabelFocusColor"
          [type]="'color'"
          label="Focus Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.floatLabelActiveColor"
          [type]="'color'"
          label="Active Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().formField.floatLabelInvalidColor"
          [type]="'color'"
          label="Invalid Color"
        />
      </div>
    </section>
  </p-fieldset>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DesignCSFormField {
  readonly colorScheme = input<any>();
}
