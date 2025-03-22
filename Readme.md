# BatteryMaster

Battery Master is a small tool developed using Rust. It can monitor the usage of a laptop battery, including its health, wear rate, design capacity, and more, allowing you to track the battery's charge and discharge levels in real-time from the taskbar. You can also view various critical system and battery information in real-time. More features are currently under development, so stay tuned!

[Download](https://github.com/topabomb/BatteryMonitor/releases/)

## dependencies

Tauri relies on the VC runtime library and WebView runtime. The latest version of Windows 11 should already have these built-in. If not, please install them manually.

- Microsoft Visual C++ 2015 - 2022 Redistributable
  x86: (https://aka.ms/vs/17/release/vc_redist.x86.exe)
  x64: (https://aka.ms/vs/17/release/vc_redist.x64.exe)
- Microsoft Edge WebView2
  (https://developer.microsoft.com/zh-cn/microsoft-edge/webview2#download)

# History

## 0.1.3

Add a historical record function for laptop battery usage to track changes in battery health, charge/discharge status, charge/discharge power, and battery wear; while also updating the monitoring page.

## 0.1.2

Added processor power viewing and limiting features, now supporting locking the processor power consumption limit.

## 0.1.0

The first version implements the following features: monitors key battery information (such as charge, discharge power, battery lifespan, etc.), minimizes to the system tray, allows changing key operational configurations, and starts with the operating system.

# Origin

I spent a long time searching for battery information monitoring software available for Windows. While HWiNFO and LibreHardwareMonitor can monitor a large amount of hardware, neither can consistently lock to the taskbar in the front. BatteryinfoView doesn't show up on the taskbar at all. I'm not criticizing these tools, as my knowledge of them only goes up to February 23, 2025, and these issues may have been resolved in newer versions. Additionally, in order to extend battery life when using my laptop, I wanted to set processor power limits to reduce battery consumption. So, I decided to develop the software myself. To practice Rust development, I forced myself to use Rust + Tauri for this project, and it needs to have at least the following features

- Display charging/discharging power or CPU usage on the taskbar. The reason for not displaying the battery percentage is that it is already provided by Windows by default.
- Ability to view battery health; ability to record various battery information history for tracking usage habits.
- For my AMD HX370 (Asus TUF Air14) laptop, the ability to view power limits, and ideally, to modify them myself.

# Screenshots

### Windows tray

![screenshot](/screenshot/tray.png "Windows tray")

### History

![screenshot](/screenshot/history.png "Battery history")

### Monitor

![screenshot](/screenshot/monitor.png "Battery Monitor")

### CPU Power limit

![screenshot](/screenshot/power.png "CPU Power limit")

### Settings

![screenshot](/screenshot/setting.png "Settings")
