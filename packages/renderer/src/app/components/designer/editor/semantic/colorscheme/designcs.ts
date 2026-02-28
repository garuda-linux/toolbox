import { Component, input } from '@angular/core';

import { DesignCSCommon } from './designcscommon';
import { DesignCSOverlay } from './designcsoverlay';
import { DesignCSList } from './designcslist';
import { DesignCSFormField } from './designcsformfield';
import { DesignCSNavigation } from './designcsnavigation';

@Component({
  selector: 'design-cs',
  standalone: true,
  imports: [DesignCSCommon, DesignCSOverlay, DesignCSList, DesignCSFormField, DesignCSNavigation],
  template: ` <design-cs-common [colorScheme]="value()" />
    <design-cs-form-field [colorScheme]="value()" />
    <design-cs-overlay [colorScheme]="value()" />
    <design-cs-list [colorScheme]="value()" />
    <design-cs-navigation [colorScheme]="value()" />`,
  host: {
    class: 'flex flex-col gap-3',
  },
})
export class DesignCS {
  readonly value = input<any>();
}
