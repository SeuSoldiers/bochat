#!/usr/bin/env python3
"""
聊天平台 API 测试脚本

这个脚本演示了如何：
1. 注册两个用户
2. 登录获取 Token
3. 用户之间相互发送消息
"""

import requests
import json
import time
from typing import Dict, Any

# 配置
BASE_URL = "http://127.0.0.1:8080"
API_VERSION = "v1"

class ChatPlatformTester:
    """聊天平台 API 测试类"""

    def __init__(self, base_url: str = BASE_URL):
        self.base_url = base_url
        self.users = {}  # 存储用户信息

    def _make_request(self, method: str, endpoint: str, data: Dict = None, token: str = None) -> Dict[str, Any]:
        """
        发送 HTTP 请求

        Args:
            method: HTTP 方法 (GET, POST, etc.)
            endpoint: API 端点
            data: 请求体数据
            token: Bearer Token

        Returns:
            响应 JSON 数据
        """
        url = f"{self.base_url}/api/{API_VERSION}{endpoint}"
        headers = {"Content-Type": "application/json"}

        if token:
            headers["Authorization"] = f"Bearer {token}"

        print(f"\n📤 {method.upper()} {endpoint}")
        if data:
            print(f"   请求数据: {json.dumps(data, ensure_ascii=False, indent=2)}")

        try:
            if method.upper() == "GET":
                response = requests.get(url, headers=headers, timeout=10)
            elif method.upper() == "POST":
                response = requests.post(url, json=data, headers=headers, timeout=10)
            else:
                raise ValueError(f"不支持的方法: {method}")

            response.raise_for_status()
            result = response.json()

            print(f"✅ 响应 ({response.status_code}): {json.dumps(result, ensure_ascii=False, indent=2)}")
            return result

        except requests.exceptions.ConnectionError:
            print(f"❌ 连接错误: 无法连接到 {url}")
            print("   请确保服务器运行中: cargo run")
            raise
        except requests.exceptions.HTTPError as e:
            print(f"❌ HTTP 错误: {e.response.status_code}")
            try:
                error_data = e.response.json()
                print(f"   错误详情: {json.dumps(error_data, ensure_ascii=False, indent=2)}")
            except:
                print(f"   响应: {e.response.text}")
            raise
        except json.JSONDecodeError as e:
            print(f"❌ JSON 解析错误: {e}")
            print(f"   响应内容: {response.text}")
            raise

    def register_user(self, username: str, email: str, password: str) -> Dict[str, Any]:
        """
        注册新用户

        Args:
            username: 用户名
            email: 邮箱
            password: 密码

        Returns:
            用户信息和 Token
        """
        print(f"\n{'='*60}")
        print(f"📝 注册用户: {username}")
        print(f"{'='*60}")

        data = {
            "username": username,
            "email": email,
            "password": password
        }

        result = self._make_request("POST", "/auth/register", data)

        # 保存用户信息
        self.users[username] = {
            "user_id": result["user_id"],
            "bot_id": result["bot_id"],
            "token": result["token"],
            "username": username,
            "email": email
        }

        print(f"✨ 用户 {username} 注册成功!")
        print(f"   用户ID: {result['user_id']}")
        print(f"   机器人ID: {result['bot_id']}")
        print(f"   Token: {result['token'][:50]}...")

        return result

    def login_user(self, username: str, password: str) -> Dict[str, Any]:
        """
        用户登录

        Args:
            username: 用户名
            password: 密码

        Returns:
            登录结果
        """
        print(f"\n{'='*60}")
        print(f"🔐 登录用户: {username}")
        print(f"{'='*60}")

        data = {
            "username": username,
            "password": password
        }

        result = self._make_request("POST", "/auth/login", data)

        # 更新 Token
        if username in self.users:
            self.users[username]["token"] = result["token"]

        print(f"✨ 用户 {username} 登录成功!")
        print(f"   新 Token: {result['token'][:50]}...")

        return result

    def send_message(self, from_user: str, to_user: str, message: str) -> Dict[str, Any]:
        """
        发送消息

        Args:
            from_user: 发送者用户名
            to_user: 接收者用户名
            message: 消息内容

        Returns:
            发送结果
        """
        if from_user not in self.users:
            raise ValueError(f"用户 {from_user} 未注册或未保存")

        if to_user not in self.users:
            raise ValueError(f"用户 {to_user} 未注册或未保存")

        from_user_info = self.users[from_user]
        to_user_info = self.users[to_user]

        print(f"\n{'='*60}")
        print(f"💬 {from_user} → {to_user}: {message}")
        print(f"{'='*60}")

        data = {
            "to_id": to_user_info["bot_id"],
            "content": {"text": message},
            "msg_type": "text"
        }

        result = self._make_request(
            "POST",
            "/message/send",
            data,
            token=from_user_info["token"]
        )

        print(f"✨ 消息发送成功!")
        print(f"   消息ID: {result['msg_id']}")
        print(f"   发送时间: {result['created_at']}")

        return result

    def health_check(self) -> bool:
        """
        检查服务器健康状态

        Returns:
            True 如果服务器正常
        """
        print(f"\n🏥 检查服务器状态...")
        try:
            response = requests.get(f"{self.base_url}/health", timeout=5)
            response.raise_for_status()
            print(f"✅ 服务器正常运行 (响应: {response.text})")
            return True
        except Exception as e:
            print(f"❌ 服务器错误: {e}")
            return False


def main():
    """主函数"""
    print("🚀 聊天平台 API 测试脚本")
    print("=" * 60)

    # 创建测试器
    tester = ChatPlatformTester()

    # 检查服务器
    if not tester.health_check():
        print("\n⚠️  错误: 无法连接到服务器")
        print("   请先启动服务器: cd /home/harkerhand/codes/rust-bochat && cargo run")
        return

    try:
        # 阶段 1: 注册两个用户
        print(f"\n\n{'#'*60}")
        print("# 阶段 1: 注册两个用户")
        print(f"{'#'*60}")

        alice_info = tester.register_user(
            username="alice",
            email="alice@example.com",
            password="alice123"
        )
        time.sleep(1)  # 等待一秒

        bob_info = tester.register_user(
            username="bob",
            email="bob@example.com",
            password="bob123"
        )
        time.sleep(1)

        # 阶段 2: 相互发送消息
        print(f"\n\n{'#'*60}")
        print("# 阶段 2: 相互发送消息")
        print(f"{'#'*60}")

        # Alice 给 Bob 发消息
        tester.send_message(
            from_user="alice",
            to_user="bob",
            message="你好，Bob！这是我的第一条消息。"
        )
        time.sleep(1)

        tester.send_message(
            from_user="alice",
            to_user="bob",
            message="你在吗？"
        )
        time.sleep(1)

        # Bob 给 Alice 发消息
        tester.send_message(
            from_user="bob",
            to_user="alice",
            message="你好，Alice！我在这里。"
        )
        time.sleep(1)

        tester.send_message(
            from_user="bob",
            to_user="alice",
            message="很高兴认识你！"
        )
        time.sleep(1)

        # 阶段 3: 重复发送更多消息
        print(f"\n\n{'#'*60}")
        print("# 阶段 3: 继续交流")
        print(f"{'#'*60}")

        messages = [
            ("alice", "bob", "这个聊天平台真的很棒！"),
            ("bob", "alice", "是的，很高效！"),
            ("alice", "bob", "我们可以实现很多功能。"),
            ("bob", "alice", "同意！让我们继续开发。"),
        ]

        for from_user, to_user, message in messages:
            tester.send_message(from_user, to_user, message)
            time.sleep(0.5)

        # 总结
        print(f"\n\n{'='*60}")
        print("✨ 测试完成！")
        print(f"{'='*60}")
        print(f"\n📊 测试统计:")
        print(f"   注册用户数: 2")
        print(f"   发送消息数: {len(messages) + 4}")
        print(f"   总消息往返: {len(messages) + 4}")
        print(f"\n用户信息:")
        for username, user_info in tester.users.items():
            print(f"\n   用户: {username}")
            print(f"   - 用户ID: {user_info['user_id']}")
            print(f"   - 机器人ID: {user_info['bot_id']}")
            print(f"   - 邮箱: {user_info['email']}")

        print(f"\n✅ 所有测试都通过了！")

    except Exception as e:
        print(f"\n❌ 测试失败: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
