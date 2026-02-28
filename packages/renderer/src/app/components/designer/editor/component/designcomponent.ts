import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { TabsModule } from 'primeng/tabs';
import { FieldsetModule } from 'primeng/fieldset';

import { DesignerService } from '../../designerservice';
import { DesignComponentSection } from './designcomponentsection';
import { ConfigService } from '../../../config/config.service';
import { Select } from 'primeng/select';
import { FormsModule } from '@angular/forms';

@Component({
  selector: 'design-component',
  standalone: true,
  imports: [FieldsetModule, TabsModule, DesignComponentSection, Select, FormsModule],
  template: `<section class="flex flex-col gap-3">
    <p-select class="capitalize mb-2" [(ngModel)]="componentKey" [options]="availableComponents" />

    <p-fieldset [toggleable]="true" legend="Common">
      <div class="flex flex-col gap-3">
        @if (hasCommonTokens()) {
          @for (entry of objectKeys(this.tokens()); track entry) {
            @if (entry !== 'colorScheme' && entry !== 'css') {
              <design-component-section [componentKey]="componentKey()" [path]="entry" />
            }
          }
        } @else {
          <span class="block py-3">There are no design tokens</span>
        }
      </div>
    </p-fieldset>
    <p-fieldset [toggleable]="true" legend="Color Scheme">
      @if (hasColorScheme()) {
        <p-tabs (valueChange)="tabValueChange($event)" value="cs-0">
          <p-tablist>
            <p-tab value="cs-0">Light</p-tab>
            <p-tab value="cs-1">Dark</p-tab>
          </p-tablist>
          <p-tabpanels>
            <p-tabpanel value="cs-0">
              <div class="flex flex-col gap-3">
                @for (entry of objectKeys(lightTokens()); track entry) {
                  <design-component-section [componentKey]="componentKey()" [path]="'colorScheme.light.' + entry" />
                }
              </div>
            </p-tabpanel>
            <p-tabpanel value="cs-1">
              <div class="flex flex-col gap-3">
                @for (entry of objectKeys(darkTokens()); track entry) {
                  <design-component-section [componentKey]="componentKey()" [path]="'colorScheme.dark.' + entry" />
                }
              </div>
            </p-tabpanel>
          </p-tabpanels>
        </p-tabs>
      } @else {
        <span class="block py-3">There are no design tokens defined per color scheme.</span>
      }
    </p-fieldset>
  </section>`,
})
export class DesignComponent implements OnInit {
  objectKeys = Object.keys;

  configService: ConfigService = inject(ConfigService);
  designerService: DesignerService = inject(DesignerService);

  availableComponents: string[] = [];
  componentKey = signal<string>('');
  // @ts-expect-error - Dynamic component key access on preset components object
  tokens = computed(() => this.designerService.designer()?.theme?.preset?.components[this.componentKey()] || {});
  lightTokens = computed(() => {
    const designer = this.designerService.designer();
    // @ts-expect-error - Dynamic component key access for color scheme tokens
    return designer.theme.preset?.components[this.componentKey()].colorScheme?.light;
  });
  // @ts-expect-error - Accessing potentially undefined colorScheme property
  darkTokens = computed(() => this.tokens().colorScheme?.dark);
  // @ts-expect-error - Accessing potentially undefined colorScheme property
  hasColorScheme = computed(() => this.tokens().colorScheme !== undefined);
  hasCommonTokens = computed(
    () =>
      Object.keys(this.tokens()).filter((name: string) => {
        return name !== 'colorScheme' && name !== 'css';
      }).length > 0,
  );

  ngOnInit() {
    // @ts-expect-error - Preset components may not be fully typed at runtime
    this.availableComponents = Object.keys(this.designerService.designer().theme.preset.components);
    this.componentKey.set(this.availableComponents[0] || '');
  }

  async tabValueChange(event: string | number | undefined) {
    if (event === 'cs-1') {
      await this.configService.updateConfig('darkTheme', true);
    }
    if (event === 'cs-0') {
      await this.configService.updateConfig('darkTheme', false);
    }
  }
}
