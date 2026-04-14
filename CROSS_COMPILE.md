# Wheel IME - ARM Linux 交叉编译指南

## 目标平台

- **CPU**: 全志 T113-S3 (Cortex-A7, ARMv7-A, hard-float)
- **系统**: Tina Linux (OpenWrt 系, musl libc)
- **显示引擎**: Allwinner DE2.0
- **显示**: Linux Framebuffer (/dev/fb0) 或 DRM/KMS (/dev/dri/)
- **输入**: libinput (触摸屏/鼠标/键盘)

### 当前板子设备节点状态

```
/dev/fb0     ✅ 存在 — fbdev 可用
/dev/disp    ✅ 存在 — 全志 sunxi-disp 驱动
/dev/dri/    ❌ 不存在 — DRM/KMS 未启用
/dev/input/  ✅ 存在 — 输入设备可用
```

当前内核使用全志 `sunxi-disp` fbdev 方案，未启用 DRM/KMS。可先用 fbdev 模式运行，后续启用 DRM/KMS 消除画面撕裂。

## 目录结构

```
ime.rust/
├── Cargo.toml              # feature 配置 (desktop/embedded)
├── .cargo/config.toml       # ARM 交叉编译链接器配置
├── src/                     # Rust 源码
├── ui/                      # Slint UI 文件
└── CROSS_COMPILE.md         # 本文件
```

## 编译环境要求

### 1. Rust 工具链

```bash
# 安装 ARM 目标
rustup target add armv7-unknown-linux-musleabihf
```

### 2. 交叉编译器

已配置的编译器路径：

```
/home/sclock/bin/arm-musl-6.4/bin/arm-openwrt-linux-muslgnueabi-gcc
```

### 3. 目标系统依赖库

链接阶段需要以下 ARM 版本的共享库（.so）和头文件：

| 库 | 版本建议 | 用途 | 来源 |
|---|---|---|---|
| libinput | >= 1.16 | 触摸/鼠标/键盘输入 | https://gitlab.freedesktop.org/libinput/libinput |
| libxkbcommon | >= 1.0 | 键盘布局处理 | https://github.com/xkbcommon/libxkbcommon |
| libudev | >= 230 | 设备枚举 (libinput 依赖) | systemd 或 eudev |
| libevdev | >= 1.10 | 输入事件 (libinput 依赖) | https://www.freedesktop.org/wiki/Software/libevdev/ |
| libdrm | >= 2.4 | DRM/KMS 显示 | https://gitlab.freedesktop.org/mesa/drm |
| libfontconfig | >= 2.13 | 字体查找 | https://www.freedesktop.org/wiki/Software/fontconfig/ |
| libfreetype | >= 2.10 | 字体渲染 | https://freetype.org/ |
| libpng | >= 1.6 | PNG 解码 (已有) | 系统自带 |
| libz | >= 1.2 | 压缩 (已有) | 系统自带 |

### 4. 编译依赖库 (以 libinput 为例)

在 Tina Linux SDK 中，可以通过以下方式编译：

```bash
# 方法1: 在 Tina SDK 中添加软件包
# 在 target/allwinner/astar-parrot/ 目录下的配置中添加软件包

# 方法2: 手动交叉编译
export CC=/home/sclock/bin/arm-musl-6.4/bin/arm-openwrt-linux-muslgnueabi-gcc
export SYSROOT=/home/sclock/bin/arm-musl-6.4/arm-openwrt-linux-muslgnueabi
export PKG_CONFIG_PATH=$SYSROOT/lib/pkgconfig:$SYSROOT/usr/lib/pkgconfig
export PKG_CONFIG_SYSROOT_DIR=$SYSROOT

# 编译每个库 (以 libinput 为例)
git clone https://gitlab.freedesktop.org/libinput/libinput.git
cd libinput
mkdir build && cd build
meson setup --cross-file=arm-cross.txt ..
ninja && ninja install
```

建议编译顺序（按依赖关系）：

```
1. libevdev     (无依赖)
2. libudev      (来自 eudev, 无额外依赖)
3. libdrm       (无额外依赖)
4. libxkbcommon (依赖 libxml2 或内嵌 expat)
5. libinput     (依赖 libevdev, libudev, libdrm)
6. freetype     (依赖 libz, libpng)
7. fontconfig   (依赖 freetype, libexpat)
```

### 5. 安装库到 sysroot

将编译产物复制到交叉编译器的 sysroot 目录：

```bash
SYSROOT=/home/sclock/bin/arm-musl-6.4/arm-openwrt-linux-muslgnueabi

# 复制头文件
cp -r include/* $SYSROOT/include/

# 复制库文件
cp lib/*.so* $SYSROOT/lib/
cp lib/*.a $SYSROOT/lib/

# 复制 pkg-config 文件
cp lib/pkgconfig/*.pc $SYSROOT/lib/pkgconfig/
```

## 显示模式与画面撕裂

### fbdev 模式 (当前可用)

fbdev 是最简单的显示接口：用户空间通过 `mmap(/dev/fb0)` 直接写显存，数据立刻可见。

- 单缓冲，没有 vsync 同步
- 用户空间写入和 LCD 扫描输出之间没有协调
- 扫描到一半时写入新数据 → 上半帧旧画面、下半帧新画面 → **撕裂**
- 对于输入法这种 UI 更新不频繁的场景，撕裂问题不大

运行方式：
```bash
SLINT_BACKEND_LINUXFB=1 wheel-rust
```

### DRM/KMS 模式 (需重新配置内核)

DRM/KMS 是现代 Linux 显示子系统，支持双缓冲和 vsync 同步：

- 至少双缓冲（Front + Back），画完一整帧再切换
- `drmModePageFlip()` 在 VBlank（帧间间隙）时原子性切换 buffer
- LCD 控制器始终读取完整帧 → **无撕裂**

T113-S3 硬件 (DE2.0) 完全支持，Linux 主线内核有 `sun4i-drm` 驱动。

### 启用 DRM/KMS 的内核配置

```bash
# 在 Tina SDK 中
make kernel_menuconfig
```

需要开启：
```
CONFIG_DRM=y
CONFIG_DRM_SUN4I=y
CONFIG_DRM_SUN4I_BACKEND=y       # DE2.0 backend
CONFIG_DRM_SUN4I_HDMI=y           # 如用 HDMI 输出
CONFIG_DRM_SUN4I_LCD=y            # LCD 输出 (RGB/LVDS)
CONFIG_DRM_PANEL=y
CONFIG_DRM_PANEL_SIMPLE=y
CONFIG_DRM_KMS_HELPER=y
CONFIG_DRM_KMS_FB_HELPER=y
CONFIG_FB=y
CONFIG_FB_SIMPLE=y                # 可选，提供 /dev/fb0 兼容
```

需要关闭（避免与 DRM 冲突）：
```
# CONFIG_FB_SUNXI  (全志旧 fbdev 驱动)
```

设备树 (dts) 中确认 display 节点已启用：
```dts
&de {
    status = "okay";
};
&tcon_lcd0 {
    status = "okay";
};
```

启用后验证：
```bash
ls /dev/dri/card0                    # 存在 = DRM/KMS 可用
cat /sys/class/drm/card0/status      # 查看连接状态
```

运行方式：
```bash
SLINT_BACKEND=linuxkms wheel-rust
```

## 编译命令

### Ubuntu 桌面版 (开发调试)

```bash
cargo build
cargo run
```

### ARM 嵌入式版

```bash
cargo build --no-default-features --features embedded \
    --target armv7-unknown-linux-musleabihf \
    --release
```

产物路径：
```
target/armv7-unknown-linux-musleabihf/release/wheel-rust
```

## 部署到 T113-S3

### 1. 复制可执行文件

```bash
scp target/armv7-unknown-linux-musleabihf/release/wheel-rust root@192.168.x.x:/usr/bin/
```

### 2. 确保板子上有运行时库

```bash
# 检查板子上的库
ls /usr/lib/libinput*
ls /usr/lib/libxkbcommon*
ls /usr/lib/libdrm*
```

### 3. 准备字体文件

板子上需要中文字体才能正常显示：

```bash
mkdir -p /usr/share/fonts/truetype/
scp /usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc root@192.168.x.x:/usr/share/fonts/truetype/
```

推荐字体（任选其一）：
- Noto Sans CJK SC
- WenQuanYi Micro Hei
- Source Han Sans SC

### 4. 运行

```bash
# DRM/KMS 模式 (推荐，支持双缓冲)
SLINT_BACKEND=linuxkms wheel-rust

# Framebuffer 模式 (兼容性好)
SLINT_BACKEND_LINUXFB=1 wheel-rust

# 指定缩放 (800x480 屏幕)
SLINT_SCALE_FACTOR=1.0 wheel-rust

# 指定旋转
SLINT_KMS_ROTATION=0 wheel-rust
```

## 常见问题

### Q: 链接报错 "cannot find -lxkbcommon" 等

A: sysroot 中缺少对应的 .so 文件，按上方 "编译依赖库" 章节补充。

### Q: 板子上运行报错找不到 .so

A: 编译出的二进制是静态链接 musl 的，但 libinput 等是动态库。确保板子 `/usr/lib/` 下有这些 .so 文件。也可尝试传递 `RUSTFLAGS='-C target-feature=+crt-static'` 全静态链接。

### Q: 触摸无反应

A: 检查 `/dev/input/event*` 设备是否存在，以及用户是否有读取权限：
```bash
ls -la /dev/input/
cat /dev/input/event0  # 触摸屏幕看有无输出
```

### Q: 画面撕裂

A: fbdev 模式下单缓冲无 vsync 同步，撕裂是正常现象。输入法 UI 更新不频繁，实际影响较小。如需消除撕裂，需启用 DRM/KMS 内核驱动（见上方"显示模式与画面撕裂"章节）。

### Q: 字体显示为方块

A: 板子上缺少中文字体文件，按上方 "准备字体文件" 章节安装。

### Q: 屏幕分辨率不匹配

A: 通过环境变量调整：
```bash
SLINT_SCALE_FACTOR=1.5 wheel-rust    # 放大
SLINT_KMS_ROTATION=90 wheel-rust      # 旋转
```
