import { HTTP_INTERCEPTORS, provideHttpClient, withFetch } from '@angular/common/http';
import {
  type ApplicationConfig,
  inject,
  isDevMode,
  LOCALE_ID,
  provideAppInitializer,
  provideZonelessChangeDetection,
} from '@angular/core';
import { provideGarudaNG } from '@garudalinux/core';
import { TranslocoHttpLoader } from './transloco-loader';
import { provideTransloco } from '@jsverse/transloco';
import { ConfirmationService } from 'primeng/api';
import { LoadingInterceptor } from './components/loading-indicator/loading-indicator.interceptor';
import { ConfigService } from './components/config/config.service';
import { LanguageManagerService } from './components/language-manager/language-manager.service';
import { initToolbox } from './app.init';
import { ThemeService } from './components/theme-service/theme-service';
import { PackagesService } from './components/packages/packages.service';
import { NotificationService } from './components/notification/notification.service';
import { TaskManagerService } from './components/task-manager/task-manager.service';
import { OsInteractService } from './components/task-manager/os-interact.service';
import { SystemStatusService } from './components/system-status/system-status.service';
import { KernelsService } from './components/kernels/kernels.service';
import { LanguagePacksService } from './components/language-packs/language-packs.service';
import { LocalesService } from './components/locales/locales.service';
import { GamingService } from './components/gaming/gaming.service';
import { LoadingService } from './components/loading-indicator/loading-indicator.service';
import { SplashService } from './components/splash/splash.service';
import { provideRouter, Router, withHashLocation, withViewTransitions } from '@angular/router';
import { routes } from './routes';
import { CatppuccinAura } from '@garudalinux/themes/catppuccin';
import { AVAILABLE_LANGUAGES } from './constants/i18n';

export const appConfig: ApplicationConfig = {
  providers: [
    ConfigService,
    ConfirmationService,
    GamingService,
    KernelsService,
    LanguageManagerService,
    LanguagePacksService,
    LoadingService,
    LocalesService,
    NotificationService,
    OsInteractService,
    PackagesService,
    SplashService,
    SystemStatusService,
    TaskManagerService,
    ThemeService,
    provideAppInitializer(initToolbox),
    provideGarudaNG(
      { font: 'InterVariable' },
      {
        theme: {
          preset: CatppuccinAura,
          options: {
            darkModeSelector: '.p-dark',
          },
        },
        ripple: true,
        inputStyle: 'outlined',
        overlayOptions: {
          appendTo: 'body',
        },
      },
    ),
    provideHttpClient(withFetch()),
    provideRouter(
      routes,
      withHashLocation(),
      withViewTransitions({
        skipInitialTransition: true,
        onViewTransitionCreated: ({ transition }) => {
          const router = inject(Router);
          try {
            const nav = router.currentNavigation();
            const info = nav?.extras?.info as any;

            if (info?.disableViewTransition) {
              const style = document.createElement('style');
              style.id = 'skip-transition';
              style.textContent = '* { view-transition-name: none !important; }';
              document.head.appendChild(style);

              transition.finished.finally(() => {
                const el = document.getElementById('skip-transition');
                if (el) el.remove();
                document.body.classList.remove('is-transitioning');
              });
            } else {
              transition.finished.finally(() => {
                document.body.classList.remove('is-transitioning');
              });
            }
          } catch {
            // Ignore parse errors, let transition proceed
          }
        },
      }),
    ),
    provideTransloco({
      config: {
        availableLangs: AVAILABLE_LANGUAGES,
        defaultLang: 'en',
        fallbackLang: 'en',
        missingHandler: {
          useFallbackTranslation: true,
        },
        prodMode: !isDevMode(),
        reRenderOnLangChange: true,
      },
      loader: TranslocoHttpLoader,
    }),
    provideZonelessChangeDetection(),
    { provide: LOCALE_ID, useValue: 'en-GB' },
    { provide: HTTP_INTERCEPTORS, useValue: [LoadingInterceptor], multi: true },
  ],
};
