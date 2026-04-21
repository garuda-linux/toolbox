import { computed, effect, inject, Injectable, signal } from '@angular/core';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { HardwareInfo, NetworkDriver, NetworkStatus, PingOptions, TracerouteOptions } from './types';

@Injectable({
  providedIn: 'root',
})
export class NetworkAssistantService {
  private readonly taskManager = inject(TaskManagerService);

  private quoteForShell(value: string): string {
    return `'${value.replaceAll("'", `'"'"'`)}'`;
  }

  private async runDirectPrivileged(script: string): Promise<void> {
    const command = `pkexec bash -lc ${this.quoteForShell(script)}`;
    const result = await this.taskManager.executeAndWaitBash(command);
    if (result.code !== 0) {
      throw new Error((result.stderr || result.stdout || 'Privileged command failed').trim());
    }
  }

  private async runDirect(script: string): Promise<void> {
    const command = `bash -lc ${this.quoteForShell(script)}`;
    const result = await this.taskManager.executeAndWaitBash(command);
    if (result.code !== 0) {
      throw new Error((result.stderr || result.stdout || 'Command failed').trim());
    }
  }

  readonly status = signal<NetworkStatus | null>(null);
  readonly drivers = signal<NetworkDriver[]>([]);
  readonly hardware = signal<HardwareInfo[]>([]);
  readonly loading = signal(true);

  readonly pingOptions = signal<PingOptions>({ host: 'google.com', count: 4, timeout: 5 });
  readonly tracerouteOptions = signal<TracerouteOptions>({ host: 'google.com', maxHops: 30 });

  readonly ethernetHardware = computed(() => this.hardware().filter((h) => h.type === 'ethernet'));
  readonly wirelessHardware = computed(() => this.hardware().filter((h) => h.type === 'wireless'));
  readonly bluetoothHardware = computed(() => this.hardware().filter((h) => h.type === 'bluetooth'));

  constructor() {
    effect(() => this.refreshStatus());
  }

  async refreshStatus(): Promise<void> {
    this.loading.set(true);
    try {
      const [status, drivers, hw] = await Promise.all([this.getStatus(), this.getDrivers(), this.getHardware()]);
      this.status.set(status);
      this.drivers.set(drivers);
      this.hardware.set(hw);
    } finally {
      this.loading.set(false);
    }
  }

  private async getStatus(): Promise<NetworkStatus> {
    const [
      routerIp,
      externalIp,
      iface,
      wifi,
      wifiAvail,
      bt,
      btAvail,
      airplane,
      gps,
      nm,
      nmInstalled,
      connman,
      connmanInstalled,
      bluetooth,
      internet,
    ] = await Promise.all([
      this.getRouterIp(),
      this.getExternalIp(),
      this.getInterface(),
      this.checkWifi(),
      this.checkWifiAvailable(),
      this.checkBt(),
      this.checkBtAvailable(),
      this.checkAirplane(),
      this.checkGps(),
      this.checkServiceActive('NetworkManager'),
      this.checkPackageInstalled('networkmanager'),
      this.checkServiceActive('connman'),
      this.checkPackageInstalled('connman'),
      this.checkServiceActive('bluetooth'),
      this.checkInternet(),
    ]);

    return {
      routerIp,
      externalIp,
      interface: iface,
      wifiEnabled: wifi,
      wifiAvailable: wifiAvail,
      btEnabled: bt,
      btAvailable: btAvail,
      airplaneEnabled: airplane,
      gpsEnabled: gps,
      nmActive: nm,
      nmInstalled,
      connmanActive: connman,
      connmanInstalled,
      bluetoothActive: bluetooth,
      hasInternet: internet,
    };
  }

  private async getRouterIp(): Promise<string> {
    const res = await this.taskManager.executeAndWaitBash("ip route | grep default | awk '{print $3}' | head -n1");
    return res.stdout.trim() || 'N/A';
  }

  private async getExternalIp(): Promise<string> {
    const res = await this.taskManager.executeAndWaitBash("curl -s https://api.ipify.org || echo 'N/A'");
    return res.stdout.trim();
  }

  private async getInterface(): Promise<string> {
    const res = await this.taskManager.executeAndWaitBash("ip route | grep default | awk '{print $5}' | head -n1");
    return res.stdout.trim() || 'N/A';
  }

  private async checkInternet(): Promise<boolean> {
    const res = await this.taskManager.executeAndWaitBash(
      'ping -c 1 -W 2 8.8.8.8 > /dev/null 2>&1 && echo "yes" || echo "no"',
    );
    return res.stdout.trim() === 'yes';
  }

  private async checkWifi(): Promise<boolean> {
    const res = await this.taskManager.executeAndWaitBash(
      "rfkill list wifi | grep -A2 'Soft blocked' | tail -1 | awk '{print $NF}'",
    );
    return res.stdout.trim() !== 'yes';
  }

  private async checkWifiAvailable(): Promise<boolean> {
    const res = await this.taskManager.executeAndWaitBash(
      "lspci | grep -i 'wireless\\|wifi\\|wlan' || lsusb | grep -i 'wireless\\|wifi\\|wlan' || echo 'none'",
    );
    return !res.stdout.trim().includes('none');
  }

  private async checkBt(): Promise<boolean> {
    const res = await this.taskManager.executeAndWaitBash(
      "rfkill list bluetooth | grep -A2 'Soft blocked' | tail -1 | awk '{print $NF}'",
    );
    return res.stdout.trim() !== 'yes';
  }

  private async checkBtAvailable(): Promise<boolean> {
    const res = await this.taskManager.executeAndWaitBash(
      "rfkill list bluetooth >/dev/null 2>&1 || bluetoothctl show >/dev/null 2>&1 || lspci | grep -i bluetooth || lsusb | grep -i bluetooth || echo 'none'",
    );
    return !res.stdout.trim().includes('none');
  }

  private async checkPackageInstalled(pkg: string): Promise<boolean> {
    const res = await this.taskManager.executeAndWaitBash(`pacman -Q ${this.quoteForShell(pkg)} >/dev/null 2>&1`);
    return res.code === 0;
  }

  private async checkAirplane(): Promise<boolean> {
    const res = await this.taskManager.executeAndWaitBash(
      'rfkill list all 2>/dev/null | awk \'/^[0-9]+:/ { line=$0 } /Soft blocked:/ { if (line ~ /Bluetooth|Wireless LAN|WLAN|Wi-Fi|wifi/) { total += 1; if ($3 == "yes") blocked += 1 } } END { if (total > 0 && blocked == total) print "yes"; else print "no" }\'',
    );
    return res.stdout.trim() === 'yes';
  }

  private async checkGps(): Promise<boolean> {
    const res = await this.taskManager.executeAndWaitBash('systemctl is-enabled geoclue 2>/dev/null');
    return res.stdout.trim() === 'enabled';
  }

  private async checkServiceActive(service: string): Promise<boolean> {
    const res = await this.taskManager.executeAndWaitBash(`systemctl is-active ${service}`);
    return res.stdout.trim() === 'active';
  }

  async getDrivers(): Promise<NetworkDriver[]> {
    const [loaded, netModules, blacklisted, broadcomBlacklisted, r8168Blacklisted] = await Promise.all([
      this.getLoadedModules(),
      this.getNetworkModules(),
      this.getBlacklistedModules('/etc/modprobe.d/blacklist.conf'),
      this.getBlacklistedModules('/usr/lib/modprobe.d/broadcom-wl.conf'),
      this.getBlacklistedModules('/usr/lib/modprobe.d/r8168.conf'),
    ]);

    const drivers: NetworkDriver[] = [];

    loaded.forEach((m) => {
      if (netModules.includes(m)) {
        const isEthernet = ['r8169', 'r8168', 'e1000', 'e1000e', 'igb', 'ixgbe', 'atlantic'].some((eth) =>
          m.toLowerCase().includes(eth),
        );
        const isWireless = ['wl', 'b43', 'brcm', 'ath', 'iwl', 'mt76', 'rtl8', 'rt2', 'rt5'].some((w) =>
          m.toLowerCase().includes(w),
        );
        drivers.push({
          name: m,
          status: 'loaded',
          category: isEthernet ? 'ethernet' : isWireless ? 'wireless' : 'other',
        });
      }
    });

    netModules.forEach((m) => {
      if (
        !loaded.includes(m) &&
        !blacklisted.includes(m) &&
        !broadcomBlacklisted.includes(m) &&
        !r8168Blacklisted.includes(m)
      ) {
        const isEthernet = ['r8169', 'r8168', 'e1000', 'e1000e', 'igb', 'ixgbe', 'atlantic'].some((eth) =>
          m.toLowerCase().includes(eth),
        );
        const isWireless = ['wl', 'b43', 'brcm', 'ath', 'iwl', 'mt76', 'rtl8', 'rt2', 'rt5'].some((w) =>
          m.toLowerCase().includes(w),
        );
        drivers.push({
          name: m,
          status: 'unloaded',
          category: isEthernet ? 'ethernet' : isWireless ? 'wireless' : 'other',
        });
      }
    });

    blacklisted.forEach((m) => {
      drivers.push({ name: m, status: 'blacklisted', category: 'other' });
    });

    broadcomBlacklisted.forEach((m) => {
      drivers.push({ name: m, status: 'blacklisted-broadcom', category: 'wireless' });
    });

    r8168Blacklisted.forEach((m) => {
      drivers.push({ name: m, status: 'blacklisted-r8168', category: 'ethernet' });
    });

    return drivers;
  }

  private async getLoadedModules(): Promise<string[]> {
    const res = await this.taskManager.executeAndWaitBash("lsmod | awk '{print $1}' | tail -n +2");
    return res.stdout
      .trim()
      .split('\n')
      .filter((m) => m.length > 0);
  }

  private async getNetworkModules(): Promise<string[]> {
    const res = await this.taskManager.executeAndWaitBash(
      "find /lib/modules/$(uname -r)/kernel/drivers/net -name '*.ko*' 2>/dev/null | sed 's/.*\\///;s/\\.ko.*//'",
    );
    const modules = res.stdout
      .trim()
      .split('\n')
      .filter((m) => m.length > 0);
    return [...new Set([...modules, 'ndiswrapper', 'atl2', 'wl'])];
  }

  private async getBlacklistedModules(path: string): Promise<string[]> {
    const res = await this.taskManager.executeAndWaitBash(`grep '^blacklist' '${path}' 2>/dev/null | awk '{print $2}'`);
    return res.stdout
      .trim()
      .split('\n')
      .filter((m) => m.length > 0);
  }

  async getHardware(): Promise<HardwareInfo[]> {
    const hardware: HardwareInfo[] = [];

    const [pciNet, usbNet] = await Promise.all([
      this.taskManager.executeAndWaitBash('lspci -nn | grep -i net'),
      this.taskManager.executeAndWaitBash('lsusb | grep -i -E "net|ethernet|wireless|wifi|bluetooth|bt"'),
    ]);

    const parsePciLine = (line: string): HardwareInfo | null => {
      if (line.includes('Ethernet controller')) {
        return {
          type: 'ethernet',
          description: line.replace(/Ethernet controller .[\da-f]{4}:/, '').trim(),
          raw: line,
        };
      }
      if (line.includes('Network controller')) {
        return {
          type: 'wireless',
          description: line.replace(/Network controller .[\da-f]{4}:/, '').trim(),
          raw: line,
        };
      }
      if (line.includes('Bluetooth')) {
        return { type: 'bluetooth', description: line.trim(), raw: line };
      }
      return null;
    };

    pciNet.stdout.split('\n').forEach((line) => {
      const hw = parsePciLine(line);
      if (hw) hardware.push(hw);
    });

    const parseUsbLine = (line: string): HardwareInfo | null => {
      if (line.includes('Ethernet') || line.includes('RNDIS')) {
        return {
          type: 'ethernet',
          description: line.replace('Ethernet controller:', '').trim(),
          raw: line,
        };
      }
      if (line.includes('Wireless') || line.includes('WiFi') || line.includes('WLAN')) {
        return {
          type: 'wireless',
          description: line.replace('Network controller:', '').trim(),
          raw: line,
        };
      }
      if (line.includes('Bluetooth')) {
        return { type: 'bluetooth', description: line.trim(), raw: line };
      }
      return null;
    };

    usbNet.stdout.split('\n').forEach((line) => {
      const hw = parseUsbLine(line);
      if (hw) hardware.push(hw);
    });

    return hardware;
  }

  async toggleWifi(enable: boolean): Promise<void> {
    const script = enable ? 'rfkill unblock wifi' : 'rfkill block wifi';
    await this.runDirect(script);
    await this.refreshStatus();
  }

  async toggleBt(enable: boolean): Promise<void> {
    const script = enable ? 'rfkill unblock bluetooth' : 'rfkill block bluetooth';
    await this.runDirect(script);
    await this.refreshStatus();
  }

  async toggleAirplane(enable: boolean): Promise<void> {
    const script = enable ? 'rfkill block all' : 'rfkill unblock all';
    await this.runDirect(script);
    await this.refreshStatus();
  }

  async toggleGps(enable: boolean): Promise<void> {
    const script = enable ? 'systemctl enable --now geoclue' : 'systemctl disable --now geoclue';
    await this.runDirectPrivileged(script);
    await this.refreshStatus();
  }

  async enableService(service: string): Promise<void> {
    const script = `systemctl enable --now ${service}`;
    const task = this.taskManager.createTask(
      0,
      `network-service-${service}`,
      true,
      `networkAssistant.enable${service}`,
      'pi pi-play',
      script,
    );
    this.taskManager.scheduleTask(task);
    await this.taskManager.executeTask(task);
    await this.refreshStatus();
  }

  async loadDriver(name: string): Promise<void> {
    const script = `systemctl stop NetworkManager; modprobe cfg80211; modprobe ${name}; pkill wpa_supplicant; systemctl start NetworkManager`;
    const task = this.taskManager.createTask(
      0,
      `network-driver-load-${name}`,
      true,
      `networkAssistant.loadDriver`,
      'pi pi-play',
      script,
    );
    this.taskManager.scheduleTask(task);
    await this.taskManager.executeTask(task);
    await this.refreshStatus();
  }

  async unloadDriver(name: string): Promise<void> {
    const script = `systemctl stop NetworkManager; modprobe -r ${name}; systemctl start NetworkManager`;
    const task = this.taskManager.createTask(
      0,
      `network-driver-unload-${name}`,
      true,
      `networkAssistant.unloadDriver`,
      'pi pi-stop',
      script,
    );
    this.taskManager.scheduleTask(task);
    await this.taskManager.executeTask(task);
    await this.refreshStatus();
  }

  async blacklistDriver(name: string, blacklist: boolean): Promise<void> {
    let script: string;
    if (blacklist) {
      const isBroadcom = ['wl', 'b43', 'brcm', 'brmf'].some((b) => name.includes(b));
      const isR8168 = name.includes('r8168');
      const confPath = isBroadcom
        ? '/usr/lib/modprobe.d/broadcom-wl.conf'
        : isR8168
          ? '/usr/lib/modprobe.d/r8168.conf'
          : '/etc/modprobe.d/blacklist.conf';
      script = `echo "blacklist ${name}" >> "${confPath}"; modprobe -r ${name} 2>/dev/null || true`;
    } else {
      script = `sed -i '/blacklist ${name}/d' /etc/modprobe.d/blacklist.conf 2>/dev/null; sed -i '/blacklist ${name}/d' /usr/lib/modprobe.d/broadcom-wl.conf 2>/dev/null; sed -i '/blacklist ${name}/d' /usr/lib/modprobe.d/r8168.conf 2>/dev/null`;
    }
    const task = this.taskManager.createTask(
      0,
      `network-driver-blacklist-${name}`,
      true,
      blacklist ? 'networkAssistant.blacklist' : 'networkAssistant.unblacklist',
      'pi pi-lock',
      script,
    );
    this.taskManager.scheduleTask(task);
    await this.taskManager.executeTask(task);
    await this.refreshStatus();
  }

  async installDriver(name: string): Promise<void> {
    const script = `pacman -S --noconfirm --needed ${name}`;
    const task = this.taskManager.createTask(
      0,
      `network-driver-install-${name}`,
      true,
      `networkAssistant.installDriver`,
      'pi pi-download',
      script,
    );
    this.taskManager.scheduleTask(task);
    await this.taskManager.executeTask(task);
    await this.refreshStatus();
  }

  async openWifiHotspot(): Promise<void> {
    const status = await this.taskManager.executeAndWaitBash('pacman -Qq wihotspot');
    if (status.code !== 0) {
      await this.runDirectPrivileged('pacman -S --noconfirm --needed wihotspot');
    }

    const runResult = await this.taskManager.executeAndWaitBash(
      'nohup /usr/bin/wihotspot >/dev/null 2>&1 & disown || true',
    );
    if (runResult.code !== 0) {
      throw new Error((runResult.stderr || runResult.stdout || 'Failed to start wihotspot').trim());
    }
  }

  copyToClipboard(text: string): void {
    void navigator.clipboard.writeText(text);
  }

  copyAllToClipboard(lines: string[]): void {
    void navigator.clipboard.writeText(lines.join('\n'));
  }
}
