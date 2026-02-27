import { ChangeDetectionStrategy, Component, computed, inject, ViewChild } from '@angular/core';
import { Drawer, DrawerModule } from 'primeng/drawer';
import { CommonModule } from '@angular/common';
import { ToastModule } from 'primeng/toast';
import { PrimeNG } from 'primeng/config';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { ConfirmPopupModule } from 'primeng/confirmpopup';
import { DesignCreateTheme } from './create/designcreatetheme';
import { DesignEditor } from './editor/designeditor';
import { DesignEditorFooter } from './editor/designeditorfooter';
import { DesignerService } from './designerservice';
import { ConfigService } from '../config/config.service';

@Component({
  selector: 'toolbox-designer',
  standalone: true,
  imports: [
    CommonModule,
    DrawerModule,
    ToastModule,
    ConfirmDialogModule,
    DesignCreateTheme,
    DesignEditor,
    DesignEditorFooter,
    ConfirmPopupModule,
  ],
  template: `<p-drawer
      #drawer
      [(visible)]="visible"
      [modal]="false"
      [dismissible]="false"
      position="right"
      styleClass="designer !w-screen md:!w-[48rem]"
    >
      <ng-template #headless>
        <div class="flex items-center justify-between p-5">
          <div class="flex items-center gap-2">
            <span class="font-bold text-xl">{{ viewTitle() }}</span>
          </div>
          <div class="flex items-center gap-2">
            <button class="icon-btn" (click)="toggleDarkMode()" type="button">
              <i class="pi" [ngClass]="{ 'pi-moon': isDarkTheme(), 'pi-sun': !isDarkTheme() }"></i>
            </button>
            <button class="icon-btn" (click)="hide($event)" type="button">
              <i class="pi pi-times"></i>
            </button>
          </div>
        </div>

        <div class="flex-auto overflow-auto overflow-x-hidden pb-5 px-5">
          <design-create-theme *ngIf="activeView() === 'create_theme'" />
          <design-editor *ngIf="activeView() === 'editor'" />
        </div>

        <div class="p-5">
          <design-editor-footer *ngIf="activeView() === 'editor'" />
        </div>
      </ng-template>
    </p-drawer>
    <p-toast key="designer" />
    <p-confirmdialog key="designer" />`,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AppDesigner {
  @ViewChild('drawer') drawer!: Drawer;

  designerService = inject(DesignerService);
  configService = inject(ConfigService);
  config: PrimeNG = inject(PrimeNG);

  activeView = computed(() => this.designerService.designer().activeView);
  viewTitle = computed(() => {
    switch (this.activeView()) {
      case 'create_theme':
        return 'Create Theme';
      case 'editor':
      default:
        return this.designerService.designer().theme.name;
    }
  });
  isDarkTheme = computed(() => this.configService.settings().darkMode);

  get visible() {
    return this.configService.state().designerActive;
  }
  set visible(value: boolean) {
    this.configService.updateState('designerActive', value);
  }

  hide(event: any) {
    this.drawer.close(event);
    this.configService.updateState('designerActive', false);
  }

  async toggleDarkMode() {
    await this.configService.updateConfig('darkMode', !this.configService.settings().darkMode);
  }
}
