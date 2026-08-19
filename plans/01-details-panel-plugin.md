# Plan: dsh-plugin-workbench — DSH 右侧多标签任务栏

## 概述

创建一个 DSH 插件，替换 `@deepseek-ai/dsh-client-ui-conversation` 注册的 `DetailsPanel`，实现多标签页右侧任务栏。

## 已确认的 API（Phase 0 文档发现）

### 插槽注册 API
- **来源**: `@deepseek-ai/dsh-client-ui-slots/lib/types/index.d.ts:562-577`
- `slots.register({ name, children?, store?, locale?, inject?, ... }, Component)` → `() => void`
- `slots.inject(key, callback)` → `() => void`
- **注意**: `details` 是 `kind: "single"`, `scope: "session"` 插槽，后注册者覆盖先注册者

### 布局服务
- **来源**: `@deepseek-ai/dsh-client-ui-layout/lib/types/client/service.d.ts:21-46`
- `ctx.layout.openDetails()` → void
- `ctx.layout.closeDetails()` → void
- `ctx.layout.toggleSidebar()` → void

### 语言服务
- **来源**: `@deepseek-ai/dsh-client-locale/lib/client.js:1002`
- `ctx.locale.register(ns, { zh, en })` → disposer
- `ctx.locale.bind(ns)` → `(key, params?) => string`

### 插件入口格式
- **来源**: `dshmarket/src/client/index.ts`
- 必须导出: `name: string`, `inject: string[]`, `apply(ctx): void`
- `ctx` 提供: `effect`, `on`, `slots`, `locale`, `theme`, `layout`

### 构建输出格式
- **来源**: `dshmarket/client/client.js:1-12`
- `window.__ModuleLoader__.load({ id: "package-name", factory: (require) => { ... return module.exports } })`
- 外部依赖通过 `require("react")`, `require("@deepseek-ai/dsh-client-ui-primitives")` 获取

### UI Primitives
- **来源**: `@deepseek-ai/dsh-client-ui-primitives/lib/types/index.d.ts:4-48`
- 可用组件: `Button`, `CodeBlock`, `MarkdownText`, `JsonBlock`, `Tooltip`, `Input`, `Menu`
- 可用图标: `IconCloseOutline16`, `IconChevronLeftOutline14`, `IconChevronRightOutline14`, `IconPanelLeftOutline16`, `IconBrowseOutline16`, `IconFolderOpenOutline16`, `IconDataOutline16`, `IconArchiveOutline20`, `IconCodeOutline16` 等

### 反模式清单
- ❌ 不要假设 `inject` 回调有参数（`details` 插槽的 inject 是零参数）
- ❌ 不要使用 `--dsw-*` 之外的颜色值——必须使用 DSH 设计令牌
- ❌ 不要调用不存在的 API 方法
- ❌ 不要忘记 `ctx.effect()` 包装注册——否则生命周期不受管理

---

## Phase 1: 项目脚手架

### 任务
创建插件包的目录结构、配置文件、构建脚本

### 文件清单
```
dsh-plugin-workbench/
├── package.json          # 已创建
├── pnpm-lock.yaml        # pnpm install 后生成
├── cordis.patch.yml      # 插件声明
├── scripts/
│   └── build.mjs         # esbuild 构建脚本
├── src/
│   └── client/
│       ├── index.ts      # 插件入口
│       ├── locales.ts    # 中英文语言包
│       ├── components/
│       │   ├── WorkbenchPanel.tsx  # 主面板容器
│       │   └── TabBar.tsx          # 标签栏
│       └── tabs/
│           ├── EvidenceTab.tsx
│           ├── BrowserTab.tsx
│           ├── FilesTab.tsx
│           ├── ArtifactsTab.tsx
│           └── SiteTab.tsx
└── client/
    └── client.js         # 构建产物
```

### cordis.patch.yml
```yaml
# dsh-plugin-workbench bundle patch
- insert:
    - id: dsh-plugin-workbench
      name: 'dsh-plugin-workbench'
```

### scripts/build.mjs
使用 esbuild 打包，目标格式为 `window.__ModuleLoader__.load()` 格式：
- 入口: `src/client/index.ts`
- 输出: `client/client.js`
- 外部依赖: `react`, `react/jsx-runtime`, `@deepseek-ai/dsh-client-ui-primitives`
- 使用 esbuild 的 `banner` 和 `footer` 选项包裹 `__ModuleLoader__.load()` 调用

### 验证清单
- [ ] `pnpm install` 在项目根目录成功
- [ ] 构建脚本运行成功，输出 `client/client.js`
- [ ] 输出文件以 `window.__ModuleLoader__.load({id: "dsh-plugin-workbench", ...})` 开头
- [ ] 输出文件在 factory 末尾 `return module.exports`
- [ ] 外部依赖被正确标记为 external，不打包进 bundle

---

## Phase 2: 插件入口与语言包

### 任务
实现 `src/client/index.ts`（插件入口）和 `src/client/locales.ts`（语言包）

### 入口逻辑
```
apply(ctx):
  1. ctx.effect(() => ctx.locale.register(NS, { zh, en }), "dsh-plugin-workbench: dict")
  2. const t = ctx.locale.bind(NS)
  3. ctx.slots.inject("details", () => {
       return ctx.slots.register({
         name: "details",
         locale: NS,
         children: {
           "details.tab.evidence":  { kind: "single", scope: "session" },
           "details.tab.browser":   { kind: "single", scope: "session" },
           "details.tab.files":     { kind: "single", scope: "session" },
           "details.tab.artifacts": { kind: "single", scope: "session" },
           "details.tab.site":      { kind: "single", scope: "session" },
         },
         inject: () => ({ closeDetails: () => ctx.layout.closeDetails() })
       }, WorkbenchPanel)
     })
```

### 语言包结构
```typescript
export const zh = {
  'tab.evidence': '结果与证据',
  'tab.browser': '浏览器',
  'tab.files': '文件',
  'tab.artifacts': '产物',
  'tab.site': 'Site',
  'close': '关闭',
  'empty': '选择工具调用以查看详情',
}
export const en = { ... }  // 对应英文
```

### 关键设计
- 声明 5 个子插槽，每个标签页一个
- 子插槽命名空间为 `details.tab.*`，与现有 `conversation.details.tool` 不冲突
- 不声明 `store` 参数——我们的面板不需要额外状态，依赖从 slot 系统继承的 props
- `inject` 工厂返回 `{ closeDetails }`，会在组件 props 中可用

### 验证
- [ ] `apply()` 函数无语法错误
- [ ] 语言包注册到 `dsh-plugin-workbench` 命名空间
- [ ] 5 个子插槽声明完整
- [ ] `slots.inject` 使用 `"details"` 作为目标插槽名

---

## Phase 3: 标签页容器组件

### 任务
实现 `WorkbenchPanel` 主容器和 `TabBar` 标签栏

### WorkbenchPanel 设计
- Props 接收: `closeDetails`, `t`, `renderSlot`, `useSession`, `sessionId`, `useSessions`, `useStore`, `actions`
- 内部状态: `activeTab` (默认 `'evidence'`)
- 渲染: 顶部 TabBar + 中间内容区（通过 `renderSlot(tab.slotName, props, { fallback })`）
- 关闭按钮在右上角，调用 `closeDetails()`

### TabBar 设计
- Props: `tabs: { id, label }[]`, `activeTab`, `onTabChange`
- 横向滚动容器，每个 tab 是 button
- active tab 有下划线高亮
- 使用 `--dsw-alias-label-primary/secondary` 和 `--dsw-alias-accent` 设计令牌

### 样式设计
- 所有颜色使用 `--dsw-*` CSS 变量，继承 DSH 主题
- 布局: flex column, header 固定高度, body flex:1 可滚动
- 无额外 CSS 文件——样式内联在组件中（通过 style 对象或 CSS-in-JS）
- 过渡动画: 标签切换时透明渐入

### 验证
- [ ] TabBar 渲染 5 个标签，active 高亮正确
- [ ] 点击标签切换 activeTab，内容区更新
- [ ] 关闭按钮响应点击
- [ ] 无注册内容时显示 fallback 占位
- [ ] 主题切换时颜色自动跟随（使用 CSS 变量）

---

## Phase 4: 标签页内容组件

### 任务
实现 5 个标签页内容组件

### EvidenceTab（结果与证据）
- 最关键的一个标签页
- 内部转发到 `conversation.details.tool` 子插槽
- 兼容现有 `@deepseek-ai/dsh-client-ui-tool` 注册的 `ToolDetails` 组件
- 当无选中工具调用时显示空状态提示

### BrowserTab（浏览器）
- 占位组件，显示"开发中"提示
- 后续可扩展为 WebView 容器

### FilesTab（文件）
- 占位组件

### ArtifactsTab（产物）
- 占位组件

### SiteTab（Site）
- 占位组件

### 验证
- [ ] EvidenceTab 正确渲染 `ToolDetails` 内容（当有 conversation.details.tool 注册时）
- [ ] 所有标签页切换无错误
- [ ] 空状态显示正确文案

---

## Phase 5: 构建、配置 Profile、验证

### 任务
构建插件、配置 DSH profile 加载它、验证运行

### Profile 配置
编辑 `~/.dsh/profiles/web/package.json`:
```json
{
  "dependencies": {
    "dshmarket": "*",
    "dsh-plugin-workbench": "*"
  },
  "dsh": {
    "profile": {
      "bundles": [
        "@deepseek-ai/dsh-base",
        "@deepseek-ai/dsh-web-app",
        "dshmarket",
        "dsh-plugin-workbench"
      ]
    }
  }
}
```

### 链接方式
将 `dsh-plugin-workbench` 链接到 profile 的 node_modules:
```bash
ln -s /path/to/dsh-plugin-workbench ~/.dsh/profiles/web/node_modules/dsh-plugin-workbench
```

### 覆盖优先级
由于 `details` 是 `kind: "single"` 插槽，后注册者覆盖先注册者。`dsh-plugin-workbench` 作为最后一个 bundle 加载，其 `apply()` 会覆盖 `@deepseek-ai/dsh-client-ui-conversation` 的 `DetailsPanel` 注册。

### 验证清单
- [ ] `node scripts/build.mjs` 构建成功
- [ ] `client/client.js` 格式正确
- [ ] Profile 配置正确，bundles 列表包含 `dsh-plugin-workbench`
- [ ] 重启 DSH 后右侧栏显示标签页 UI
- [ ] 标签页切换正常工作
- [ ] 关闭按钮收起右侧栏
- [ ] 点击 tool call 时 `ctx.layout.openDetails()` 打开右侧栏（由 conversation 插件触发）
- [ ] EvidenceTab 正确显示工具调用详情
- [ ] 浏览器控制台无 JS 错误
- [ ] 主题切换时颜色跟随变化

---

## 发布路线图（后续）

1. 发布到 npm: `npm publish`
2. 用户通过 dshmarket 安装: `dsh plugin --profile web add dsh-plugin-workbench`
3. 或直接通过市场 UI 搜索安装

## 已知限制
- `details` 是 single 插槽，不能与其他插件共存。这是 DSH 框架的设计限制，不是插件的问题
- 浏览器标签页需要 WebView，在纯 Web 环境中受限
- 文件标签页需要文件系统访问，需要通过 Tauri IPC 或 DSH workspace API