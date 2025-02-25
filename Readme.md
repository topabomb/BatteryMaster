# BatteryMaster

The Battery Master is a small tool developed using Rust. It allows you to monitor the charging and discharging power of your battery in real-time from the taskbar. You can also view various critical battery information in real-time. More features are currently under development, so stay tuned!

[Download](https://github.com/topabomb/BatteryMonitor/releases/)

# History

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

### Monitor

![screenshot](/screenshot/monitor.png "Monitor")

### Power limit

![screenshot](/screenshot/power.png "Power limit")

### Settings

![screenshot](/screenshot/setting.png "Settings")
