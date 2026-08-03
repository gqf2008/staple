# Staple 重写开工手册（Codex 启动指引）

> 给接手开发工作的 Codex 实例。开工前完整读一遍本文件。

## 1. 项目身份

- **本仓库**：`gqf2008/staple` —— 独立项目，**不是 fork**，与上游零关联。
- **使命**：用 **Rust + Topcoat + Turso** 从零重写 Paperclip（AI 智能体公司控制面）。
- **参考镜像**：`gqf2008/paperclip`（fork，自动同步上游）——只读参考，改动永不回传。

## 2. 必读文档（按顺序）

1. `README.md` —— 项目定位与状态
2. `doc/plans/2026-08-03-topcoat-turso-rewrite.md` —— 路线图、功能清单、分阶段计划
3. `doc/plans/2026-08-03-codex-onboarding.md`（本文件）
4. `AGENTS.md` —— 仓库协作规则
5. 需要行为基准时：参考镜像的 `doc/SPEC-implementation.md`、`doc/PRODUCT.md`、`doc/DATABASE.md`

## 3. 工作区与环境

- 主工作区：`/Volumes/Workspace/GitHub/staple`（挂载卷）
- **Topcoat 源码**：`/Volumes/Workspace/GitHub/topcoat`（本地已有，写代码前先查它的 API/examples，不要臆造 API）
- **Rust 工具链**：rustup stable（edition 2024），`rust-toolchain.toml` 锁定版本
- **Turso/libSQL**：dev 用嵌入式（文件库），生产用 `TURSO_URL`/`TURSO_AUTH_TOKEN`
- **挂载卷注意事项**：git/vitest 前先清理 AppleDouble 垃圾：`find . -name "._*" -type f -delete`；push 前确保 LFS locksverify 已禁用

## 4. Issue 驱动开发流程

- 任务入口：https://github.com/gqf2008/staple/issues（按里程碑 Phase 1 → 5）
- **只做当前 issue**，不越级；每个 issue 内有"目标/范围/验收标准/参考"
- 完成一个 issue：验收标准全部满足 → 跑测试 → 推送分支 + 开 PR → 在 issue 评论说明验证结果并关联 PR，PR 合并后关闭 issue
- 需要新功能对照时查参考镜像 `gqf2008/paperclip`（`doc/SPEC-implementation.md` 是行为基准）

## 5. 代码组织

- 新代码全部放 **`crates/`**：
  - `crates/app` —— Topcoat 应用入口（路由、中间件、配置）
  - `crates/domain` —— 领域模型与业务规则（纯 Rust，无 I/O）
  - `crates/data` —— Turso/libSQL 数据层（schema、迁移、repository）
  - `crates/adapters` —— 适配器契约与实现（Phase 4）
- 仓库里的 `server/`、`ui/`、`packages/` 是 **Node 参考快照**：只读，除非"复制参考代码"场景；实现完对应模块后由维护者删除

## 6. 提交与 PR

- 提交信息：上游风格 `feat(scope): 描述` / `fix(scope): 描述` / `chore(scope): 描述`
- PR 必须填 `.github/PULL_REQUEST_TEMPLATE.md` 全部章节（含 Verification、Model Used）
- 推送/PR 用 gqf2008 账户：`gh auth switch --user gqf2008`

## 7. 验证标准

- 每个 PR 必须通过：`cargo fmt --check`、`cargo clippy -- -D warnings`、`cargo test`
- 涉及 API 的改动附 curl 验证命令
- CI 绿后才算完成

## 8. 与上游的关系

- 不拉取、不合并上游代码；上游新功能通过 `doc/plans/parity-checklist.md`（Phase 5 issue #25）登记为需求
- 技术问题先查 Topcoat/Turso 官方文档与本地源码，再查参考镜像
