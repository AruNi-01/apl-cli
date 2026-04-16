# apl — Apollo Configuration Center CLI

针对 AI Coding 场景设计的 Apollo 配置中心命令行工具。让 AI Agent 能在编码过程中直接读取和管理 Apollo 动态配置，不再只能依赖默认值/空值来分析代码。

## 安装

### 一键安装（推荐，无需 Rust）

```bash
curl -fsSL https://raw.githubusercontent.com/AruNi-01/apl-cli/main/install.sh | sh
```

自动检测系统（macOS / Linux）和架构（x86_64 / aarch64），下载预编译二进制到 `~/.local/bin`。

### 从 Skill 安装

如果你希望让支持 Skill 的 AI Agent 自动完成 CLI 安装和初始化，可以先安装这个 Skill：

```bash
npx skills add https://github.com/AruNi-01/apl-cli
```

然后把下面这段 Prompt 发给 AI：

```text
使用 apl-cli skill，帮我安装对应 CLI，并完成初始化 setup。
```

AI 会先检查本机是否已经安装 `apl`；如果未安装，会自动执行官方安装脚本，然后继续引导你完成 Apollo 配置。

### 从源码安装

```bash
cargo install --git https://github.com/AruNi-01/apl-cli.git
```

### 验证

```bash
apl --version
```

## 快速开始

### 1. 获取 Token

在 Apollo Portal 中创建 Open API Token：

> Apollo Portal → 开放平台 → 创建第三方应用 → 授权 Namespace

### 2. 初始化配置

在项目根目录执行：

```bash
apl init \
  --portal-url "http://apollo-portal.your-company.com" \
  --token "your-open-api-token" \
  --app-id "YourAppId" \
  --operator "your-domain-account"
```

这会在当前目录生成 `.apollo-cli.toml`，每个项目独立配置，天然隔离。

### 3. 使用

```bash
# 查看所有 Namespace
apl ns

# 查看某个 Namespace 的全部配置
apl get application

# 查看指定 key（最常用 — 避免上下文污染）
apl get application --keys timeout,batch.size,retry.count

# 查看单个 key
apl get application timeout
```

## 命令一览


| 命令                           | 说明                         |
| ---------------------------- | -------------------------- |
| `apl init`                   | 生成 `.apollo-cli.toml` 配置文件 |
| `apl show`                   | 显示当前配置（token 脱敏）           |
| `apl envs`                   | 列出所有环境和集群                  |
| `apl ns`                     | 列出所有 Namespace             |
| `apl get <ns> [key]`         | 读取配置，支持 `--keys k1,k2` 过滤  |
| `apl set <ns> <key> <value>` | 创建或修改配置                    |
| `apl delete <ns> <key>`      | 删除配置                       |
| `apl publish <ns>`           | 发布 Namespace 变更            |


## 读取配置

```bash
# 全部配置
apl get application

# 指定多个 key
apl get application --keys timeout,max.retry

# 单个 key
apl get application timeout

# JSON 格式输出（AI Agent 推荐）
apl get application --keys timeout,batch --format json
# 输出: {"batch":"100","timeout":"3000"}

# 查询其他环境
apl get application --env FAT --format json
```

## 修改配置

```bash
# 修改一个值（会显示确认提示）
apl set application timeout 5000 --comment "increase timeout"

# 跳过确认（脚本/AI 调用时使用）
apl set application timeout 5000 --yes

# 修改后发布使其生效
apl publish application --title "update timeout"
```

**PRO 环境保护**：所有写操作（`set` / `delete` / `publish`）在 PRO 环境下会被自动拦截，需要通过 Apollo Portal 操作。

## 配置文件

文件位置：项目根目录 `.apollo-cli.toml`

```toml
portal_url       = "http://apollo-portal.your-company.com"
token            = "your-open-api-token"
default_env      = "UAT"
default_app_id   = "YourAppId"
default_cluster  = "default"
default_operator = "your-domain-account"
```

**配置优先级**：CLI 参数 > 环境变量 > 配置文件 > 默认值

支持的环境变量：


| 环境变量                | 对应配置            |
| ------------------- | --------------- |
| `APOLLO_PORTAL_URL` | portal_url      |
| `APOLLO_TOKEN`      | token           |
| `APOLLO_ENV`        | default_env     |
| `APOLLO_APP_ID`     | default_app_id  |
| `APOLLO_CLUSTER`    | default_cluster |


## 全局选项

所有命令均支持以下全局选项：

```
--portal-url <URL>     覆盖 Portal 地址
--token <TOKEN>        覆盖认证 Token
--env <ENV>            覆盖环境（DEV/FAT/UAT/PRO）
--app-id <ID>          覆盖 AppId
--cluster <NAME>       覆盖集群名（默认 default）
--format <text|json>   输出格式（默认 text）
```

## AI Agent 集成

配套 Skill 安装后位于 `~/.cursor/skills/apl-cli/SKILL.md`，AI Agent 会在以下场景自动使用：

- 代码中遇到 `@Value("${...}")` 或 `@ApolloJsonValue` 需要实际值
- 用户询问 Apollo 配置或动态配置
- 分析代码逻辑需要了解运行时配置（特性开关、阈值、URL 等）

Agent 始终使用 `--format json` 获取结构化输出，并通过 `--keys` 只取需要的 key，避免上下文污染。

## 发布新版本

推送 `v*` tag，GitHub Actions 自动编译 4 个平台 + 创建 Release：

```bash
git tag v0.1.0
git push origin v0.1.0
```


| 平台                            | 构建方式             |
| ----------------------------- | ---------------- |
| Linux x86_64                  | ubuntu-latest 原生 |
| Linux aarch64                 | cross 交叉编译       |
| macOS x86_64 (Intel)          | macos-15-intel 原生 |
| macOS aarch64 (Apple Silicon) | macos-latest 原生  |


## 项目结构

```
apl-cli/
├── .github/workflows/
│   └── release.yml     # CI: 打 tag → 编译 → 发 Release
├── Cargo.toml
├── install.sh          # 一键安装脚本
├── README.md
├── skills/
│   └── apl-cli/
│       └── SKILL.md    # AI Agent Skill（源文件）
└── src/
    ├── main.rs         # 入口
    ├── cli.rs          # 命令行定义（clap derive）
    ├── config.rs       # 配置文件加载与优先级合并
    ├── client.rs       # Apollo Open API HTTP 客户端
    ├── models.rs       # API 请求/响应模型
    ├── output.rs       # 输出格式化（text 表格 / json）
    └── commands.rs     # 所有命令实现
```

