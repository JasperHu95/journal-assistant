# Journal Assistant 代码评审报告

评审日期：2026-07-19
评审方：Hermes Agent、Claude Code、OpenCode

---

## [Hermes Agent 评审]

### 总分：79/100

### 各维度得分

1. **架构设计：16/20**
   - Rust 后端模块划分清晰：fetcher（HTTP）、parser（解析）、discovery（发现）、extract（提取）职责单一
   - 前端数据库层（db.ts）与 UI 分离合理
   - 不足：前端路由用手动 if/else，无 URL 状态管理

2. **代码质量：16/20**
   - Rust 错误处理完整，全部使用 Result + map_err，无生产代码 unwrap
   - 单元测试覆盖 parser、fetcher、discovery、extract 核心路径
   - 不足：前端 handleExtract/handleTranslate 在两个页面重复

3. **安全性：14/20**
   - SSRF 防护已实现（extract.rs），拒绝私有地址
   - 全部 SQL 使用参数化查询
   - HTTP 有 UA、超时、重定向限制
   - 不足：CSP 禁用（`"csp": null`），API Key 明文存储，fetch_text 的 URL 未做 SSRF 校验

4. **用户体验：16/20**
   - i18n 完整覆盖中英文
   - 已读/未读、提取摘要、翻译功能齐全
   - 不足：删除操作无确认对话框，日期未格式化

5. **可维护性：17/20**
   - Rust 注释质量高，函数文档完整
   - 命名规范一致（Rust snake_case / TS camelCase）
   - CSS 配色体系统一（QJE 学术复古风格）
   - 不足：i18n 响应式样板重复

### 亮点
- RSS 编码处理（BOM → XML 声明 → HTTP charset → GB2312/GBK）是生产级实现
- 翻译竞态条件处理（articleId 校验）细致
- Kimi + Claude 协作开发模式有效，Claude 多次发现实际 bug

### 不足
- CSP 关闭是安全隐患
- 前端组件重复（详情面板、i18n 样板）
- 无前端测试

---

## [Claude Code 评审]

### 总分：76/100

### 各维度得分

1. **架构设计：16/20**
   - Rust 后端模块化良好，各模块职责单一
   - Tauri command 薄包装模式一致
   - 不足：前端无组件化，路由简陋

2. **代码质量：15/20**
   - Rust 错误处理优秀，无 panic 路径
   - 测试覆盖 RSS 解析和编码转换
   - 不足：前端代码重复严重，categories 类型在三层不统一

3. **安全性：14/20**
   - SSRF 防护已实现但不完整（仅 extract 有，fetch_text/discovery 无）
   - SQL 参数化查询安全
   - 不足：CSP 关闭、API Key 明文、172.x 前缀匹配过宽

4. **用户体验：15/20**
   - 功能完整：RSS 管理、期刊分类、摘要提取、AI 翻译
   - 竞态条件处理到位
   - 不足：错误信息直接暴露 Rust 堆栈、无分页、日期未格式化

5. **可维护性：16/20**
   - Rust 注释质量高，测试覆盖关键路径
   - 命名规范一致
   - 不足：i18n 响应式 hack 无文档、前端零测试

### 亮点
- 编码处理覆盖 GB2312/GBK/Big5，适合中文学术场景
- parser 对 Atom 多 link、RSS guid 回退的处理比多数开源 RSS 阅读器更细致

### 不足
- SSRF 防护应扩展到所有网络请求入口
- 前端应提取 ArticleDetail 共享组件
- 应恢复基本 CSP 配置

---

## [OpenCode 评审]

### 总分：75/100

### 各维度得分

1. **架构设计：15/20**
   - 模块职责清晰，fetcher/parser/discovery/extract 无跨层依赖
   - 前端路由简陋，无 URL 状态管理
   - 不足：前端组件化不足，shared logic 重复

2. **代码质量：15/20**
   - Rust 错误处理完整，Result 链路清晰
   - 测试覆盖关键路径
   - 不足：categories 在 Rust/SQLite/TS 三层类型不统一

3. **安全性：13/20**
   - SSRF 防护仅限 extract，IPv6 未覆盖
   - API Key 明文存储
   - CSP 禁用
   - 不足：translate_text 无输入长度限制

4. **用户体验：14/20**
   - i18n 完整，已读/未读视觉区分清晰
   - 翻译竞态处理到位
   - 不足：无骨架屏、无确认对话框、日期未本地化

5. **可维护性：15/20**
   - 注释详尽，命名规范
   - CSS 配色体系统一
   - 不足：i18n 响应式模式重复、详情面板代码重复

### 亮点
- 编码处理是生产级实现
- 模块职责划分比许多开源项目更清晰

### 不足
- SSRF 防护不完整
- 类型不一致增加维护成本
- 路由应改用 svelte-spa-router

---

## 综合评分

| 评审方 | 架构 | 代码 | 安全 | 体验 | 可维护 | 总分 |
|--------|------|------|------|------|--------|------|
| Hermes | 16 | 16 | 14 | 16 | 17 | **79** |
| Claude | 16 | 15 | 14 | 15 | 16 | **76** |
| OpenCode | 15 | 15 | 13 | 14 | 15 | **72** |
| **平均** | 15.7 | 15.3 | 13.7 | 15.0 | 16.0 | **75.7** |

## 共识亮点

1. RSS 编码处理（GB2312/GBK/Big5 → UTF-8）是生产级实现
2. Rust 后端模块化清晰，错误处理优秀
3. 翻译竞态条件处理细致
4. QJE 学术复古风格 UI 一致

## 共识不足

1. SSRF 防护不完整（仅 extract 有，fetcher/discovery 无）
2. CSP 禁用、API Key 明文存储
3. 前端组件重复（详情面板、i18n 样板）
4. 无前端测试
5. 日期未格式化、删除无确认

## 优先改进建议

**P0（安全）**：扩展 SSRF 校验到所有网络请求、恢复 CSP、API Key 加密存储
**P1（重构）**：提取 ArticleDetail 共享组件、封装 i18n hook
**P2（体验）**：日期格式化、删除确认、toast 提示
**P3（测试）**：添加前端 E2E 测试
