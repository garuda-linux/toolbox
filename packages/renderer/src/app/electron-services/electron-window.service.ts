import { Service } from '@angular/core';
import {
  windowClose,
  windowFocus,
  windowGetPosition,
  windowGetSize,
  windowGetTitle,
  windowHide,
  windowIsMaximized,
  windowIsMinimized,
  windowIsVisible,
  windowMaximize,
  windowMinimize,
  windowRequestClose,
  windowSetPosition,
  windowSetSize,
  windowSetTitle,
  windowShow,
} from './electron-api-utils.js';

@Service()
export class ElectronWindowService {
  async close(): Promise<void> {
    windowClose();
  }

  async requestClose(): Promise<void> {
    windowRequestClose();
  }

  async minimize(): Promise<void> {
    windowMinimize();
  }

  async maximize(): Promise<void> {
    windowMaximize();
  }

  async hide(): Promise<void> {
    windowHide();
  }

  async show(): Promise<void> {
    windowShow();
  }

  async focus(): Promise<void> {
    windowFocus();
  }

  async isMaximized(): Promise<boolean> {
    return windowIsMaximized();
  }

  async isMinimized(): Promise<boolean> {
    return windowIsMinimized();
  }

  async isVisible(): Promise<boolean> {
    return windowIsVisible();
  }

  async setTitle(title: string): Promise<void> {
    windowSetTitle(title);
  }

  async getTitle(): Promise<string> {
    return windowGetTitle();
  }

  async setSize(width: number, height: number): Promise<void> {
    windowSetSize(width, height);
  }

  async getSize(): Promise<number[]> {
    const size = windowGetSize();
    return [size.width, size.height];
  }

  async setPosition(x: number, y: number): Promise<void> {
    windowSetPosition(x, y);
  }

  async getPosition(): Promise<number[]> {
    const position = windowGetPosition();
    return [position.x, position.y];
  }
}
