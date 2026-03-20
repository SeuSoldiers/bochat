# ✅ 注册功能 400 错误修复

## 问题分析

用户在前端注册时收到 HTTP 400 (Bad Request) 错误。

### 根本原因
后端 `RegisterRequest` 模型需要三个字段：
```rust
pub struct RegisterRequest {
    pub name: String,        // 必需
    pub id_number: String,   // 必需
    pub phone: String,       // 必需
}
```

但前端注册表单只提供了两个字段（`phone` 和 `idNumber`），缺少 `name` 字段，导致 JSON 反序列化失败，返回 400 错误。

## 修复方案

### 后端修改 (3 个文件)

#### 1. `/src/models/user.rs` - 使 name 字段可选
```rust
pub struct RegisterRequest {
    #[serde(default)]
    pub name: Option<String>,  // 改为可选
    pub id_number: String,
    pub phone: String,
}
```

#### 2. `/src/handlers/auth.rs` - 处理缺失的 name
- 验证改为只检查 `id_number` 和 `phone`（`name` 现在可选）
- 如果没有提供 `name`，自动生成默认名字
  - 默认格式：`用户{手机号后4位}`
  - 例如：手机号是 13812345678，则名字为 `用户5678`
- 使用生成的 `name` 创建用户记录和默认 Bot

### 前端修改 (1 个文件)

#### 3. `/ui-vue/src/services/auth.ts` - 改进注册流程
```typescript
export async function register(phone: string, idNumber: string) {
  const response = await apiClient.post<User>('/auth/register', {
    phone,
    id_number: idNumber,
    // ❌ 不需要发送 name
  })
  return response
}
```

#### 4. `/ui-vue/src/stores/auth.ts` - 注册后自动登录
- 注册成功后立即调用登录接口
- 登录返回 token 和完整用户信息
- 自动保存 token 和 user 到 localStorage
- 实现无缝注册-登录流程

## 修复效果

✅ **注册流程改进**
- 用户无需输入名字，简化注册流程
- 系统自动为用户分配默认名字
- 注册成功后自动登录，无需再次输入凭证

✅ **错误消息改进**
- 清晰的错误提示：`"缺少必填字段（身份证号和手机号为必需）"`
- 前端捕获并显示所有错误信息

✅ **数据一致性**
- Token 和 user 信息正确保存到 localStorage
- 刷新页面后仍保持登录状态

## 编译验证

```
后端: ✅ cargo check - 通过
前端: ✅ npm run build - 成功
```

## 测试步骤

### 快速验证
1. 打开浏览器
2. 访问 http://localhost:5173/login
3. 点击"注 册"标签
4. 输入以下信息：
   - 手机号: 13812345678
   - 身份证号: 110101199003071234 (18位)
5. 点击"注 册"按钮
6. **预期结果**: ✅ 注册成功，自动登录，进入首页

### 完整验证清单
- [ ] 注册页面正常显示
- [ ] 输入手机号和身份证号
- [ ] 点击注册按钮
- [ ] 收到成功响应（无 400 错误）
- [ ] 自动跳转到首页
- [ ] localStorage 中保存了 `bot_token` 和 `user`
- [ ] 刷新页面仍保持登录状态

## 相关文件修改清单

### 后端
- `/src/models/user.rs` - RegisterRequest 模型修改
- `/src/handlers/auth.rs` - 注册处理器改进

### 前端
- `/ui-vue/src/services/auth.ts` - 注册服务改进
- `/ui-vue/src/stores/auth.ts` - 注册状态管理改进

## 问题排查

### 如果仍然出现 400 错误
1. 检查后端日志：`RUST_LOG=debug cargo run --release`
2. 验证身份证号长度是否为 18 位
3. 检查浏览器开发者工具 Network 标签查看请求体

### 如果身份证号验证失败
- 身份证号必须是 18 位数字
- 最后一位可以是 X（大写）
- 示例有效身份证号：`110101199003071234`

---

修复完成时间: 2026-03-20
修复内容: 注册 400 错误 + 自动登录流程优化
