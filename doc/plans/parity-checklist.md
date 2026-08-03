# 功能 Parity Checklist（Phase 5 完成度度量）

> 按模块跟踪与上游（参考镜像 `gqf2008/paperclip`）的功能对齐。
> 状态：`未开始` / `进行中` / `完成`。更新日期：2026-08-03（routines #54 完成）。

## 使用方式

- 参考镜像每次合入上游新功能后：在对应模块追加一行需求/测试任务，状态置为 `未开始`。
- Rust 实现完成 + 测试通过后：状态置为 `完成` 并注明 PR/测试位置。
- Phase 5 结束时：本表全部 `完成` 且 parity 测试清零。

## 模块清单（对应路线图 §4）

| 模块 | 上游功能/测试 | Rust 状态 | 备注/证据 |
|---|---|---|---|
| **P0 数据层** | Turso schema、迁移、公司隔离约束 | ✅ 完成 | `crates/data`；迁移 0001–0008；SQL 层 composite FK 隔离（`tests/data_layer.rs`） |
| **P0 核心 API：companies** | CRUD + issue prefix 分配 | ✅ 完成 | `routes/companies.rs` + repository；201/404/422 |
| **P0 核心 API：goals/projects** | CRUD + 层级约束（同公司引用） | ✅ 完成 | `routes/goals.rs`、`projects.rs`；跨公司引用 422 |
| **P0 核心 API：issues** | CRUD、identifier、状态机 §8.2、single-assignee、blocker | ✅ 完成 | `routes/issues.rs`、`issue_relations.rs`；状态转换 + 副作用测试 |
| **P0 核心 API：comments** | 评论 CRUD | ✅ 完成 | `routes/comments.rs` |
| **P0 核心 API：documents** | 追加式修订、issue 文档 key 链接 | ✅ 完成 | `routes/documents.rs`；revision 轮转测试 |
| **P0 核心 API：attachments** | 上传（本地盘）、内容读取、issue 链接 | ✅ 完成 | `routes/assets.rs`；multipart + SHA-256 |
| **P0 核心 API：work products** | issue work products | ✅ 完成 | `routes/work_products.rs`（迁移 0004） |
| **P0 执行控制面：heartbeat** | 发起/观察/取消/恢复、原子 checkout、执行锁 | ✅ 完成 | `routes/heartbeat.rs`；并发互斥测试 |
| **P0 执行控制面：watchdog** | §9.9 授权契约（子树、排除 task_watchdog 分支） | ✅ 完成 | `watchdog_authorized` + 测试 |
| **P0 执行控制面：失败归因** | infrastructure vs agent | ✅ 完成 | `error_kind` 列 + 测试 |
| **P0 执行控制面：recovery issue** | recovery actions、wake 调度 | ⏳ 进行中 | 授权门已就绪；调度/恢复执行未实现 |
| **P1 身份与安全：认证** | board 会话、agent API keys（哈希、吊销）、公司边界 | ✅ 完成 | `auth.rs`、`api_keys.rs`；三身份权限测试 |
| **P1 身份与安全：权限矩阵** | §9 权限矩阵 | ⏳ 进行中 | 核心 board-only/公司作用域已强制；完整矩阵（子预算、inbox 管理授权等）未全覆盖 |
| **P1 治理：预算/成本** | cost_events、聚合、硬停自动暂停 | ✅ 完成 | `costs.rs`；耗尽暂停 + 重置恢复测试 |
| **P1 治理：审批门** | approvals §8.3 状态机、审批门 | ✅ 完成 | `approvals.rs`；budget override 门测试 |
| **P1 治理：审计** | activity_log 全量 mutating 动作 | ✅ 完成 | `activity.rs` + 全路由接入 |
| **P1 治理：密钥** | company_secrets 版本化、加密、redaction | ✅ 完成 | `secrets.rs`、`secrets/` cipher；加密静态断言 + redact |
| **P2 治理扩展：决策桌** | queues/items/triage | ✅ 完成 | `decision_desk.rs`（迁移 0006） |
| **P2 治理扩展：skills** | 公司技能库 + 策略评估器 | ✅ 完成 | `skills.rs` 纯评估器 + repository（迁移 0008） |
| **P2 治理扩展：inbox** | 归档/恢复、注意力排序 | ✅ 完成 | `set_hidden`/`list_inbox`；排序为更新时间倒序 |
| **P2 治理扩展：external objects** | 关联 + 状态刷新 | ✅ 完成 | `external_objects.rs`（迁移 0007） |
| **P2 扩展：environments + 执行工作区** | environments 池、project/execution workspaces、runtime services、workspace operations | ✅ 完成 | `environments.rs` + `workspaces.rs`（迁移 0009）；issue #52/PR #67 |
| **P2 扩展：issue 结构增强** | labels、issue 线程、已读状态、审批链接、执行决策 | ✅ 完成 | `labels.rs` + `issue_structure.rs`（迁移 0010）；issue #53/PR #68 |
| **P2 扩展：routines** | 例行任务定义 + 追加式修订 + 触发器（manual/cron/webhook）+ 运行 | ✅ 完成 | `routines.rs`（迁移 0011）；issue #54/PR #69；cron 实际调度留给 scheduler |
| **P2 UI：看板** | 公司/项目/issue 列表 | ✅ 完成 | `ui/pages.rs` + 令牌层 |
| **P2 UI：issue 详情** | 属性/评论/文档/附件/work products | ✅ 完成 | `ui/pages.rs` issue_detail |
| **P2 UI：审批流** | 发起/审批/拒绝 | ✅ 完成 | `ui/pages.rs` approvals + 表单路由 |
| **P2 UI：审计视图** | 审计日志 | ✅ 完成 | `ui/pages.rs` activity |
| **P2 插件生态** | 外部适配器插件契约（版本化） | ✅ 完成 | `plugins.rs`；诊断报告 + 覆盖内置 |
| **P3 数据迁移** | Postgres → Turso 导出/导入 | ⏳ 进行中 | `tools/migrate/`（#26） |
| **P3 双栈切换** | Node 冻结与删除 | ⏳ 进行中 | #27 |

## 未对齐/明确延后的上游能力（登记为需求）

| 上游能力 | 说明 | 状态 |
|---|---|---|
| 完整权限矩阵（§9.8 scoped grants、inbox:manage 等） | 子集已实现 | 进行中 |
| recovery actions / wake 调度 | watchdog 授权已实现，调度器未实现 | 进行中 |
| managed checkout / git 凭据 | 上游近期新增 | 未开始 |
| decision desk 完整 retention/sweeper | 基础队列/三态已实现 | 进行中 |
| 插件生态（plugin namespaces 等） | 上游 addenda | 未开始 |
| 访问与运营（company memberships、instance roles、invites、board API keys、CLI auth challenges、budget policies/incidents、sidebar preferences、company logos） | 未实现 | 未开始（issue #56） |
| UI 完整功能（搜索、看板拖拽、设置页等） | 骨架 + 核心页已实现 | 进行中 |
| **UI 国际化（i18n）** | 上游已合入 zh-CN/zh-TW 全量 sweep + 多语言 locale（`ui/src/i18n/locales/*.json`，约 2100 键）；Rust/Topcoat UI 已有 en + zh-CN 轻量 i18n 层（`crates/app/src/i18n.rs`，`?lang=` 切换，6 个看板页面全量接入，issue #50/PR #51） | ✅ 完成（zh-TW 与完整键集为后续增量） |

## 参考镜像同步登记

- 每次 `gqf2008/paperclip` 同步上游后，diff 中新增的 API/表/测试按上述模块追加登记，状态默认 `未开始`。
- 2026-08-03：登记上游 **UI 国际化（i18n）sweep**（zh-CN/zh-TW 全量翻译 + 多语言 locale，来自参考分支 `sync/reference-i18n`，仅改动 Node 参考快照 `ui/`，不影响 `crates/`）。同日完成 Rust 侧 en + zh-CN 实现（#50）。
