import { ChangeDetectionStrategy, Component, inject } from '@angular/core';

import { FieldsetModule } from 'primeng/fieldset';
import { FormsModule } from '@angular/forms';
import { DesignerService } from '../../designerservice';
import { DesignTokenField } from '../designtokenfield';

@Component({
  selector: 'design-border-radius',
  standalone: true,
  imports: [DesignTokenField, FieldsetModule, FormsModule],
  template: ` <p-fieldset [toggleable]="true" legend="Rounded">
    <section class="grid grid-cols-4 gap-2">
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="borderRadiusNone" label="None" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="borderRadiusXs" label="Extra Small" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="borderRadiusSm" label="Small" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="borderRadiusMd" label="Medium" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="borderRadiusLg" label="Large" />
      </div>
      <div class="flex flex-col gap-1">
        <design-token-field [(modelValue)]="borderRadiusXl" label="Extra Large" />
      </div>
    </section>
  </p-fieldset>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DesignBorderRadius {
  designerService = inject(DesignerService);

  get borderRadiusNone() {
    // @ts-expect-error - dynamic property access on theme preset object
    return this.designerService.designer().theme?.preset?.primitive?.borderRadius.none;
  }
  set borderRadiusNone(value: any) {
    // @ts-expect-error - dynamic property access on theme preset object
    this.designerService.designer.update((prev) => {
      if (prev.theme.preset) {
        return {
          ...prev,
          theme: {
            ...prev.theme,
            preset: {
              ...prev.theme.preset,
              primitive: {
                // @ts-expect-error - dynamic property access on theme preset object
                ...prev.theme.preset.primitive,
                // @ts-expect-error - dynamic property access on theme preset object
                borderRadius: { ...prev.theme.preset.primitive.borderRadius, none: value },
              },
            },
          },
        };
      }
      return prev;
    });
  }

  get borderRadiusXs() {
    // @ts-expect-error - dynamic property access on theme preset object
    return this.designerService.designer().theme?.preset?.primitive.borderRadius.xs;
  }
  set borderRadiusXs(value: any) {
    // @ts-expect-error - dynamic property access on theme preset object
    this.designerService.designer.update((prev) => {
      if (prev.theme.preset) {
        return {
          ...prev,
          theme: {
            ...prev.theme,
            preset: {
              ...prev.theme.preset,
              primitive: {
                // @ts-expect-error - dynamic property access on theme preset object
                ...prev.theme.preset.primitive,
                // @ts-expect-error - dynamic property access on theme preset object
                borderRadius: { ...prev.theme.preset.primitive.borderRadius, xs: value },
              },
            },
          },
        };
      }
      return prev;
    });
  }

  get borderRadiusSm() {
    // @ts-expect-error - dynamic property access on theme preset object
    return this.designerService.designer().theme?.preset?.primitive?.borderRadius.sm;
  }
  set borderRadiusSm(value: any) {
    // @ts-expect-error - dynamic property access on theme preset object
    this.designerService.designer.update((prev) => {
      if (prev.theme.preset) {
        return {
          ...prev,
          theme: {
            ...prev.theme,
            preset: {
              ...prev.theme.preset,
              primitive: {
                // @ts-expect-error - dynamic property access on theme preset object
                ...prev.theme.preset.primitive,
                // @ts-expect-error - dynamic property access on theme preset object
                borderRadius: { ...prev.theme.preset.primitive.borderRadius, sm: value },
              },
            },
          },
        };
      }
      return prev;
    });
  }

  get borderRadiusMd() {
    // @ts-expect-error - dynamic property access on theme preset object
    return this.designerService.designer().theme?.preset?.primitive?.borderRadius.md;
  }
  set borderRadiusMd(value: any) {
    // @ts-expect-error - dynamic property access on theme preset object
    this.designerService.designer.update((prev) => {
      if (prev.theme.preset) {
        return {
          ...prev,
          theme: {
            ...prev.theme,
            preset: {
              ...prev.theme.preset,
              primitive: {
                // @ts-expect-error - dynamic property access on theme preset object
                ...prev.theme.preset.primitive,
                // @ts-expect-error - dynamic property access on theme preset object
                borderRadius: { ...prev.theme.preset.primitive.borderRadius, md: value },
              },
            },
          },
        };
      }
      return prev;
    });
  }

  get borderRadiusLg() {
    // @ts-expect-error - dynamic property access on theme preset object
    return this.designerService.designer().theme?.preset?.primitive?.borderRadius.lg;
  }
  set borderRadiusLg(value: any) {
    // @ts-expect-error - dynamic property access on theme preset object
    this.designerService.designer.update((prev) => {
      if (prev.theme.preset) {
        return {
          ...prev,
          theme: {
            ...prev.theme,
            preset: {
              ...prev.theme.preset,
              primitive: {
                // @ts-expect-error - dynamic property access on theme preset object
                ...prev.theme.preset.primitive,
                // @ts-expect-error - dynamic property access on theme preset object
                borderRadius: { ...prev.theme.preset.primitive.borderRadius, lg: value },
              },
            },
          },
        };
      }
      return prev;
    });
  }

  get borderRadiusXl() {
    // @ts-expect-error - dynamic property access on theme preset object
    return this.designerService.designer().theme?.preset?.primitive?.borderRadius.xl;
  }
  set borderRadiusXl(value: any) {
    // @ts-expect-error - dynamic property access on theme preset object
    this.designerService.designer.update((prev) => {
      if (prev.theme.preset) {
        return {
          ...prev,
          theme: {
            ...prev.theme,
            preset: {
              ...prev.theme.preset,
              primitive: {
                // @ts-expect-error - dynamic property access on theme preset object
                ...prev.theme.preset.primitive,
                // @ts-expect-error - dynamic property access on theme preset object
                borderRadius: { ...prev.theme.preset.primitive.borderRadius, xl: value },
              },
            },
          },
        };
      }
      return prev;
    });
  }
}
