# 注册与个人资料说明

这份说明替代旧的“注册 400 修复”记录，当前以前后端现状为准。

## 当前认证规则

- 注册和登录支持 `phone` / `id_number` 二选一，至少填写一项
- `name` 不强制，后端会自动生成 `用户-xxxxxxxx` 形式的默认昵称
- 注册和登录都会返回用户级 `token`
- 认证与用户资料响应不再暴露内部 `user_id`

注册示例：

```json
{
  "phone": "13800138000"
}
```

或：

```json
{
  "id_number": "110101199003071234"
}
```

## 前端配套行为

- 登录页支持手机号 / 身份证号切换输入
- 注册页不再强制要求用户名
- 登录态保存的是用户 token，不是 bot token
- 顶部导航提供“个人信息”入口
- 个人信息页可以编辑：
  - `name`
  - `phone`
  - `avatar_url`

## 个人信息接口

- `GET /api/v1/users/me`
- `PUT /api/v1/users/me`

更新规则：

- `name` 可以修改，但不能为空字符串
- `phone` 为当前资料页可编辑的登录手机号
- 头像可以直接填 URL，也可以先走文件上传接口后回填 `avatar_url`

## 相关文档

- [README.md](/home/harkerhand/codes/rust-bochat/README.md)
- [API指南.md](/home/harkerhand/codes/rust-bochat/API指南.md)
- [BOT接入指南.md](/home/harkerhand/codes/rust-bochat/BOT接入指南.md)
