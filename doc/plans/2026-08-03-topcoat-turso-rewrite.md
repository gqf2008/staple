# Topcoat + Turso 重写规划

日期：2026-08-03
状态：规划中（Phase 0 已完成）
范围：`gqf2008/staple`（独立项目，非 fork）

## 1. 背景与目标

以 **Topcoat + Turso 从零重写** Paperclip（AI 智能体公司控制面）。项目架构：

- **`gqf2008/staple`（本仓库，独立项目）**：不是 GitHub fork，不跟踪上游。
  重写工作全部在这里进行，最终完全替换上游 Node.js 代码。
- **`gqf2008/paperclip`（参考镜像 fork）**：保留上游全量代码与自动同步
  （`sync/upstream` 分支 + fast-forward/PR 工作流），只作为功能参考，永不合并回本仓库。

## 2. 现状盘点（参考代码，Node.js 版）

- **Monorepo（pnpm）**：`server/`（Express REST API + 编排服务）、`ui/`（React + Vite）、`packages/db`（Drizzle + Postgres，dev 用嵌入式 PGlite）、`packages/shared`、`packages/adapters`（Claude Code、Codex、Cursor 等适配器）、`packages/plugins`。
- **核心能力**（对齐 `doc/SPEC-implementation.md` 与近期新增）：
  - 公司与组织：companies、goals、projects、memberships、org 结构
  - 任务体系：issues（稳定 ID、父子任务、blocker、single-assignee、评论、文档、附件、work products）
  - 执行控制面：heartbeat_runs、原子 checkout、执行锁、recovery、task watchdog、workspaces/runtime 服务
  - 治理：预算与 cost_events（硬停自动暂停）、approvals 审批门、activity_log 审计、密钥管理（company_secrets + 版本）
  - 智能体接入：适配器（本地 CLI 会话、HTTP/webhook、外部插件）、agent_api_keys（哈希存储、公司隔离）
  - 近期上游新增：decision desk（决策队列/保留）、skills（公司技能库与策略）、inbox/attention、external objects、managed checkout + git 凭据

## 3. 技术选型：为什么 Topcoat + Turso

### Topcoat（Rust 全栈 Web 框架）
- Tokio 团队 2026-07-22 发布，server-first、无 WASM，服务端部分重渲染。
- 单一二进制交付：部署/运维成本远低于 Node + 前端构建链。
- 端到端类型安全（Rust 类型贯穿路由、模板、表单）；Tokio 生态性能与并发能力适合控制面这类 I/O 密集 + 后台任务密集系统。

### Turso（libSQL / SQLite 生态）
- Rust 系（Limbo 重写）SQLite/libSQL，本地优先 + 同步能力，可嵌入 Rust 进程。
- Paperclip 是单实例、多公司的控制面，数据量远低于分布式 OLTP 场景；Turso 提供更简单的部署（无需独立数据库服务），并保留并发写入改进。
- dev 环境直接用嵌入式 Turso，无需 PGlite。

### 权衡
- Topcoat 生态年轻（2026-07 发布），UI 组件与第三方库远少于 React 生态——重写 UI 将是主要成本。
- 上游仍以 Node 演进；重写期间以功能对齐替代代码级合并。

## 4. 功能清单（重写范围，按优先级）

| 优先级 | 模块 | 说明 |
|---|---|---|
| P0 | 数据层 | Turso schema、迁移工具、公司隔离约束 |
| P0 | 核心 API | companies / goals / projects / issues / comments / documents / attachments |
| P0 | 执行控制面 | heartbeat、原子 checkout、锁、recovery、watchdog、预算硬停、审批门、审计日志 |
| P1 | 身份与安全 | board 会话、agent API keys（哈希 + 公司隔离）、权限矩阵 |
| P1 | 智能体接入 | 适配器契约（CLI / HTTP / webhook）、heartbeat 协议 |
| P1 | 治理模块 | 密钥管理、审批、审计、决策 desk、skills 策略 |
| P2 | UI（Topcoat） | 看板、issue 详情、设置、审批流、审计视图 |
| P2 | 插件/生态 | 外部适配器插件契约（若可行，兼容现有 JSON 契约） |
| P3 | 数据迁移 | Postgres → Turso 导出/导入工具 |

## 5. 分阶段路线图

- **Phase 0（已完成）**：独立仓库 `gqf2008/staple`、参考镜像 fork（含自动同步）、本规划。
- **Phase 1：Rust 骨架 + 数据层**。Cargo workspace（`crates/`）、Topcoat 最小应用、Turso 连接、schema 建模（对齐 §7 数据模型）、迁移工具。验证：`/api/health`、companies CRUD。
- **Phase 2：核心 API**。issues/agents/heartbeat/budgets/approvals/audit/secrets，逐步以 Rust 服务替换 Node 路由。每完成一模块跑上游对应测试作为行为基准。
- **Phase 3：Topcoat UI**。看板与详情页，对齐现有 DESIGN.md 的设计令牌体系。
- **Phase 4：适配器与执行**。heartbeat 调度、workspace/runtime、外部适配器调用。
- **Phase 5：对齐与切换**。功能 parity checklist 清零、数据迁移工具、Node 代码冻结并删除。

## 6. 上游参考策略（本仓库不直接同步）

- **本仓库（staple）**：无 `upstream` remote、无同步工作流。上游只以文档与测试形式沉淀为需求（见 parity checklist）。
- **参考镜像（gqf2008/paperclip）**：`.github/workflows/sync-upstream.yml` 每 6 小时 + 手动触发同步；未分叉自动 fast-forward，分叉后开 PR。本地可用 `scripts/sync-upstream.sh`。
- **功能对齐机制**：维护 `doc/plans/parity-checklist.md`，镜像每次合入上游后，把新功能登记为需求/测试，翻译成 Rust 实现任务。

## 7. 数据迁移（Postgres → Turso）

- 用 Drizzle/Postgres 导出为行级快照（companies/agents/issues/...），Turso 侧按新 schema 导入。
- 迁移脚本放 `tools/migrate/`，先支持离线一次性迁移；后续按需提供增量。
- 保留 `doc/DATABASE.md` 中所有索引/约束语义（唯一性、外键、公司边界）。

## 8. 风险与决策点

- **风险**：Topcoat 生态新、UI 复刻成本高、上游演进快、插件生态（Node 包）难以直接复用。
- **决策点**：
  1. UI 完全用 Topcoat 模板渲染，还是先做 API + 保留 React 前端过渡？
  2. 重写期间是否继续维护 Node 版功能（双轨成本）？
  3. Turso 用嵌入式模式（每实例一库）还是托管同步模式？

## 9. 附：已落地的基础设施

- 独立仓库 `gqf2008/staple`（默认分支 `main`，非 fork）。
- 参考镜像 `gqf2008/paperclip`：master 与上游同步、`sync/upstream` 镜像分支、自动同步工作流（已验证）、本地脚本。
- 本规划文档。
