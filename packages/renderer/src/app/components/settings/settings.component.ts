import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { ConfigService } from '../config/config.service';
import { Checkbox } from 'primeng/checkbox';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { FormsModule } from '@angular/forms';
import { Select, SelectChangeEvent } from 'primeng/select';
import { LogLevel } from '../../logging/interfaces';
import { Logger } from '../../logging/logging';
import { LangPipePipe } from '../lang-pipe/lang-pipe.pipe';
import { themes } from '../../theme';
import { Card } from 'primeng/card';
import { Button } from 'primeng/button';
import { FileUpload } from 'primeng/fileupload';
import { MessageToastService } from '@garudalinux/core';
import { WallpaperService } from '../wallpaper/wallpaper.service';
import { Slider } from 'primeng/slider';

@Component({
  selector: 'toolbox-settings',
  imports: [Checkbox, TranslocoDirective, FormsModule, Select, Card, Button, FileUpload, Slider],
  templateUrl: './settings.component.html',
  styleUrl: './settings.component.css',
  providers: [LangPipePipe],
})
export class SettingsComponent implements OnInit {
  protected readonly configService = inject(ConfigService);
  protected readonly checkBoxSettings: string[] = [
    'copyDiagnostics',
    'darkMode',
    'autoStart',
    'autoRefresh',
    'showMainLinks',
    'systemdUserContext',
  ];

  protected readonly availableThemes: string[] = Object.keys(themes);
  protected readonly languages = signal<{ language: string; label: string }[]>([]);
  protected readonly logLevels: string[] = Object.values(LogLevel).filter((key) => typeof key !== 'number');
  protected readonly logLevelType = LogLevel;

  protected readonly settings = computed(() => {
    const settings: { name: string; value: boolean }[] = [];
    for (const [name, value] of Object.entries(this.configService.settings())) {
      if (this.checkBoxSettings.includes(name)) settings.push({ name, value });
    }
    return settings;
  });

  private readonly languagePipe = inject(LangPipePipe);
  private readonly logger = Logger.getInstance();
  private readonly messageToastService = inject(MessageToastService);
  private readonly translocoService = inject(TranslocoService);
  private readonly wallpaperService = inject(WallpaperService);

  ngOnInit() {
    const languages = [];
    for (const lang of this.translocoService.getAvailableLangs()) {
      languages.push({
        language: lang as string,
        label: this.languagePipe.transform(lang as string),
      });
    }

    this.languages.set(languages);
  }

  async toggleSetting(entry: string) {
    await this.configService.updateConfig(entry, !this.configService.settings()[entry]);
  }

  async selectLogLevel($event: SelectChangeEvent) {
    const level = $event.value as 'ERROR' | 'DEBUG' | 'WARN' | 'TRACE' | 'INFO';
    await this.configService.updateConfig('logLevel', LogLevel[level]);
  }

  async selectLanguage($event: SelectChangeEvent) {
    this.logger.trace($event.value);
    await this.configService.updateConfig('language', $event.value);
  }

  async updateConfig(config: string, value: unknown) {
    await this.configService.updateConfig(config, value);
  }

  async clearBackground() {
    await this.configService.updateConfig('background', null);
  }

  async onFileSelect(event: { files: File[] }) {
    const file: File = event.files[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e: ProgressEvent<FileReader>) => {
      try {
        this.configService.updateConfig('background', e.target?.result);
        this.messageToastService.success('File loaded successfully', 'The background image has been updated.');
      } catch {
        this.messageToastService.error('Error loading file', 'The selected file is not a valid image or is corrupted.');
      }
    };
    reader.onerror = (e: ProgressEvent<FileReader>) => {
      this.logger.error(`Error reading file: ${e.target?.error?.message}`);
      this.messageToastService.error('Error reading file', 'An error occurred while reading the file.');
    };
    reader.readAsDataURL(file);
  }
}
