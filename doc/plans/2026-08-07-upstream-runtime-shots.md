# 上游运行时截图基线（Storybook/UI 实测对照）— issue #238

> 状态：**完成（2026-08-07）**。上游 Storybook 在本机成功运行，按 F7 把 `ui-pixel-baseline.md` 中「派生/待实测」项升级为**实测**，产出 32 张 1440×900 运行时截图（浅色 16 + 深色 16）。
> 本文件为 #238 交付文档；`doc/plans/ui-pixel-baseline.md` 与 `doc/plans/parity-checklist.md` 的状态登记由主流程统一更新（本次不改）。

## 1. 运行方式与证据

- **上游参考镜像**：`/Volumes/Workspace/GitHub/paperclip/ui`（Node 参考快照，含 `node_modules`、`storybook/`、`vite.qa.config.mjs`）。
- **启动命令**（成功）：`cd /Volumes/Workspace/GitHub/paperclip/ui && pnpm storybook --no-open`
  - Storybook v10.5.5，监听 `http://localhost:6006/`；manager 346ms + preview 370ms 就绪；首屏 story 按需编译约 3–6s/个。
  - 未运行 `build-storybook`（dev server 已满足截图需要，避免整包构建）。
  - 未运行 `vite dev`（Storybook 组件级 mock 数据已覆盖目标组件/页面，且无需 DB/后端）。
- **截图工具**：Playwright headless Chromium（`/Users/sqb/Library/Caches/ms-playwright/chromium-1223/...`，本机已装，无需 `npx playwright install`）。
- **视口**：1440×900（与 Staple 截图一致），deviceScaleFactor=1。
- **取景方式**：直接访问 Storybook iframe（`http://localhost:6006/iframe.html?id=<story-id>&viewMode=story&globals=theme:light|dark`），按 story 内小节定位后截图；每张 1440×900 非空 PNG。
- **主题**：上游 `preview.tsx` 定义 `globalTypes.theme`，**默认 dark**；`globals=theme:light` 可切浅色。Staple 目前仅浅色，故**并排主集用浅色**（`upstream-runtime/*.png`），另存**默认运行时深色集**（`upstream-runtime/dark/*.png`）作为事实记录。

## 2. 截图清单（32 张）

主集（浅色，与 Staple 同主题并排）：`doc/plans/ui-baseline/upstream-runtime/*.png`（16 张）。
补充集（默认运行时深色）：`doc/plans/ui-baseline/upstream-runtime/dark/*.png`（16 张，同名）。

| # | 文件 | 来源 Storybook story（ID） | 内容 |
|---|---|---|---|
| 1 | `sidebar.png` | `product-navigation-layout--board-chrome-matrix` | 侧栏/导航布局：SidebarShell 展开态 + 折叠态、菜单、面包屑 |
| 2 | `sidebar-icon-alignment.png` | `product-navigation-layout--sidebar-icon-alignment` | 侧栏图标对齐 |
| 3 | `button.png` | `foundations-primitive-matrix--all-primitives` | Button 全部 variant（default/secondary/outline/ghost/destructive/link）与尺寸 |
| 4 | `badge.png` | `foundations-primitive-matrix--all-primitives` | Badge 全部 variant + 状态徽标（PAP-1641、in review 等） |
| 5 | `card.png` | `paperclip-assigned-backlog-safeguards--overview` | Card（`bg-card rounded-lg border p-4`）面板：创建条 + 意图卡 + 被停放工作 |
| 6 | `board-columns-cards.png` | `paperclip-successful-run-handoff--issue-card-indicator` | KanbanBoard 列（Backlog/Todo/In Progress/In Review/Blocked/Done/Cancelled）与任务卡 |
| 7 | `issue-list-rows.png` | `product-issue-management--full-surface-matrix` | IssuesList：分组 issue 行 + 列头（status/id/assignee/project/workspace/labels/updated） |
| 8 | `issue-columns.png` | `product-issue-management--full-surface-matrix` | IssueColumns 列配置 + IssueGroupHeader 分组 |
| 9 | `issue-detail-properties.png` | `product-issue-management--issue-properties-long-values-desktop` | IssueProperties 详情属性面板（状态/优先级/标签/负责人/项目/父级） |
| 10 | `issue-thread.png` | `product-chat-comments--issue-chat-with-timeline` | Issue 聊天/时间线线程（评论、运行、时间线事件） |
| 11 | `command-palette-inline.png` | `foundations-primitive-coverage--command-palette-inline` | CommandPalette（cmdk Command 组件）内联渲染 |
| 12 | `command-k-search.png` | `product-search-command-k--search-surfaces` | Cmd+K palette（Search-all 行）与搜索结果行/来源 chip |
| 13 | `inbox-rows.png` | `product-control-plane-surfaces--board-state-matrix` | Inbox/task 行（未读/已选/阻塞/评审请求状态） |
| 14 | `approval-card.png` | `product-control-plane-surfaces--board-state-matrix` | ApprovalCard 治理卡（pending/revision/approved） |
| 15 | `budget-card.png` | `product-control-plane-surfaces--board-state-matrix` | BudgetPolicyCard（healthy/warning/hard-stop） |
| 16 | `inbox-blocked-empty.png` | `product-inbox-blocked-tab--desktop-loaded` | Inbox Blocked tab 空态（桌面布局） |

> 深色补充集为同 16 个 story 在默认 `theme:dark` 下的截图。

## 3. 运行时实测数据（浅色，Playwright getComputedStyle）

实测确认 `ui-pixel-baseline.md` §2 的**浅色 token 表**与运行时一致：

| 项 | 上游运行时实测值 | Staple（#228 基线实测值） | 差异结论 |
|---|---|---|---|
| 侧边栏宽 | **240px**（`aside`，`border-r 1px oklch(0.922 0 0)`，bg `oklch(1 0 0)`） | 220px 固定 | 差异（窄 20px，且无折叠/拖拽） |
| Button 默认 | 高 **40px**，padding 8px 12px，radius **6.4px**（rounded-md），bg `oklch(0.205 0 0)`（近黑）+ 白字，font-size 14px / weight 500 | 高 36px，padding 8px 12px，radius 4.8px，蓝 `#2563eb` 底白字 | **差异**（高度/圆角/主色/字号/字重） |
| Badge | 高 **22px**，padding 2px 8px，radius rounded-full（胶囊，Tailwind v4 序列化为极大值），font-size 12px；default=近黑底白字、secondary=`oklch(0.97 0 0)` | Staple 徽标规格见基线 §3 | 待 Staple 侧复核（基线未量化 badge） |
| Card | radius **8px**（rounded-lg），border 1px `oklch(0.922 0 0)`，bg `oklch(1 0 0)`，padding 16px | inbox 卡 radius 8px / border `#e7e5e4` / padding 12px；看板卡 4.8px | 部分一致（8px/边框近似）；padding 与看板卡圆角不同 |
| CommandPalette | 宽 448px（max-w-md，**Storybook 内联故事**），radius 8px，border 1px，bg 白，shadow md（0 4px 6px -1px / 0 2px 4px -2px）；真实产品 Cmd+K 走 CommandDialog `sm:max-w-lg`=512px + shadow-lg（见 issue #252） | Staple 已按真实 Dialog 实施（#252/PR #253） | 已对齐（512px/8px/shadow-lg） |
| KanbanBoard | 列：Backlog/Todo/In Progress/In Review/Blocked/Done/Cancelled；卡含 identifier、标题、负责人头像 | Staple 看板列结构近似 | 结构近似，token 差异同上 |
| 字体 | `InterVariable, Inter, ui-sans-serif, …`（--font-sans） | Staple `ui-sans-serif, system-ui, …`（无 Inter） | **差异**（缺 Inter） |
| 浅色 token | background `oklch(1 0 0)`、foreground `oklch(0.145 0 0)`、primary `oklch(0.205 0 0)`、muted `oklch(0.97 0 0)`、muted-foreground `oklch(0.556 0 0)`、border `oklch(0.922 0 0)`、radius 0.5rem（md=×0.8、lg=0.5rem） | Staple 见基线 §2 表 | 与基线 §2 判断一致（系统性差异） |

## 4. 逐项与 Staple 的差异观察

- **侧栏/导航**：上游 240px、可折叠/拖拽（故事含 expanded/collapsed 态）；Staple 220px 固定 → F3 差异确认，且上游运行时为白底细边框。
- **Button/Card/Badge**：上游近黑主色 + 40px 按钮 + 6.4px 圆角 + Inter 字体；Staple 蓝主色 + 36px 按钮 + 4.8px 圆角 + 无 Inter → 基线 F1/F2/F5 差异全部由「派生」升级为「实测」。
- **Board 列与卡**：上游列结构与 Staple 近似（均为 Kanban 列 + 卡），但列标题样式（text-xs 大写、font-medium）、卡密度与 token 不同。
- **Issue 列表行**：上游列头（status/id/assignee/project/workspace/labels/updated）与 Staple 列表列结构近似；视觉 token 差异同上。
- **Issue 详情**：上游为右侧属性面板（IssueProperties，320px pane，长值截断）；Staple issue-detail 页结构近似，但为 SSR 页面布局而非组件面板。
- **CommandPalette / Cmd+K**：上游 cmdk 组件，浅色白底 8px 圆角；Staple 无对应组件（基线登记为未迁移/待实施项）。
- **Inbox 行**：上游行卡（未读/选中/阻塞/评审请求）与 Staple inbox 行结构近似，token 差异同上。

## 5. 仍不可核实项及原因

1. **完整 SPA 页面路由（/issues、/board、/search 等真实数据页）**：Storybook 提供的是组件级「矩阵」故事，不挂载 App 路由外壳；真实页面 = Storybook 片段 + 路由/数据层，需 vite dev + 后端/假数据。本次未运行 `vite dev`（Storybook 已满足验收 ≥8）。
2. **Command K 全局弹层交互**（打开/焦点/键盘导航/空态切换）：静态故事只呈现渲染态，无法由截图核实交互行为。
3. **侧栏折叠/拖拽、看板拖卡（DnD）交互**：截图仅静态态；交互行为需浏览器自动化实测（本次范围外）。
4. **字体文件来源**：已核实本地打包——上游 `ui/public/fonts/InterVariable.woff2` 存在，Storybook 下 `document.fonts` 确认 InterVariable 加载成功（Staple 侧 #236 已同步自托管 + 静态路由）。
5. **深色主题下的 Staple 对照**：Staple 当前仅浅色，深色集（`dark/`）仅作上游默认运行时记录，无法并排。

## 6. 验收对照

- [x] 上游 Storybook/UI 至少成功截图 8 个核心组件/页面（1440×900）→ **32 张成功**（16 浅色 + 16 深色）。
- [x] 基线文档「派生/待实测」项逐条标注实测结果或不可行原因 → 本文 §3–§5；`ui-pixel-baseline.md`/`parity-checklist.md` 由主流程统一更新。
- [x] 素材入库 `doc/plans/ui-baseline/upstream-runtime/`，并排入口更新 → `contact-sheet.html` 追加「上游运行时」区。
- [x] 上游 UI 可运行（Storybook v10.5.5 启动成功），无降级需求。

## 7. 复现/验证

```sh
# 1) 起上游 Storybook
cd /Volumes/Workspace/GitHub/paperclip/ui && pnpm storybook --no-open   # http://localhost:6006

# 2) 截图（Playwright headless，1440×900，脚本思路见提交说明）
#    浅色: iframe.html?id=<story-id>&viewMode=story&globals=theme:light
#    深色: 同上但不带 globals（默认 theme:dark）

# 3) 验证 PNG 非空
file doc/plans/ui-baseline/upstream-runtime/*.png   # PNG image data, 1440 x 900

# 4) 运行时实测（getComputedStyle）
#    body 字体 / --color-* / sidebar aside 宽度 240px / button h=40 radius=6.4px 等，见 §3
```

- 本任务无 Rust/UI 源码改动，未运行 cargo。
