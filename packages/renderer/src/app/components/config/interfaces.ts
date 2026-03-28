import type { LogLevel } from '../../logging/interfaces';
import type { AppTheme } from '../../theme';

export interface AppSettings {
  activeTheme: AppTheme;
  autoRefresh: boolean;
  autoStart: boolean;
  background: string | null;
  backgroundFit: boolean;
  blurBackground: boolean;
  blurStrength: number;
  copyDiagnostics: boolean;
  customDesign: string | null;
  darkMode: boolean;
  language: string;
  logLevel: LogLevel;
  showMainLinks: boolean;
  systemdUserContext: boolean;
  [key: string]: any;
}

export interface AppState {
  availablePkgs: Map<string, boolean>;
  codeName: string;
  designerActive: boolean;
  desktopEnvironment: DesktopEnvironment;
  hostname: string;
  isLiveSystem: boolean | undefined;
  isMaximized: boolean;
  kernel: string;
  locale: string;
  rebootPending: boolean;
  user: string;
  userHome: string;
  [key: string]: any;
}

export type DesktopEnvironment =
  | 'GNOME'
  | 'GNOME-Flashback'
  | 'KDE'
  | 'LXDE'
  | 'LXQt'
  | 'MATE'
  | 'TDE'
  | 'Unity'
  | 'XFCE'
  | 'EDGE'
  | 'Cinnamon'
  | 'Pantheon'
  | 'DDE'
  | 'Hyprland';
