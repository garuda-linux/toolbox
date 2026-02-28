import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { palette } from '@primeuix/themes';
import { FieldsetModule } from 'primeng/fieldset';
import { DesignTokenField } from '../designtokenfield';
import { DesignColorPalette } from '../designcolorpalette';
import { DesignerService } from '../../designerservice';

@Component({
  selector: 'design-general',
  standalone: true,
  imports: [DesignColorPalette, DesignTokenField, FormsModule, FieldsetModule],
  template: ` <p-fieldset [toggleable]="true" legend="General">
    <section class="flex justify-between items-center mb-4 gap-8">
      <div class="flex gap-2 items-center">
        <span class="text-sm">Primary</span>
        <input
          [value]="designerService.resolveColor($any(designerService.designer().theme.preset?.semantic).primary['500'])"
          (input)="onPrimaryColorChange($event)"
          type="color"
        />
      </div>
      <design-color-palette [value]="$any(semantic())?.primary" />
    </section>
    <section class="grid grid-cols-4 mb-3 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).transitionDuration"
          label="Transition Duration"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).disabledOpacity"
          label="Disabled Opacity"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).iconSize"
          label="Icon Size"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).anchorGutter"
          label="Anchor Gutter"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).content.borderRadius"
          label="Border Radius"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).mask.transitionDuration"
          label="Mask Transition Dur."
        />
      </div>
      <div class="flex flex-col gap-1"></div>
      <div class="flex flex-col gap-1"></div>
    </section>

    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0">Focus Ring</div>
    <section class="grid grid-cols-4 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).focusRing.width"
          label="Width"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).focusRing.style"
          label="Style"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).focusRing.color"
          [type]="'color'"
          label="Color"
        />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field
          [(modelValue)]="$any(designerService.designer().theme.preset?.semantic).focusRing.offset"
          label="Offset"
        />
      </div>
    </section>
  </p-fieldset>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DesignGeneral {
  designerService: DesignerService = inject(DesignerService);

  semantic = computed(() => this.designerService.designer().theme.preset?.semantic);

  onPrimaryColorChange(event: Event) {
    // @ts-expect-error - designer signal update may not be fully typed
    this.designerService.designer.update((prev) => {
      if (prev.theme.preset) {
        return {
          ...prev,
          theme: {
            ...prev.theme,
            preset: {
              ...prev.theme.preset,
              // @ts-expect-error - semantic property may not be fully typed at runtime
              semantic: { ...prev.theme.preset.semantic, primary: palette(event.target.value) },
            },
          },
        };
      }
      return prev;
    });
  }
}
