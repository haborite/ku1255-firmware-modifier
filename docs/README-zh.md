[English](README.md) | [日本語](docs/README-ja.md) |  [简体中文](docs/README-zh.md) 

# KU-1255 固件修改器

这是一个用于自定义 **[ThinkPad 指点杆键盘](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)** 固件的简洁 GUI 工具。
你可以重映射键盘上的任意按键——例如，把左下角位置的 `Fn` 与 `Ctrl` 互换。你还可以将 TrackPoint（小红点）的速度提升到**联想驱动设置限制的水平**。

由于所有修改都直接写入键盘**固件**，因此**无需任何系统端配置**。无论连接到哪台设备或哪个操作系统，布局都能保持一致。

![GUI 总览](https://github.com/haborite/ku1255-firmware-modifier/blob/main/docs/gui-overview.png)

---

## 📜 兼容机型

**[Lenovo ThinkPad Compact USB Keyboard with TrackPoint（KU-1255）](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)**

**型号（Part Number）**：
0B47190, 0B47191, 0B47192, 0B47194, 0B47195, 0B47197, 0B47198, 0B47200, 0B47201, 0B47202, 0B47204, 0B47205, 0B47206, 0B47207, 0B47208, 0B47209, 0B47210, 0B47211, 0B47212, 0B47213, 0B47215, 0B47216, 0B47217, 0B47218, 0B47219, 0B47220, 0B47221, 0B47222, 0B47223, 0B47224, 0B47225

## ✅ 系统要求

* 目前应用仅支持 **Windows**。若有需求，可开发 **macOS** 与 **Linux** 版本。
* 要在 Microsoft Windows 上运行，需要安装 Microsoft Visual C++ 可再发行软件包。
* 一旦完成固件写入，键盘在**多数操作系统**上都可直接使用，与安装时所用系统无关。
* 首次运行本应用需要联网，以从联想官网页面下载官方固件安装程序。

## 🚀 下载与运行

1. 前往 [Releases](https://github.com/haborite/ku1255-firmware-modifier/releases/latest) 页面下载 `ku1255-firmware-modifier.zip` 的[最新版本](https://github.com/haborite/ku1255-firmware-modifier/releases/latest)。
2. 解压下载得到的 `.zip` 文件。
3. 运行 `ku1255-firmware-modifier.exe`。
   * 如果看到 “Windows protected your PC / Microsoft Defender SmartScreen prevented an unrecognized app from starting” 的提示，点击 **More info**（更多信息），然后选择 **Run anyway**（仍要运行） 继续。

## 🖥️ 界面总览

1. **Keyboard Selection（键盘选择）**
   选择你的键盘型号。美式布局请选择：`0B47190 (84 keys - ANSI)`。

2. **Language Selection（语言选择）**
   选择偏好的语言。常见的美式布局请选择 `US / English`。

3. **Main Layer（主层）**
   定义默认键位映射。点击任意按键可更改，并在下拉菜单中选定要映射的新按键。

4. **2nd Layer（第二层）**
   定义配合 **Mod** 键使用时的按键行为。

   * 该层**默认禁用**，因为初始状态下主层未映射 Mod 键。
   * **必须在主层与第二层的同一位置同时分配 Mod 键**。

5. **TrackPoint Speed（小红点速度）**
   设置 TrackPoint 速度（默认值：1）。这与联想驱动或操作系统的鼠标设置**无关**。建议**先**在驱动/系统里完成设置，再来修改固件速度。

6. **Load config（加载配置）**
   从 `.json` 文件加载已保存的键位映射。

7. **Save config（保存配置）**
   将当前键位映射保存为 `.json` 文件。

8. **Install firmware（安装固件）**
   将当前配置刷写到键盘。
   开始前请确保键盘已插入。
   安装完成后，**拔下并重新插入**键盘以使更改生效。

## 🔧 示例：交换 Fn 与 Ctrl

1. 点击 **Load config**，打开 `example/Swap-Fn-Ctrl.json`。
2. 在 **Main Layer（主层）** 中确认 `Fn` 与 `Left Ctrl` 已互换。
   （被互换的按键会以**蓝色高亮**。）
3. 点击 **Install firmware**。
4. 固件安装器启动后，点击 **Start**。
5. 安装完成后关闭安装器。
6. 拔下并重新连接键盘，新键位映射即刻生效。

---

# 致谢

本项目所采用的固件二进制分析方法参考了以下讨论帖：

* [https://github.com/lentinj/tp-compact-keyboard/issues/32](https://github.com/lentinj/tp-compact-keyboard/issues/32)

参考的 Usage ID 与名称对照表：

* [https://bsakatu.net/doc/usb-hid-to-scancode/](https://bsakatu.net/doc/usb-hid-to-scancode/)

本应用以可扩展为设计目标，可支持多语言键盘布局。

非常欢迎提交 PR，为你所用语言的键盘添加支持！
