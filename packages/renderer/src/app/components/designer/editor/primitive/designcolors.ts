import { DesignColorPalette } from '../designcolorpalette';
import { DesignerService } from '../../designerservice';

import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { palette } from '@primeuix/themes';
import { FieldsetModule } from 'primeng/fieldset';

@Component({
  selector: 'design-colors',
  standalone: true,
  imports: [FieldsetModule, FormsModule, DesignColorPalette],
  template: ` <p-fieldset [toggleable]="true" legend="Colors">
    @for (key of objectKeys(designerService.designer().theme?.preset?.primitive); track key) {
      @if (key !== 'borderRadius') {
        <section class="flex justify-between items-center mb-4 gap-8">
          <div class="flex gap-2 items-center">
            <span class="text-sm capitalize block w-20">{{ key }}</span>
            <input
              [value]="
                designerService.resolveColor($any(designerService.designer().theme.preset?.primitive)[key]['500'])
              "
              (change)="onColorChange($event, key)"
              (blur)="onBlur()"
              type="color"
            />
          </div>
          <design-color-palette [value]="$any(designerService.designer().theme?.preset?.primitive)[key]" />
        </section>
      }
    }
  </p-fieldset>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DesignColors {
  designerService: DesignerService = inject(DesignerService);

  onColorChange(event: any, color: any) {
    // @ts-expect-error - designer signal update may not be fully typed
    this.designerService.designer.update((prev) => {
      if (prev.theme.preset) {
        return {
          ...prev,
          theme: {
            ...prev.theme,
            preset: {
              ...prev.theme.preset,
              // @ts-expect-error - dynamic color property assignment on primitive object
              primitive: { ...prev.theme.preset.primitive, [color]: palette(event.target.value) },
            },
          },
        };
      }
      return prev;
    });
  }

  onBlur() {
    this.designerService.refreshACTokens();
  }

  objectKeys = Object.keys;
}
