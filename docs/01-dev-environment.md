# VIBE Skills Manager - 开发环境搭建指南

## 环境要求

| 工具 | 版本要求 | 用途 |
|------|---------|------|
| Node.js | 18+ | 前端构建、包管理 |
| Rust | latest stable | Tauri 后端 |
| pnpm | 8+ | 包管理（推荐，比 npm 快） |

---

## Windows 环境搭建

### 1. Node.js
已安装则跳过。
```bash
# 验证
node -v
```

### 2. pnpm
```bash
npm install -g pnpm
```

### 3. Rust 工具链
```bash
# 下载安装 rustup：https://rustup.rs/
# 或执行：
winget install Rustlang.Rustup

# 安装完成后重启终端，验证
rustc --version
cargo --version
```

### 4. Tauri 2 依赖（Windows）
Tauri 需要 WebView2（Win10/11 通常已预装）和 C++ 构建工具。

```bash
# 检查 WebView2（Win10 20H2+ 通常已有）
# 如果没有，下载安装：https://developer.microsoft.com/en-us/microsoft-edge/webview2/

# 安装 Visual Studio Build Tools（C++ 编译环境）
winget install Microsoft.VisualStudio.2022.BuildTools
# 安装时勾选 "Desktop development with C++"
```

### 5. Windows Symlink 权限
创建 symlink 需要以下任一条件：
- **开启开发者模式**（推荐）：设置 → 更新和安全 → 开发者选项 → 开发人员模式
- **管理员权限**运行终端

---

## Mac 环境搭建

### 1. Xcode Command Line Tools
```bash
xcode-select --install
```

### 2. Node.js
```bash
# 已安装则跳过
brew install node
```

### 3. pnpm
```bash
npm install -g pnpm
```

### 4. Rust 工具链
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证
rustc --version
cargo --version
```

---

## 项目初始化（两个平台通用）

### 1. 创建项目
```bash
cd F:\workspace\demo\qs-vibe-mg   # Windows
# cd ~/workspace/demo/qs-vibe-mg  # Mac

# 创建 Tauri 2 + Vue + TS 项目（在当前目录下）
pnpm create tauri-app . --template vue-ts
```

> 如果 `pnpm create tauri-app` 报错，可以用：
> ```bash
> npm create tauri-app@latest
> # 选择：Vue + TypeScript
> # 项目名：vibe-skills-manager
> # 然后把生成的文件移到当前目录
> ```

### 2. 安装依赖
```bash
pnpm install
```

### 3. 安装 Tailwind CSS
```bash
pnpm add -D tailwindcss @tailwindcss/vite
```

配置 `vite.config.ts`：
```ts
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [vue(), tailwindcss()],
});
```

在 `src/styles.css` 顶部添加：
```css
@import "tailwindcss";
```

### 4. 验证运行
```bash
pnpm tauri dev
```

首次编译 Rust 较慢（3-5 分钟），后续热更新秒级。

---

## 项目目录结构（初始化后）

```
qs-vibe-mg/
├── src-tauri/           # Rust 后端
│   ├── src/
│   │   └── main.rs      # Rust 入口
│   ├── Cargo.toml       # Rust 依赖
│   └── tauri.conf.json  # Tauri 配置
├── src/                 # Vue 前端
│   ├── App.vue
│   ├── main.ts
│   └── components/
├── package.json
├── vite.config.ts
├── tsconfig.json
└── docs/
```

---

## 常见问题

### Q: `pnpm tauri dev` 卡在编译 Rust？
首次需要下载和编译所有 Rust 依赖，正常。后续会缓存。

### Q: Windows 报 symlink 权限错误？
开启开发者模式：设置 → 更新和安全 → 开发者选项 → 开发人员模式

### Q: Mac 报 `xcrun: error: invalid active developer path`？
```bash
xcode-select --install
```

### Q: Rust 编译报 linker 错误（Windows）？
确保安装了 Visual Studio Build Tools 的 "Desktop development with C++" 组件。

### Q: 前端改了代码但没热更新？
检查 `pnpm tauri dev` 终端输出，Vite HMR 应该自动生效。如果只改了 Rust 代码需要重启。
