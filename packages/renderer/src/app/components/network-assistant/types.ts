export interface NetworkDriver {
  name: string;
  status: 'loaded' | 'unloaded' | 'blacklisted' | 'blacklisted-broadcom' | 'blacklisted-r8168';
  category: 'ethernet' | 'wireless' | 'other';
}

export interface HardwareInfo {
  type: 'ethernet' | 'wireless' | 'bluetooth' | 'other';
  description: string;
  raw: string;
}

export interface NetworkStatus {
  routerIp: string;
  externalIp: string;
  interface: string;
  wifiEnabled: boolean;
  wifiAvailable: boolean;
  btEnabled: boolean;
  btAvailable: boolean;
  airplaneEnabled: boolean;
  gpsEnabled: boolean;
  nmActive: boolean;
  nmInstalled: boolean;
  connmanActive: boolean;
  connmanInstalled: boolean;
  bluetoothActive: boolean;
  hasInternet: boolean;
}

export interface PingOptions {
  host: string;
  count: number;
  timeout: number;
}

export interface TracerouteOptions {
  host: string;
  maxHops: number;
}
