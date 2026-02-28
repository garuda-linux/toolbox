import { ChangeDetectionStrategy, Component, input } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { palette } from '@primeuix/themes';
import { FieldsetModule } from 'primeng/fieldset';
import { DesignTokenField } from '../../designtokenfield';
import { DesignColorPalette } from '../../designcolorpalette';

@Component({
  selector: 'design-cs-common',
  standalone: true,
  imports: [DesignColorPalette, DesignTokenField, FormsModule, FieldsetModule],
  template: ` <p-fieldset [toggleable]="true" legend="Common">
    <section class="flex justify-between items-center mb-4 gap-8">
      <div class="flex gap-2 items-center">
        <span class="text-sm">Surface</span>
        <input [value]="colorScheme().surface['500']" [type]="'color'" (input)="onSurfaceColorChange($event)" />
      </div>
      <design-color-palette [value]="colorScheme().surface" />
    </section>
    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Typography</div>
    <section class="grid grid-cols-4 mb-3 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().text.color" [type]="'color'" label="Text" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().text.hoverColor" [type]="'color'" label="Text Hover" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().text.mutedColor" [type]="'color'" label="Text Muted" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().text.hoverMutedColor"
          [type]="'color'"
          label="Text Hover Muted"
        />
      </div>
    </section>
    <section class="grid grid-cols-4 mb-3 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().content.background" [type]="'color'" label="Content BG" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().content.hoverBackground"
          [type]="'color'"
          label="Content Hover BG"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().content.borderColor"
          [type]="'color'"
          label="Content Border Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().content.color" [type]="'color'" label="Content Color" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().content.hoverColor"
          [type]="'color'"
          label="Content Hover Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().mask.background" [type]="'color'" label="Mask BG" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().mask.color" [type]="'color'" label="Mask Color" />
      </div>
    </section>

    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Accent</div>
    <section class="grid grid-cols-4 mb-3 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().primary.color" [type]="'color'" label="Primary" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().primary.contrastColor"
          [type]="'color'"
          label="Primary Contrast"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().primary.hoverColor" [type]="'color'" label="Primary Hover" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().primary.activeColor"
          [type]="'color'"
          label="Primary Active"
        />
      </div>
    </section>
    <section class="grid grid-cols-4 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().highlight.background" [type]="'color'" label="Highlight BG" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="colorScheme().highlight.color" [type]="'color'" label="Highlight Color" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().highlight.focusBackground"
          [type]="'color'"
          label="Highlight Focus BG"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="colorScheme().highlight.focusColor"
          [type]="'color'"
          label="Highlight Focus Color"
        />
      </div>
    </section>
  </p-fieldset>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DesignCSCommon {
  readonly colorScheme = input<any>();

  onSurfaceColorChange(event: any) {
    // @ts-expect-error - colorScheme.surface property may not be fully typed at runtime
    this.colorScheme().surface = { ...{ 0: '#ffffff' }, ...palette(event.target.value) };
  }
}
