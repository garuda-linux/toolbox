import { effect, type ElementRef, inject, type Renderer2, Service, signal } from '@angular/core';
import { ConfigService } from '../config/config.service';
import { Logger } from '../../logging/logging';

@Service()
export class WallpaperService {
  private prevBlurStrength: null | number = null;
  private readonly configService = inject(ConfigService);
  private readonly logger = Logger.getInstance();

  // Signals to track renderer and element references
  private readonly renderer = signal<Renderer2 | null>(null);
  private readonly elementRef = signal<ElementRef | null>(null);

  constructor() {
    effect(() => {
      const settings = this.configService.settings();
      const renderer = this.renderer();
      const el = this.elementRef();

      if (renderer && el) {
        this.logger.debug('WallpaperService effect triggered and renderer and elementRef are set');
        this.loadWallpaperExecutor(el, renderer, settings.background);
        this.applyBgContain(el, renderer, settings.backgroundFit);
        this.applyBgBlur(el, renderer, settings.blurBackground);
      }
    });
  }

  /**
   * Initialize the service with renderer and element references
   * @param renderer Renderer2 instance for DOM manipulation
   * @param el ElementRef pointing to the target element (usually body)
   */
  initialize(renderer: Renderer2, el: ElementRef): void {
    this.renderer.set(renderer);
    this.elementRef.set(el);
  }

  /**
   * Loads the wallpaper based on the provided wallpaper type.
   * @param wallpaper The type of wallpaper to load. Can be 'custom', 'none', or a URL.
   * @param renderer Renderer2 to manipulate the DOM.
   * @param el ElementRef to the body element.
   * @param url Optional URL for the custom wallpaper, if applicable.
   */
  loadWallpaper(wallpaper: string, renderer: Renderer2, el: ElementRef, url?: string): void {
    if (wallpaper === 'custom') {
      this.loadWallpaperExecutor(el, renderer, url ?? '');
    } else if (wallpaper === 'none') {
      this.loadWallpaperExecutor(el, renderer, null);
    } else if (wallpaper !== '') {
      this.loadWallpaperExecutor(el, renderer, wallpaper);
    }
  }

  /**
   * Loads a new startpage background.
   * @param el ElementRef to the body element.
   * @param renderer Renderer2 to the body element.
   * @param wallpaper URL to the wallpaper to load.
   */
  loadWallpaperExecutor(el: ElementRef, renderer: Renderer2, wallpaper: string | null): void {
    if (wallpaper === null) {
      this.logger.debug('Removing background image from body');
      renderer.removeStyle(el.nativeElement.ownerDocument.body, 'backgroundImage');
    } else {
      this.logger.debug(`Setting background image to body`);
      renderer.setStyle(el.nativeElement.ownerDocument.body, 'background-image', `url(${wallpaper})`);
    }
  }

  /**
   * Apply wallpaper style to the body element.
   * @param kind Kind of wallpaper style to apply.
   * @param value Value to apply.
   * @param renderer Renderer2 to the origin element.
   * @param el ElementRef to the origin element.
   */
  applyWallpaperStyle(kind: string, value: boolean, renderer: Renderer2, el: ElementRef): void {
    switch (kind) {
      case 'backgroundFit':
        this.applyBgContain(el, renderer, value);
        break;
      case 'blurBackground':
        this.applyBgBlur(el, renderer, value);
        break;
      case 'blurStrength':
        if (!this.configService.settings().blurBackground) {
          return;
        }
        this.applyBgBlur(el, renderer, value);
        break;
      default:
        this.applyBgContain(el, renderer, false);
        this.applyBgBlur(el, renderer, false);
        break;
    }
  }

  /**
   * Apply background blur style to the body element.
   * @param el ElementRef to the body element.
   * @param renderer Renderer2 to the body element.
   * @param apply Whether to apply the style or not.
   */
  applyBgBlur(el: ElementRef, renderer: Renderer2, apply = true): void {
    const strength = this.configService.settings().blurStrength;
    if (apply) {
      if (this.prevBlurStrength) {
        this.logger.debug(`Removing previous blur strength class: ${this.prevBlurStrength}`);
        renderer.removeClass(el.nativeElement.ownerDocument.body, `background-blurred-${this.prevBlurStrength}`);
      }
      this.logger.debug(`Applying background blur style to body with strength: ${strength}`);
      this.prevBlurStrength = strength;
      renderer.addClass(el.nativeElement.ownerDocument.body, `background-blurred-${strength}`);
    } else {
      this.logger.debug(`Removing background blur style from body with strength: ${strength}`);
      renderer.removeClass(el.nativeElement.ownerDocument.body, `background-blurred-${strength}`);
    }
  }

  /**
   * Apply background contain style to the body element.
   * @param el ElementRef to the body element.
   * @param renderer Renderer2 to the body element.
   * @param apply Whether to apply the style or not.
   */
  applyBgContain(el: ElementRef, renderer: Renderer2, apply = true): void {
    if (apply) {
      this.logger.debug('Applying background contain style to body');
      renderer.addClass(el.nativeElement.ownerDocument.body, 'bg-contain');
    } else {
      this.logger.debug('Removing background contain style from body');
      renderer.removeClass(el.nativeElement.ownerDocument.body, 'bg-contain');
    }
  }
}
