export type ConfigType = 'file' | 'regex';

export interface RegexConfig {
  file: string;
  pattern: string;
  enableReplacement: string;
  disableReplacement: string;
}

export interface ConfigEntry {
  key: string;
  name: string;
  type: ConfigType;
  sudo?: boolean;
  path?: string;
  content?: string;
  regex?: RegexConfig;
}

export const CONFIGS: ConfigEntry[] = [
  {
    key: 'audioNoSuspend',
    name: 'Audio No-Suspend',
    type: 'file',
    path: '.config/wireplumber/wireplumber.conf.d/disable-suspension.conf',
    content: `monitor.alsa.rules = [
  {
    matches = [
      {
        node.name = "~alsa_input.*"
      },
      {
        node.name = "~alsa_output.*"
      }
    ]
    actions = {
      update-props = {
        session.suspend-timeout-seconds = 0
      }
    }
  }
]
monitor.bluez.rules = [
  {
    matches = [
      {
        node.name = "~bluez_input.*"
      },
      {
        node.name = "~bluez_output.*"
      }
    ]
    actions = {
      update-props = {
        session.suspend-timeout-seconds = 0
      }
    }
  }
]
`,
  },
  {
    key: 'kdeBorderlessMaximize',
    name: 'KDE Borderless Maximize',
    type: 'regex',
    regex: {
      file: '.config/kwinrc',
      pattern: '^\\s*BorderlessMaximizedWindows\\s*=.*$',
      enableReplacement: 'BorderlessMaximizedWindows=true',
      disableReplacement: 'BorderlessMaximizedWindows=false',
    },
  },
];
