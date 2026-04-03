export function getErrorMessage(error: any, fallback: string): string {
  const code = error?.code as string | undefined
  const status = error?.status as number | undefined
  const message = error?.message as string | undefined

  const codeMessageMap: Record<string, string> = {
    user_token_required: '请先登录后再操作',
    invalid_user_token: '登录状态已过期，请重新登录',
    bot_token_required: '请选择一个可用的 Bot',
    invalid_bot_token: 'Bot 凭证已失效，请重新选择 Bot',
    bot_ownership_mismatch: '只能操作自己名下的 Bot',
    bot_not_in_group: '该 Bot 还不在当前群里',
    bot_inactive: '目标 Bot 未激活',
    no_available_bot: '当前账号没有可用的 Bot',
    id_number_conflict: '该身份证号已注册',
    phone_conflict: '该手机号已注册',
    account_conflict: '该账号已注册',
    invalid_credentials: '账号或密码错误',
    invalid_id_number: '身份证号格式不正确',
    file_too_large: '文件过大，请上传更小的文件',
    invalid_file_format: '文件格式不支持',
    bot_not_found: 'Bot 不存在或已被删除',
    user_not_found: '用户不存在或登录信息不正确',
    forbidden: message || '没有权限执行该操作',
    bad_request: message || fallback,
  }

  if (code && codeMessageMap[code]) {
    return codeMessageMap[code]
  }

  if (status === 413) {
    return '文件过大，请上传更小的文件'
  }

  if (status === 404) {
    return message || '请求的资源不存在'
  }

  if (status === 403) {
    return message || '没有权限执行该操作'
  }

  if (status === 400) {
    return message || fallback
  }

  return message || fallback
}
