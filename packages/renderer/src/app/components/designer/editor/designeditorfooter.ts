import { ChangeDetectionStrategy, Component, inject } from '@angular/core';

import { DesignerService } from '../designerservice';
import { ConfigService } from '../../config/config.service';

@Component({
  standalone: true,
  selector: 'design-editor-footer',
  imports: [],
  template: `<div class="flex justify-end gap-2">
    <button class="btn-design-outlined" (click)="save()" type="button">Save</button>
    <button class="btn-design" (click)="apply()" type="button">Apply</button>
    <button class="btn-design-outlined" (click)="download()" type="button">Download</button>
    <button class="btn-design-outlined" (click)="reset()" type="button">Reset</button>
  </div>`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DesignEditorFooter {
  configService: ConfigService = inject(ConfigService);
  designerService: DesignerService = inject(DesignerService);

  async apply() {
    if (!this.designerService.designer().theme) {
      await this.designerService.createThemeFromPreset();
    }
  }

  async save() {
    await this.configService.updateConfig('customDesign', JSON.stringify(this.designerService.designer().theme));
  }

  async reset() {
    this.designerService.designer.set({
      ...this.designerService.designer(),
      activeView: 'create_theme',
      theme: {
        key: null,
        name: null,
        preset: null,
        config: null,
      },
    });
  }

  async download() {
    await this.designerService.downloadTheme(this.designerService.designer().theme);
  }
}
