# Aster IR 与 AI 工具接口规范

**文档编号：** ASTER-IR-001
**版本：** V0.1 Draft
**状态：** 草案。
**依赖：** ASTER-ARCH-001（ADR-004、ADR-010）、01 语言规范、02 UI 规范

---

## 1. 范围与目标

本规范定义两件事：

1. **统一中间表示（IR）的分层与标识规则。** 编译器、布局检查器、调试器、AI 工具、未来的 Studio 都读取同一套结构，而不是各自解析源码。
2. **AI 工具接口。** 规定 CLI 输出、诊断格式、运行时检查、结构化补丁与自动修复闭环的协议。

本规范遵循基础设计文档原则 1（AI 是一等开发者）与原则 4（可观测性优先于猜测）。核心要求：AI 通过结构化数据读取事实、提交局部修改、运行验证，而不依赖截图或终端文本。

## 2. IR 分层

| 层 | 名称 | 内容 | 主要消费者 |
|---|---|---|---|
| L0 | 源（Source） | 原始文本、文件表、行列索引 | 格式化器、补丁写入器 |
| L1 | 语法树（CST/AST） | 保留注释与空白的无损语法树，每个节点带 span | 格式化器、语法诊断、补丁定位 |
| L2 | 语义模型（HIR） | 已解析名称、类型标注、状态与事件引用 | 类型检查、影响分析 |
| L3 | UI 模型（UI IR） | 组件树、属性、绑定、事件、样式来源 | 布局、检查器、AI 工具 |
| L4 | 布局结果 | 每个节点的矩形、测量来源、约束与冲突 | 检查器、布局测试、诊断 |
| L5 | 显示列表 | 绘制命令序列（见 02 §11） | 渲染后端 |
| 辅助 | 调试元数据 | 源位置映射、运行时事件时间线、状态快照 | 调试器、诊断协议 |

规则：

- 每层只能依赖更低编号的层（L3 不依赖 L5）。逆向依赖由工具链拒绝。
- L1 必须无损：从 L1 可以无歧义地还原源文件（用于补丁写入后的格式保持）。
- 每层输出带 `ir_version` 字段。版本规则见 §10。
- 所有层的序列化格式为 JSON（用于工具与测试），内部可以使用二进制缓存，但二进制格式不属于对外协议。

## 3. 稳定标识（ID）

### 3.1 组件 ID

- 显式声明的 `id`（见 02 §3.1）直接作为组件 ID。
- 未声明时，编译器生成 ID，格式为：`<父ID>/<种类>[<key或序号>]`，例如 `main/Column/Row[path=/home/a]`。
- 带 `key` 的循环项使用 `key` 值，保证元素移动后 ID 不变。
- 不带 `key` 的生成 ID 使用序号，插入或删除会导致后续 ID 变化。这一限制必须在诊断中提示（`UNSTABLE_ID`，级别 info）。

### 3.2 ID 的写回

**V0.1 不把生成的 ID 写回源码。** 原因：写回会改变源码，造成 AI 的每次读取都引起 diff 噪声。替代做法是：

- AI 对未声明 ID 的组件做补丁时，必须使用 `inspect` 返回的 `source` span 定位，或者先由 AI 添加显式 `id`。
- 补丁协议中的"添加 `id`"是合法操作（见 §6）。

### 3.3 其他标识

| 对象 | 标识 |
|---|---|
| 状态字段 | `state.<字段路径>` |
| 动作 | `action.<名称>` |
| 令牌 | `token.<分组>.<名称>` |
| 文件 | 相对项目根的路径 |
| 源位置 | `file:line:column`（1 起始，列以 Unicode 码点计数） |

## 4. 诊断格式

所有工具输出的诊断使用同一结构：

```json
{
  "protocol_version": "0.1",
  "code": "LAYOUT_CONFLICT",
  "severity": "error",
  "message": "sum of child min widths exceeds available width",
  "source": {
    "file": "app/main.as",
    "line": 12,
    "column": 9,
    "end_line": 12,
    "end_column": 30
  },
  "component": "sidebar",
  "property": "min_width",
  "actual": 736,
  "expected": 720,
  "overflow": 16,
  "related": [
    {"component": "content", "property": "min_width", "value": 400, "source": "app/main.as:18:9"}
  ],
  "suggestions": [
    {"id": "S1", "text": "lower content.min_width", "kind": "edit", "confidence": "medium"},
    {"id": "S2", "text": "allow sidebar to collapse", "kind": "design", "confidence": "low"}
  ]
}
```

字段规则：

| 字段 | 必需 | 说明 |
|---|---|---|
| `protocol_version` | 是 | 当前为 `0.1` |
| `code` | 是 | 稳定错误码，见 §5 |
| `severity` | 是 | `error` / `warning` / `info` |
| `message` | 是 | 面向人类的简述。**源文本必须为英语**（见 06 §3.0）。若用户选择了本地化语言，工具可以输出译文；`code` 始终是稳定标识，不随语言变化（见 §4.1） |
| `source` | 编译类诊断必需 | 源位置；运行时诊断若无法映射则为 `null`，并附 `runtime_path` |
| `component`、`property` | 视情况 | 相关组件与属性 |
| `actual`、`expected` | 视情况 | 数值型诊断的实际值与期望值 |
| `related` | 可选 | 参与原因的其他约束或位置 |
| `suggestions` | 可选 | 建议的修复，`kind` 为 `edit`（可自动生成补丁）、`design`（需要人工判断） |

**规则：**

- AI 修复时优先使用 `kind: edit` 的建议，并且必须转换为补丁（§6）后经验证。
- 诊断不包含截图或图像数据。
- 同一个根因产生的诊断应合并，并在 `related` 中互相引用，避免一次修改引出上百条重复错误。

### 4.1 诊断的本地化

- 编译器与工具链的消息模板以英语为源文本，存放在码表中，每个 `code` 对应一个英语模板。
- 译文存放在工具链的消息语言包中（格式与应用语言包相同，见 02 §9.1），缺失时回退到英语。
- **AI 与自动化工具必须使用英语输出或显式请求语言。** 默认输出英语，以保证 AI 读取的诊断在不同用户环境下一致。`--locale <tag>` 参数用于请求译文。
- 无论语言如何，`code`、`source`、`actual`、`expected`、`suggestions` 的结构和标识符保持不变。修复逻辑只能依赖结构化字段，不得解析消息文本。

## 5. 错误码

### 5.1 编码规则

| 前缀 | 范围 | 示例 |
|---|---|---|
| `E1xxx` | 词法与语法 | `E1001` |
| `E2xxx` | 名称、模块、类型 | `E2301` |
| `E3xxx` | 状态、动作、事件、组件元数据 | `E3001` |
| `E4xxx` | 布局、样式、令牌 | `E4002` |
| `E5xxx` | 权限、依赖、包 | `E5001` |
| `E9xxx` | 内部错误（编译器缺陷） | `E9001` |
| 名称 | 运行时诊断（大写英文标识，不带数字） | `LAYOUT_CONFLICT` |
| `W` 前缀 | 警告，编号规则相同 | `W1001` |

- 错误码一经发布不得复用或改变含义。废弃时保留码并标注 `deprecated`。
- 每个错误码必须在码表（机器可读 JSON）中有：含义、默认级别、示例、修复建议类别。码表与编译器同源生成，并由测试验证每个码都有至少一个反例测试。

### 5.2 内部错误

`E9xxx` 表示编译器自身缺陷，不是用户代码问题。AI 遇到 `E9xxx` 时不得尝试修改用户代码来"绕过"，必须报告为工具缺陷，并附上 `aster diagnose --bundle` 生成的诊断包。

## 6. 结构化补丁

AI 修改代码时，不通过整文件重写，而是提交结构化补丁。这样修改范围可以被验证，并且可以在提交前预测影响。

### 6.1 补丁操作（V0.1）

| 操作 | 参数 | 说明 |
|---|---|---|
| `set_prop` | `target`、`name`、`value`、`expect`（可选） | 设置属性 |
| `add_prop` | `target`、`name`、`value` | 属性不存在时添加 |
| `remove_prop` | `target`、`name` | 删除属性，恢复默认值 |
| `set_id` | `target`、`id` | 为未声明 ID 的组件添加显式 ID |
| `add_node` | `parent`、`position`、`source` | 插入组件，`source` 为 Aster 源代码片段 |
| `remove_node` | `target` | 删除组件 |
| `move_node` | `target`、`parent`、`position` | 移动组件 |
| `set_token` | `path`、`value` | 修改令牌值 |

`expect` 用于乐观并发：若目标当前值与 `expect` 不符，补丁被拒绝并报告 `PATCH_CONFLICT`，避免覆盖其他修改。

### 6.2 补丁示例：侧栏宽度 240 → 280

```json
{
  "protocol_version": "0.1",
  "base_revision": "rev-1f3a",
  "ops": [
    {
      "op": "set_prop",
      "target": "sidebar",
      "name": "width",
      "value": "280dp",
      "expect": "240dp"
    }
  ]
}
```

### 6.3 补丁的执行语义

1. **检查基线**：`base_revision` 与当前项目修订号不一致时，拒绝执行（`PATCH_STALE`）。
2. **干运行（dry run）**：返回预计的文本差异、语义差异（§7）、以及重新检查后的诊断增量。
3. **应用**：全部操作成功才写入；任何一条失败则整个补丁回滚。
4. **验证**：应用后自动执行 `check`。若新增 `error` 级诊断，则自动回滚，并返回失败原因。
5. **写入格式**：通过格式化器写回，保持未修改部分的格式不变。

补丁是原子的。AI 不得依赖"部分成功"。

### 6.4 补丁范围限制

- 补丁只能修改项目目录内的 `.as` 文件与清单中声明的资源。
- 补丁不能修改 `aster.lock`、权限声明或安全相关配置。这些修改必须由人类通过普通编辑完成（见 04 §9）。
- 补丁不能修改测试的期望值，除非提交人明确设置 `allow_expectation_change: true`，并且该修改会被标记在审查记录中（对应设计讨论 §七 硬规则 5：AI 不能自行修改验收标准）。

## 7. 语义差异与影响分析

### 7.1 语义差异

`aster diff <旧修订> <新修订>` 输出的不是文本 diff，而是 IR 层面的差异：

```json
{
  "added": [],
  "removed": [],
  "changed": [
    {
      "id": "sidebar",
      "property": "width",
      "before": "240dp",
      "after": "280dp",
      "layout_effect": {"content.width": {"before": 560, "after": 520}}
    }
  ]
}
```

- `layout_effect` 列出该修改影响的其他组件的布局结果。需要布局结果时，`diff` 会在两个修订上分别计算布局。

### 7.2 影响分析

`aster impact <目标>` 返回：

- 依赖该组件或属性的组件（反向依赖）；
- 读取相关状态字段的绑定与事件；
- 可能受影响的布局测试与快照测试；
- 建议的最小重新检查范围（对应增量编译）。

这一接口使 AI 不必每次都重新理解整个项目（设计讨论中的"影响范围"能力）。

## 8. 协议与命令

### 8.1 CLI

| 命令 | 作用 | 输出 |
|---|---|---|
| `aster check` | 语法、类型、布局静态检查 | 诊断列表 |
| `aster build` | 构建产物 | 构建报告（产物路径、哈希、诊断） |
| `aster run` | 运行应用（开发模式） | 运行状态；可连接检查接口 |
| `aster test` | 运行测试 | 测试结果 |
| `aster fmt` | 官方格式化 | 无输出或 `--check` 结果 |
| `aster inspect tree\|layout\|state\|events` | 读取运行或静态模型 | 对应 JSON |
| `aster diagnose` | 收集诊断包，用于工具缺陷报告 | 诊断包路径 |
| `aster patch` | 应用或预演补丁 | 补丁结果 |
| `aster diff` / `aster impact` | 语义差异与影响分析 | JSON |

通用规则：

- 所有命令支持 `--format json`。缺省为人类可读文本。
- JSON 输出为单个对象，或 NDJSON（每行一个对象，用于流式诊断）。
- 退出码：`0` 成功；`1` 存在 error 级诊断或测试失败；`2` 用法错误；`3` 内部错误（`E9xxx`）。

### 8.2 常驻协议服务

AI 工具与未来的 Studio 通过 `aster serve --stdio` 连接，协议为 **JSON-RPC 2.0**，消息以换行分隔。

V0.1 方法：

| 方法 | 参数 | 结果 |
|---|---|---|
| `project.info` | 无 | 项目名、语言版本、清单摘要、修订号 |
| `check` | 可选：文件列表 | 诊断列表 |
| `inspect.tree` | 可选：根 ID | 组件树（ID、种类、属性、source） |
| `inspect.layout` | `window: {width, height, scale}` | 每个组件的矩形与约束状态 |
| `inspect.state` | 可选：字段路径 | 当前状态快照 |
| `inspect.events` | `since`（时间戳或序号） | 事件时间线 |
| `diagnose` | 无 | 当前全部诊断（含运行时诊断） |
| `patch.dry_run` | 补丁 | 文本差异、语义差异、诊断增量 |
| `patch.apply` | 补丁 | 应用结果或回滚原因 |
| `test.run` | 可选：测试名 | 测试结果 |
| `run.start` / `run.stop` | 无 | 运行状态 |

通知（服务端推送）：`diagnostics.changed`、`state.changed`（需订阅）、`run.exited`。

### 8.3 运行时布局检查

`inspect.layout` 必须返回：

```json
{
  "window": {"width": 720, "height": 500, "scale": 1.0},
  "nodes": [
    {
      "id": "sidebar",
      "rect": {"x": 0, "y": 0, "width": 240, "height": 500},
      "size_sources": {"width": "explicit:main.as:14:9", "height": "fill:parent"},
      "clipped": false,
      "overflow": {"x": 0, "y": 0},
      "status": "ok"
    }
  ],
  "fonts": [{"family": "PingFang SC", "fallback_used": false}]
}
```

- `size_sources` 必须说明每个尺寸的来源，对应 02 §4 中"每个尺寸的来源"的要求。
- 不允许只返回图片。截图可以作为可选附件（`screenshot_path`），但不是检查的依据。

## 9. 自动修复闭环

标准流程（用于 EXP-004 与日常 AI 开发）：

```
1. project.info          读取项目与修订号
2. check / diagnose      获取诊断
3. inspect.*             定位相关组件与事实
4. patch.dry_run         预演修改
5. patch.apply           应用（自动回滚保护）
6. check                 确认无新增 error
7. test.run              运行测试与布局快照
8. 结束或回到 2
```

**成功的判定**：

- 原有诊断中的目标错误已消失；
- 无新增 error 级诊断；
- 全部相关测试通过，且布局快照与期望一致；
- 修复次数与耗时被记录。

AI 不得自行修改快照期望值使测试通过。快照更新只能由人类通过 `aster test --update-snapshots` 执行，并生成需审查的差异。

**失败处理**：连续 3 次修复失败，或出现 `E9xxx`，闭环必须停止并把诊断包交给人类。

## 10. 版本

- `protocol_version` 与 `ir_version` 独立维护。
- 新增可选字段为次版本号增加（`0.1` → `0.2`）。
- 删除字段、改变字段含义或改变错误码含义为主版本变化；在 V0.x 阶段也必须在变更日志中说明。
- 客户端遇到未知字段必须忽略，遇到不支持的主版本必须报错，而不是猜测。

## 11. 安全与隔离

- 协议服务只监听标准输入输出或本机 Unix Socket / 命名管道，不监听网络端口。
- `patch`、`run`、`test` 需要用户在清单或配置中授权（见 04 文档）。AI 工具不得默认拥有执行权限。
- 诊断包可能包含源码与路径，导出前必须由用户确认，并提供脱敏选项（用户名、绝对路径）。

## 12. 验收

本规范对应以下可验证的验收条件：

1. 每条诊断都能通过 JSON 模式解析，并通过 schema 测试。
2. 每个错误码都有反例测试，且该测试检查码与源位置。
3. 补丁在基线不匹配时被拒绝；在引入新 error 时被自动回滚。
4. `inspect.layout` 对相同输入给出逐字节相同的输出（确定性）。
5. 完成 EXP-004 时，闭环的成功率、修复次数、耗时与回归缺陷被记录，不只记录成功案例。

## 13. 未纳入 V0.1 的内容

- Studio 的可视化设计器与源码双向同步。
- 多人协作的实时同步。
- 远程协议服务（网络传输）。
- 基于 LLM 的自动补丁生成本身（本规范只定义 AI 与工具之间的接口，不定义模型）。
- 热更新协议（属于 V0.2 AI 调试阶段的后续设计）。
