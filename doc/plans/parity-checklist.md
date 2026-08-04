# 功能 Parity Checklist（Phase 5 完成度度量）

> 按模块跟踪与上游（参考镜像 `gqf2008/paperclip`）的功能对齐。
> 状态：`未开始` / `进行中` / `完成`。更新日期：2026-08-04（全部对齐完成：扩展 issues + scheduler + UI 全页面 + Pipelines 全套 + 38 语言 locale + 看板拖拽 + P3 双栈切换 #27）。

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
| **P0 执行控制面：recovery issue** | recovery actions、wake 调度 | ✅ 完成 | `agent_runtime` 仓库（迁移 0015）；issue #62；内建 scheduler 周期领取 wake → heartbeat run（`scheduler.rs`） |
| **P1 身份与安全：认证** | board 会话、agent API keys（哈希、吊销）、公司边界 | ✅ 完成 | `auth.rs`、`api_keys.rs`；三身份权限测试 |
| **P1 身份与安全：权限矩阵** | §9 权限矩阵（scoped grants、tasks:assign_scope、inbox:manage、manager-subtree 子预算） | ✅ 完成 | `principal_permission_grants`（迁移 0012）+ `permissions.rs` 评估器 + 路由；issue #55（成员/实例角色等由 #56 覆盖） |
| **P1 治理：预算/成本** | cost_events、聚合、硬停自动暂停 | ✅ 完成 | `costs.rs`；耗尽暂停 + 重置恢复测试 |
| **P1 治理：审批门** | approvals §8.3 状态机、审批门 | ✅ 完成 | `approvals.rs`；budget override 门测试 |
| **P1 治理：审计** | activity_log 全量 mutating 动作 | ✅ 完成 | `activity.rs` + 全路由接入 |
| **P1 治理：密钥** | company_secrets 版本化、加密、redaction | ✅ 完成 | `secrets.rs`、`secrets/` cipher；加密静态断言 + redact |
| **P2 治理扩展：决策桌** | queues/items/triage + triage 历史/retention/sweeper/通知 outbox | ✅ 完成 | `decision_desk.rs`（迁移 0006 + 0016）；issue #63；内建 scheduler 每日定时跑 90 天 sweeper（`scheduler.rs`） |
| **P2 治理扩展：skills** | 公司技能库 + 策略评估器 | ✅ 完成 | `skills.rs` 纯评估器 + repository（迁移 0008） |
| **P2 治理扩展：inbox** | 归档/恢复、注意力排序 | ✅ 完成 | `set_hidden`/`list_inbox`；排序为更新时间倒序 |
| **P2 治理扩展：external objects** | 关联 + 状态刷新 | ✅ 完成 | `external_objects.rs`（迁移 0007） |
| **P2 扩展：environments + 执行工作区** | environments 池、project/execution workspaces、runtime services、workspace operations | ✅ 完成 | `environments.rs` + `workspaces.rs`（迁移 0009）；issue #52/PR #67 |
| **P2 扩展：issue 结构增强** | labels、issue 线程、已读状态、审批链接、执行决策 | ✅ 完成 | `labels.rs` + `issue_structure.rs`（迁移 0010）；issue #53/PR #68 |
| **P2 扩展：访问与运营** | company memberships、instance roles、invites/join requests、board API keys、CLI auth challenges、budget policies/incidents、sidebar preferences、company logos | ✅ 完成 | `memberships`/`invites`/`board_keys`/`budget_policies`/`preferences` 仓库（迁移 0013）；issue #56；board key 认证（`bk-`）接入 auth 层 |
| **P2 扩展：routines** | 例行任务定义 + 追加式修订 + 触发器（manual/cron/webhook）+ 运行 | ✅ 完成 | `routines.rs`（迁移 0011）；issue #54/PR #69；内建 scheduler 按 cron 表达式触发（`scheduler.rs`） |
| **P2 扩展：managed checkout / git 凭据** | 服务端 clone/fetch + company secret 凭据注入 + redaction | ✅ 完成 | `git.rs`（迁移 0017）；issue #64；真实本地仓库 materialize 测试 + 凭据 redact 测试 |
| **P2 扩展：插件生态** | 插件注册/配置/状态/实体/作业/日志/webhook、database namespaces + migration ledger、company settings、managed resources | ✅ 完成 | `plugins` + `plugin_runtime` 仓库（迁移 0014）；issue #57；注册→配置→运行→日志全链路测试 |
| **P2 UI：看板** | 公司/项目/issue 列表 | ✅ 完成 | `ui/pages.rs` + 令牌层 |
| **P2 UI：issue 详情** | 属性/评论/文档/附件/work products | ✅ 完成 | `ui/pages.rs` issue_detail |
| **P2 UI：agents/inbox/决策桌/访问/成本/例行/密钥/技能/实例设置** | agents 列表/详情、收件箱、决策桌、公司访问（成员/邀请/加入申请/授权）、成本、例行任务、密钥、技能、实例设置（角色/board keys/CLI 挑战） | ✅ 完成 | `pages.rs` + `ui/routes.rs` 表单路由；issue #65；release smoke 覆盖全部新页面 |
| **P2 扩展：Pipelines（核心五表）** | pipelines/stages/transitions/cases/events + 强制流转 + 阶段移动 + 事件审计 | ✅ 完成 | 迁移 0019 + `pipelines` 仓库/路由/UI；issue #86/PR #87；issue_links/blockers/documents/automation 延后批次 |
| **P2 UI：搜索/看板/设置** | 任务搜索、看板列 + 原生 HTML5 拖拽状态变更、设置页（公司/预算/密钥/技能） | ✅ 完成 | `pages.rs` + `ui/routes.rs` + `board.js`；issue #65 + #94；smoke 覆盖渲染与脚本 |
| **P2 UI：审批流** | 发起/审批/拒绝 | ✅ 完成 | `ui/pages.rs` approvals + 表单路由 |
| **P2 UI：审计视图** | 审计日志 | ✅ 完成 | `ui/pages.rs` activity |
| **P2 插件生态** | 外部适配器插件契约（版本化） | ✅ 完成 | `plugins.rs`；诊断报告 + 覆盖内置 |
| **P3 数据迁移** | Postgres → Turso 导出/导入 | ✅ 完成 | `tools/migrate/`（#26 + #66）；Postgres 读取器（`export_postgres`）+ 列感知导入 + verify；本地 Postgres 契约测试 + CLI 端到端 |
| **P3 双栈切换** | Node 冻结与删除 | ✅ 完成 | #27：Node 树/workflows/scripts 与全部 Node 时代残留移除；Rust 单二进制唯一入口；参考镜像保留 Node 代码 |

## 未对齐/明确延后的上游能力（登记为需求）

| 上游能力 | 说明 | 状态 |
|---|---|---|
| **UI 国际化（i18n）** | 上游全部 38 种语言 locale（各 10172 键）嵌入 `crates/app/locales/`；`?lang=` 全语言切换，本地键 fallback | ✅ 完成（#50 + #65 + #90） |

## 数据结构对齐登记（与上游 schema 逐表逐列）

| 批次 | 范围 | 状态 |
|---|---|---|
| 第 1 批 | 执行面：heartbeat_runs +37 列、issues +16 列 + 索引（迁移 0021） | ✅ 完成（issue #96/PR #97） |
| 第 2 批 | Case 附属表（case_issue_links/case_events/case_documents/case_labels/case_attachments）+ external_objects 目录/mentions 对齐（旧 issue_external_objects 保留） | ✅ 完成（issue #98/PR #99） |
| 第 3 批 | 决策桌扩展（decisions/bundles/effect_executions/target_issues/training_examples） | ⏳ 待办 |
| 第 4 批 | Skills 版本体系 + Secret provider/bindings + user_secret_* | ⏳ 待办 |
| 第 5 批 | 工具链域（tool_*，约 30 张） | ⏳ 待办 |
| 第 6 批 | 基础设施（user/session/account/verification/instance_settings/folders 等） | ⏳ 待办 |
| 收尾 | 其余散表（status_cards/summary_slots/smoke_*/feedback_*/finance_*/heartbeat_run_events/watchdogs 等） | ⏳ 待办 |

## 参考镜像同步登记

- 每次 `gqf2008/paperclip` 同步上游后，diff 中新增的 API/表/测试按上述模块追加登记，状态默认 `未开始`。
- 2026-08-03：登记上游 **UI 国际化（i18n）sweep**（zh-CN/zh-TW 全量翻译 + 多语言 locale，来自参考分支 `sync/reference-i18n`，仅改动 Node 参考快照 `ui/`，不影响 `crates/`）。同日完成 Rust 侧 en + zh-CN 实现（#50）。
