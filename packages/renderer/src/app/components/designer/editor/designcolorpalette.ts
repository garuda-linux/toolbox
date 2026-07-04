import { DesignerService } from '../designerservice';
import { ChangeDetectionStrategy, Component, inject, input } from '@angular/core';

@Component({
  selector: 'design-color-palette',
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: ` @if (value()) {
    @for (color of objectValues(value()); track color) {
      <div class="flex-1 h-8 w-8" [style.backgroundColor]="designerService.resolveColor(color)" [title]="color"></div>
    }
  }`,
  host: {
    class: 'flex w-full border border-surface rounded-l-lg rounded-r-lg overflow-hidden',
  },
})
export class DesignColorPalette {
  designerService: DesignerService = inject(DesignerService);

  value = input<Record<string, string>>({});

  objectValues = Object.values;
}
