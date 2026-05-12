# OpenWarp macOS 编译与打包指南

> 适用于 Intel (x86_64) Mac，基于 OpenWarp 社区分支编译。

## 环境要求

| 组件 | 要求 |
|------|------|
| macOS | 10.14+ (当前测试: 26.5) |
| 架构 | x86_64 (Intel) 或 arm64 (Apple Silicon) |
| Xcode | 完整版 (非 Command Line Tools) |
| Rust | 通过 rustup 安装 |
| Homebrew | 已安装 |

## 第一步：安装依赖

### 1.1 安装 Rust 工具链

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### 1.2 安装 Xcode 并接受许可

```bash
# 从 App Store 安装 Xcode 后执行：
sudo xcodebuild -license accept
sudo xcodebuild -runFirstLaunch
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

### 1.3 安装 Metal Toolchain

```bash
xcodebuild -downloadComponent MetalToolchain
```

> **重要**：OpenWarp 使用 Metal 渲染器，必须安装 Metal Toolchain。Command Line Tools 不包含此组件。

### 1.4 安装 brew 依赖

```bash
brew install jq clang-format create-dmg pkgconf llvm protobuf
brew install getsentry/tools/sentry-cli multitime powershell
```

### 1.5 安装 cargo 工具

```bash
source "$HOME/.cargo/env"

# cargo-binstall
cargo install cargo-binstall@1.14.3 --locked

# cargo-about
cargo install --locked cargo-about@0.8.4

# wgslfmt (GPU 着色器格式化工具)
cargo install --git https://github.com/wgsl-analyzer/wgsl-analyzer --tag "2025-06-28" wgslfmt

# cargo-nextest
cargo binstall --secure --no-confirm --no-discover-github-token cargo-nextest

# ARM 交叉编译目标
rustup target add aarch64-apple-darwin
```

## 第二步：克隆并编译

### 2.1 克隆仓库

```bash
cd ~
git clone https://github.com/zerx-lab/warp.git openwarp
cd openwarp
```

### 2.2 编译

```bash
source "$HOME/.cargo/env"
cargo build --release
```

编译时间约 15-20 分钟 (Intel Mac)。

### 2.3 验证编译产物

```bash
ls -la target/release/warp-oss
# 应该看到: warp-oss (可执行文件，约 330MB)
```

> **注意**：使用 `warp-oss` 而非 `warp`。`warp` 需要内部私有工具 `warp-channel-config`，外部无法访问。

## 第三步：打包 DMG

### 3.1 生成图标 (首次)

```bash
ICONSET_DIR="app/channels/oss/icon/AppIcon.iconset"
ICON_SRC="app/channels/oss/icon/no-padding"

mkdir -p "$ICONSET_DIR"
sips -z 16 16 "$ICON_SRC/16x16.png" --out "$ICONSET_DIR/icon_16x16.png"
sips -z 32 32 "$ICON_SRC/16x16.png" --out "$ICONSET_DIR/icon_16x16@2x.png"
sips -z 32 32 "$ICON_SRC/32x32.png" --out "$ICONSET_DIR/icon_32x32.png"
sips -z 64 64 "$ICON_SRC/32x32.png" --out "$ICONSET_DIR/icon_32x32@2x.png"
sips -z 128 128 "$ICON_SRC/128x128.png" --out "$ICONSET_DIR/icon_128x128.png"
sips -z 256 256 "$ICON_SRC/128x128.png" --out "$ICONSET_DIR/icon_128x128@2x.png"
sips -z 256 256 "$ICON_SRC/256x256.png" --out "$ICONSET_DIR/icon_256x256.png"
sips -z 512 512 "$ICON_SRC/256x256.png" --out "$ICONSET_DIR/icon_256x256@2x.png"
sips -z 512 512 "$ICON_SRC/512x512.png" --out "$ICONSET_DIR/icon_512x512.png"
cp "$ICON_SRC/512x512.png" "$ICONSET_DIR/icon_512x512@2x.png"

iconutil -c icns "$ICONSET_DIR" -o "app/channels/oss/icon/AppIcon.icns"
```

### 3.2 创建 App 包

```bash
APP_NAME="OpenWarp"
BUILD_DIR="build"
BUNDLE_DIR="$BUILD_DIR/$APP_NAME.app"

rm -rf "$BUILD_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

cp target/release/warp-oss "$BUNDLE_DIR/Contents/MacOS/$APP_NAME"
chmod +x "$BUNDLE_DIR/Contents/MacOS/$APP_NAME"
cp app/channels/oss/icon/AppIcon.icns "$BUNDLE_DIR/Contents/Resources/"
```

### 3.3 创建 Info.plist

```bash
cat > "$BUNDLE_DIR/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleDisplayName</key>
    <string>OpenWarp</string>
    <key>CFBundleExecutable</key>
    <string>OpenWarp</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>dev.openwarp.OpenWarp</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleLocalizations</key>
    <array>
        <string>en</string>
        <string>ja</string>
        <string>zh-CN</string>
    </array>
    <key>CFBundleName</key>
    <string>OpenWarp</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>© 2025 OpenWarp Contributors</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.14</string>
</dict>
</plist>
PLIST
```

### 3.4 创建 DMG

```bash
DMG_DIR="$BUILD_DIR/dmg"
mkdir -p "$DMG_DIR"
cp -R "$BUNDLE_DIR" "$DMG_DIR/"

create-dmg \
    --volname "$APP_NAME" \
    --no-internet-enable \
    --background "app/assets/resources/mac/warp_install_image.png" \
    --icon-size 128 \
    --window-size 700 500 \
    --format UDZO \
    --app-drop-link 550 250 \
    --icon "$APP_NAME.app" 150 250 \
    --filesystem APFS \
    "$BUILD_DIR/$APP_NAME.dmg" \
    "$DMG_DIR"

rm -rf "$DMG_DIR"
```

## 第四步：运行

### 直接运行

```bash
./target/release/warp-oss
```

### 从 DMG 安装后运行

1. 双击 `OpenWarp.dmg` 挂载
2. 将 `OpenWarp` 拖到 `Applications` 文件夹
3. 从启动台或应用程序文件夹运行

> **注意**：首次运行可能需要右键选择"打开"，因为没有 Apple 开发者签名。

## 常见问题

### 1. 编译失败：`unable to find utility "metal"`

**原因**：缺少 Metal Toolchain

**解决**：
```bash
xcodebuild -downloadComponent MetalToolchain
```

### 2. 编译失败：`protoc: command not found`

**原因**：缺少 Protocol Buffers 编译器

**解决**：
```bash
brew install protobuf
```

### 3. 运行失败：`'warp-channel-config' was not found on PATH`

**原因**：使用了 `warp` 而非 `warp-oss`

**解决**：使用 `warp-oss` 二进制文件

### 4. DMG 中图标显示为空白

**原因**：缺少 .icns 图标文件

**解决**：按第三步 3.1 生成图标

### 5. 编译失败：`Xcode license agreements`

**原因**：未接受 Xcode 许可协议

**解决**：
```bash
sudo xcodebuild -license accept
```

## 文件结构

```
openwarp/
├── target/release/
│   ├── warp              # 完整版 (需要内部工具)
│   └── warp-oss          # 社区版 (可直接使用)
├── build/
│   ├── OpenWarp.app      # App 包
│   └── OpenWarp.dmg      # DMG 安装包
└── app/channels/oss/icon/
    ├── no-padding/       # PNG 图标源文件
    ├── AppIcon.iconset/  # iconset 临时目录
    └── AppIcon.icns      # macOS 图标文件
```

## 快速打包脚本

将以下内容保存为 `build_dmg.sh`，可一键完成打包：

```bash
#!/bin/bash
set -e

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_NAME="OpenWarp"
BUNDLE_DIR="$WORKSPACE_ROOT/build/$APP_NAME.app"
DMG_DIR="$WORKSPACE_ROOT/build/dmg"
TARGET_PROFILE="release"
WARP_BIN="warp-oss"

rm -rf "$WORKSPACE_ROOT/build"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"
mkdir -p "$DMG_DIR"

cp "$WORKSPACE_ROOT/target/$TARGET_PROFILE/$WARP_BIN" "$BUNDLE_DIR/Contents/MacOS/$APP_NAME"
chmod +x "$BUNDLE_DIR/Contents/MacOS/$APP_NAME"

ICNS_FILE="$WORKSPACE_ROOT/app/channels/oss/icon/AppIcon.icns"
if [[ ! -f "$ICNS_FILE" ]]; then
    ICONSET_DIR="$WORKSPACE_ROOT/app/channels/oss/icon/AppIcon.iconset"
    ICON_SRC="$WORKSPACE_ROOT/app/channels/oss/icon/no-padding"
    mkdir -p "$ICONSET_DIR"
    sips -z 16 16 "$ICON_SRC/16x16.png" --out "$ICONSET_DIR/icon_16x16.png"
    sips -z 32 32 "$ICON_SRC/16x16.png" --out "$ICONSET_DIR/icon_16x16@2x.png"
    sips -z 32 32 "$ICON_SRC/32x32.png" --out "$ICONSET_DIR/icon_32x32.png"
    sips -z 64 64 "$ICON_SRC/32x32.png" --out "$ICONSET_DIR/icon_32x32@2x.png"
    sips -z 128 128 "$ICON_SRC/128x128.png" --out "$ICONSET_DIR/icon_128x128.png"
    sips -z 256 256 "$ICON_SRC/128x128.png" --out "$ICONSET_DIR/icon_128x128@2x.png"
    sips -z 256 256 "$ICON_SRC/256x256.png" --out "$ICONSET_DIR/icon_256x256.png"
    sips -z 512 512 "$ICON_SRC/256x256.png" --out "$ICONSET_DIR/icon_256x256@2x.png"
    sips -z 512 512 "$ICON_SRC/512x512.png" --out "$ICONSET_DIR/icon_512x512.png"
    cp "$ICON_SRC/512x512.png" "$ICONSET_DIR/icon_512x512@2x.png"
    iconutil -c icns "$ICONSET_DIR" -o "$ICNS_FILE"
fi
cp "$ICNS_FILE" "$BUNDLE_DIR/Contents/Resources/AppIcon.icns"

cat > "$BUNDLE_DIR/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>English</string>
    <key>CFBundleDisplayName</key><string>OpenWarp</string>
    <key>CFBundleExecutable</key><string>OpenWarp</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundleIdentifier</key><string>dev.openwarp.OpenWarp</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>CFBundleLocalizations</key><array><string>en</string><string>ja</string><string>zh-CN</string></array>
    <key>CFBundleName</key><string>OpenWarp</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>0.1.0</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSHumanReadableCopyright</key><string>© 2025 OpenWarp Contributors</string>
    <key>LSMinimumSystemVersion</key><string>10.14</string>
</dict>
</plist>
PLIST

cp -R "$BUNDLE_DIR" "$DMG_DIR/"
create-dmg --volname "$APP_NAME" --no-internet-enable \
    --background "app/assets/resources/mac/warp_install_image.png" \
    --icon-size 128 --window-size 700 500 --format UDZO \
    --app-drop-link 550 250 --icon "$APP_NAME.app" 150 250 \
    --filesystem APFS "$WORKSPACE_ROOT/build/$APP_NAME.dmg" "$DMG_DIR"
rm -rf "$DMG_DIR"

echo "完成: build/$APP_NAME.dmg"
```

使用方式：
```bash
chmod +x build_dmg.sh
./build_dmg.sh
```
