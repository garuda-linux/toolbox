import { Component, computed, inject, input } from '@angular/core';

import { DesignerService } from '../../designerservice';
import { DesignTokenField } from '../designtokenfield';

@Component({
  selector: 'design-component-section',
  standalone: true,
  imports: [DesignTokenField],
  template: `<section>
    <div class="text-sm mb-1 font-semibold text-surface-950 dark:text-surface-0 capitalize">
      {{ sectionName() }}
    </div>
    <div class="grid grid-cols-4 gap-x-2 gap-y-3">
      @for (entry of objectKeys(tokens()); track entry) {
        @if (!isObject(tokens()[entry])) {
          <design-token-field
            [(modelValue)]="tokens()[entry]"
            [label]="camelCaseToSpaces(entry)"
            [componentKey]="componentKey()"
            [path]="path() + '.' + entry"
            [type]="isColor(entry) ? 'color' : undefined"
            [switchable]="true"
          />
        }
      }
    </div>
    @if (hasNestedTokens()) {
      @for (entry of objectKeys(nestedTokens()); track entry) {
        <design-component-section class="block mt-3" [componentKey]="componentKey()" [path]="path() + '.' + entry" />
      }
    }
  </section>`,
})
export class DesignComponentSection {
  objectKeys = Object.keys;

  designerService: DesignerService = inject(DesignerService);

  componentKey = input<string>();

  path = input<any>();

  sectionName = computed(() => {
    const names = this.path().split('.');

    return names
      .filter((n: string) => n !== 'colorScheme' && n !== 'light' && n !== 'dark')
      .map((n: string) => this.capitalize(this.camelCaseToSpaces(n)))
      .join(' ');
  });

  tokens = computed(() => {
    const designer = this.designerService.designer();
    if (!designer.theme.preset) {
      return {};
    }

    // @ts-expect-error - Dynamic component key access on preset components object
    const source = designer.theme.preset.components[this.componentKey()];
    return this.getObjectProperty(source, this.path());
  });

  nestedTokens = computed(() => {
    const groups = {};
    const obj = this.tokens();

    for (const key in obj) {
      if (Object.prototype.hasOwnProperty.call(obj, key)) {
        const value = obj[key];

        if (this.isObject(value)) {
          // @ts-expect-error - Dynamic property assignment to groups object
          groups[key] = value;
        }
      }
    }

    return groups;
  });

  hasNestedTokens = computed(() => {
    return Object.keys(this.nestedTokens()).length > 0;
  });

  camelCaseToSpaces(val: string) {
    return val.replace(/([a-z])([A-Z])/g, '$1 $2');
  }

  isColor(val: string) {
    return val.toLowerCase().includes('color') || val.toLowerCase().includes('background');
  }

  isObject(val: null) {
    return val !== null && typeof val === 'object';
  }

  getObjectProperty(obj: any, path: string) {
    const keys = path.split('.');
    let current = obj;

    for (const key of keys) {
      if (current[key] !== undefined) {
        current = current[key];
      } else {
        return undefined;
      }
    }

    return current;
  }

  capitalize(str: any) {
    if (typeof str !== 'string' || str.length === 0) {
      return str;
    }

    return str.charAt(0).toUpperCase() + str.slice(1);
  }
}
