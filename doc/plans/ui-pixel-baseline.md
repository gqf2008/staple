# UI/UX 像素级对齐基线（Staple vs 上游 Paperclip）

> 状态：**基线已建立（2026-08-07）**。结论：**当前未达到像素级 1:1**，本文给出量化差距与可执行修复清单。
> 用途：后续 UI polish 的验收依据；对应 issue #228。
>
> **更新（2026-08-07）**：F1–F7 全部实施——F1/F2/F6（issue #236/PR #239）：色彩/圆角/动效/状态色/字体（InterVariable 自托管）对齐上游 index.css；F3/F4/F5（issue #237）：侧栏 240px + 折叠 rail 64px、内容区全宽 p-4/p-6、按钮/卡片规格对齐上游 shadcn（侧栏拖拽调整与 SPA 动画登记为架构近似）；F7（issue #238/PR #240）：上游 Storybook 运行时截图基线（32 张，浅色/深色）已入库。
>
> **更新（2026-08-07，issue #242）**：深色主题已实施——`.dark` 语义 token（背景/前景/卡片/主色/弱化/边框/破坏/侧栏等）与状态图标色（`--color-status-icon-*`，浅色+深色 AA 覆盖）逐值对齐上游 `ui/src/index.css` `.dark`，由可复现测试（`scripts/tests/theme_tokens.test.mjs`）锁定；`/static/theme_init.js`（head 同步，防 FOUC）+ `/static/theme.js`（system/light/dark 三态切换 + localStorage 持久化 + 跟随系统）。上游 Staple 无消费者的 token（`--bubble-agent`/`--chart-*`/`--chip-match-*`/`--paperclip-doc-annotation-*`）未移植，已登记。

### F1/F2/F6 实测记录（2026-08-07，issue #236）

Playwright 1440×900 实测（token 对齐后 build）：

| 项 | 实测值 |
|---|---|
| `body` font-family | `InterVariable, Inter, ui-sans-serif, …`（字体路由 200，`document.fonts.check('16px "InterVariable"') = true`） |
| `body` background / color | `oklch(1 0 0)` / `oklch(0.145 0 0)` |
| `--color-primary` | `oklch(0.205 0 0)`（主按钮背景，文字 `oklch(0.985 0 0)`） |
| `--color-muted-foreground` / `--color-border` | `oklch(0.556 0 0)` / `oklch(0.922 0 0)` |
| `--radius-md` / `--radius-lg` | `0.4rem` / `0.5rem` |
| `--motion-duration-fast` | `160ms`（reduced-motion 下 `0ms`） |
| 状态色 | `running=#2563eb`、`done=#22c55e`、`in-review=#7c3aed`、`cancelled/backlog=#a8aeb2`（release_smoke 断言 token 与字体路由） |

## 1. 方法与素材

- **Staple 侧**：`cargo run`（端口 3100，演示公司 `Demo 智能体公司`），Playwright headless Chromium 1440×900 截图 19 个核心页面（`doc/plans/ui-baseline/staple/*.png`），并实测计算样式（布局/字号/颜色/圆角/间距）。
- **上游侧**：参考镜像 `gqf2008/paperclip`（Node 参考快照）：
  - 设计令牌源 `ui/src/index.css`（DESIGN.md 指定的唯一 token 源）；
  - 布局常量与组件规格（`SidebarShell.tsx`、`ui/button.tsx`、`ui/card.tsx` 等）；
  - 已有截图 `docs/pr-screenshots/**`、`doc/assets/**`（`doc/plans/ui-baseline/upstream/*.png`）作为人眼对照。
- 由于上游 SPA 未在本机运行，上游**运行时**渲染值标为「派生/待实测」；凡标「实测」的均为本机直接测量。

**查看入口**：`doc/plans/ui-baseline/contact-sheet.html`（Staple 19 页 vs 上游 6 张参考图并排）。

## 2. Token 层对比（量化）

令牌源比对：上游 `ui/src/index.css` 含 **540 个唯一 token 名（595 处声明）**，Staple `crates/app/src/ui/styles.rs` 仅 **50 个**（统计命令见 §7）。**仅 18 个同名 token，其中 17 个解析值不同**（`--color-card` 同为纯白，`--font-mono` 主字体栈一致、回退链略短）；上游语义层约 80+ token，Staple 大量缺失（popover/secondary/accent/input/ring/sidebar-*/chart-*/status-*/agent 渐变/chip-match 等）。

| token | 上游值（ui/src/index.css） | Staple 值（styles.rs） | 差异 |
|---|---|---|---|
| `--color-background` | `oklch(1 0 0)`（纯白 #fff） | `#fafaf9`（暖灰 stone-50） | 明显 |
| `--color-foreground` | `oklch(0.145 0 0)`（近黑） | `#1c1917`（stone-900） | 轻微 |
| `--color-card` | `oklch(1 0 0)` = #fff | `#ffffff` | 解析值一致 |
| `--color-primary` | `oklch(0.205 0 0)`（近黑主色） | `#2563eb`（蓝 blue-600） | **重大** |
| `--color-muted` | `oklch(0.97 0 0)`（近白） | `#f5f5f4` | 轻微 |
| `--color-muted-foreground` | `oklch(0.556 0 0)`（中性灰） | `#78716c`（暖灰） | 明显（色相偏暖）|
| `--color-border` | `oklch(0.922 0 0)` | `#e7e5e4` | 轻微 |
| `--color-destructive` | `oklch(0.577 0.245 27.325)` | `#dc2626` | 近似（红）|
| `--font-sans` | `InterVariable, Inter, ui-sans-serif…` | `ui-sans-serif, system-ui…` | **重大（无 Inter）** |
| `--font-mono` | `ui-monospace, SFMono-Regular, Menlo…` | `ui-monospace, SFMono-Regular, Menlo…` | 主栈一致，回退链略短 |
| `--radius-sm/md/lg` | `0.3rem / 0.4rem / 0.5rem` | `0.3rem / 0.5rem / 0.75rem` | md/lg 不一致 |
| `--motion-duration-fast/base/slow` | `160ms / 240ms / 360ms`（`prefers-reduced-motion` 下归零 0ms） | `120ms / 200ms / 1s` | fast/slow 不一致 |

> 说明：上游 `--radius` 基值 0.5rem，sm=0.3、md=0.4、lg=0.5；Staple md=0.5、lg=0.75，圆角整体偏大。上游 motion 值见 index.css（待复核具体行），Staple `slow=1s` 用于 pulse 动画。

## 3. 布局度量对比（1440×900）

| 项 | Staple（实测） | 上游（派生/代码） | 结论 |
|---|---|---|---|
| 侧边栏宽 | **220px 固定**，无折叠/rail/拖拽 | 默认 **240px**，可拖拽 208–420，rail 64px，可折叠（SidebarShell.tsx） | 差异（窄 20px + 少交互） |
| 内容区 | `max-width:960px; margin:0 auto; padding:24px`（.app-main） | `flex-1` 全宽，`p-4 md:p-6`（16/24px），无 max-width | **差异（960 居中 vs 全宽）** |
| 页面 h1 | 24px（`--font-size-xl`） | 上游页面常用 `text-2xl`(24px)/`text-3xl`，不统一 | 待实测 |
| 正文字号 | 16px（body） | Tailwind `text-sm`(14px) 为主、`text-base`(16px) 为辅 | **可能差异（整体偏大）** |
| 主按钮 | 高 36px，padding 8px 12px，radius 4.8px，蓝底白字 | shadcn Button 默认 `h-10`(40px) `px-4`，`rounded-md`(6.4px)，`bg-primary`(近黑) `text-sm`(14px) | **差异（高度/圆角/主色/字号）** |
| 卡片 | inbox 卡 radius 8px、border #e7e5e4、padding 12px；看板卡 radius 4.8px | shadcn Card `rounded-lg`(8px)、`border`、`py-6`、header `px-6` | 部分一致（8px），padding 与看板卡圆角不一致 |
| 品牌色 | 蓝 `#2563eb` 为主色 | 近黑 `oklch(0.205 0 0)` 为主色，蓝仅作状态色 | **重大品牌差异** |

## 4. 逐页结论（19 页）

| 页面 | Staple 截图 | 上游对照 | 结构 | 备注 |
|---|---|---|---|---|
| Home `/` | staple/home.png | 06-invite-landing（邀约落地页，非完全同页） | 近似 | 结构近，但配色/字体/宽度体系不同 |
| Company | staple/company.png | 01-nav-layout | 近似 | 侧栏 220 vs 240、内容 960 居中 |
| Board | staple/board.png | 04-board-backlog | 近似 | 列/卡结构有，主色/圆角/密度不同 |
| Issues | staple/issues.png | 02-issues-list | 近似 | 列表卡结构有，尺寸/颜色不同 |
| Issue detail | staple/issue-detail.png | 05-issue-thread | 近似 | 信息架构近，视觉 token 不同 |
| Inbox | staple/inbox.png | 03-inbox | 近似 | 行卡结构有，token 不同 |
| Approvals / Decisions | staple/approvals.png、decisions.png | — | 结构对齐 | 卡片化已做（B5），视觉 token 不同 |
| My issues / What needs me | staple/my-issues.png、what-needs-me.png | — | 结构对齐 | 同上 |
| Agents / Costs / Routines / Skills | 对应截图 | — | 结构对齐 | B7 卡片化 |
| Board chat | staple/board-chat.png | —（上游 task-chat 已登记不迁移） | 近似 | 聊天交互有，上游富文本/附件未迁移 |
| Dashboard / Settings / Pipelines / Review queue | 对应截图 | — | 结构对齐 | 功能页 |

**总结**：页面**结构与信息架构**与上游大体对齐（功能 1:1 基本达成）；**视觉层（token）系统性未对齐**——主色、底色、字体、圆角、内容宽度、按钮形态均与上游不一致。

## 5. 明确结论

1. **未达到像素级 1:1（剩余项均为已登记架构性差异）**。功能/API/数据结构已对齐（见 parity-checklist）；视觉 token 层（#236）、布局与组件规格（#237）、上游运行时实测基线（#238）、深色主题（#242）均已实施；剩余差异为 Topcoat SSR 架构性限制（侧栏拖拽尺寸、SPA 路由动画/乐观更新/skeleton、移动端适配），已逐项登记。
2. 差距分两类：
   - **可修复（建议立项）**：token 层对齐（背景/前景/主色/圆角/字体）、侧栏 240px + 折叠/拖拽、内容区去 960 居中改全宽、按钮/卡片规格对齐。
   - **架构性不可 1:1（登记即可）**：SPA 路由动画/即时局部更新/富文本编辑器，Topcoat SSR 下只能近似（parity-checklist 已登记）。

## 6. 可执行修复清单（后续 issue）

- [x] F1 视觉 token 层对齐上游（issue #236：色彩/圆角/动效已对齐；补齐 input/ring/sidebar/popover 等语义 token）：`--color-background/foreground/primary/muted-*/border/radius-*` 等值改为上游 OKLCH 值（保留 Staple 扩展 token 作为超集），并补齐缺失语义 token（sidebar/input/ring/popover 等）。
- [x] F2 字体：引入 InterVariable（issue #236：自托管 woff2 + @font-face + 静态路由）（或本地字体回退链）对齐上游 `--font-sans`。
- [x] F3 侧栏（issue #237 + #244）：默认 240px + 折叠 rail 64px（#sidebar-toggle + localStorage，折叠后隐藏文本残片）；拖拽调整宽度已实现（#sidebar-resizer，208–420px 夹紧 + localStorage 持久化 + 键盘 Arrow/Home/End + aria-valuenow）；≤48rem 窄屏自动折叠为 rail 64px 并隐藏折叠/拖拽控件（与上游 off-canvas drawer 的差异登记为 SSR 近似；根页 `.inline-form` 窄屏溢出已修复（issue #246））。
- [x] F4 内容区（issue #237）：移除 960px 居中，改全宽 + `p-4`（`≥48rem` 时 `p-6`）。
- [x] F5 按钮/卡片（issue #237）：主按钮 `h-10/px-4/text-sm/rounded-md`；卡片 `rounded-lg/py-6/px-6` 无阴影（对齐上游 Card，真实表面 `.issue-section` 已对齐；`.card` 为预留基类）。
- [x] F6 状态色/优先级色（issue #236：状态色对齐上游 --status-*；优先级色保留为扩展并登记）：上游 `--status-task-*`/`--status-agent-*`（hex）与 Staple `--color-status-*` 映射核对（部分已对齐，如 done=#16a34a vs 上游 #22c55e 需复核）。
- [x] F7 上游运行时截图基线（issue #238/PR #240）：上游 Storybook v10.5.5 实测 32 张（浅 16 + 深 16），侧栏 240/Button 40×6.4px/Badge 22px/Card 8px 16px/CommandPalette 448px 全部实测，见 `doc/plans/2026-08-07-upstream-runtime-shots.md`。

## 7. 复现/验证

```sh
# Staple 运行
STAPLE_DB_PATH=/tmp/staple-demo-1786027836.db PORT=3100 ./target/debug/staple-app
# 截图（本机 Playwright headless 1440×900，脚本见提交说明）
# token 对比
grep -oE '^\s*--[A-Za-z0-9-]+:' /Volumes/Workspace/GitHub/paperclip/ui/src/index.css | sort -u | wc -l   # 540 唯一
grep -cE '^\s*--[a-z0-9-]+:' crates/app/src/ui/styles.rs   # 50
```

- 交付物：`doc/plans/ui-baseline/`（staple/ 19 页 + upstream/ 6 张对照 + contact-sheet.html）。
- 本次为文档/基线交付，不含代码改动；F1–F7 已由 issue #236/#237/#238 实施并合入。
