#!/usr/bin/env python3
"""
聊天平台测试脚本 - 新架构与用户+Bot管理

本脚本演示新的身份认证系统：
- 用户使用身份证号注册（实名认证）
- 用户可以创建和管理多个Bot
- 只有Bot才能通过API发送消息
"""

import requests
import json
import sys
from typing import Optional, Dict, Any

class ChatPlatformTester:
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
        self.session = requests.Session()
        self.users: Dict[str, Dict[str, Any]] = {}
        self.bots: Dict[str, Dict[str, Any]] = {}

    def _make_request(
        self, method: str, endpoint: str, data: Optional[Dict] = None, token: Optional[str] = None
    ) -> requests.Response:
        """发送HTTP请求并处理错误。"""
        url = f"{self.base_url}{endpoint}"
        headers = {"Content-Type": "application/json"}

        if token:
            headers["Authorization"] = f"Bearer {token}"

        try:
            if method == "GET":
                response = self.session.get(url, headers=headers)
            elif method == "POST":
                response = self.session.post(url, json=data, headers=headers)
            elif method == "DELETE":
                response = self.session.delete(url, headers=headers)
            else:
                raise ValueError(f"不支持的方法: {method}")

            return response
        except requests.exceptions.ConnectionError:
            print("❌ 错误：无法连接到服务器。服务器是否在8080端口上运行？")
            sys.exit(1)
        except Exception as e:
            print(f"❌ 请求错误: {e}")
            sys.exit(1)

    def health_check(self) -> bool:
        """检查服务器是否运行。"""
        try:
            response = self._make_request("GET", "/health")
            return response.status_code == 200
        except:
            return False

    def register_user(self, name: str, id_number: str, phone: str) -> bool:
        """使用身份证号注册新用户。"""
        print(f"\n📝 正在注册用户: {name}")
        print(f"   身份证号: {id_number}")

        data = {
            "name": name,
            "id_number": id_number,
            "phone": phone
        }

        response = self._make_request("POST", "/api/v1/auth/register", data)

        if response.status_code == 201:
            result = response.json()
            user_id = result["user_id"]
            bot_id = result["bot_id"]
            bot_token = result["bot_token"]

            # 存储用户和Bot信息
            self.users[name] = {
                "user_id": user_id,
                "id_number": id_number,
                "phone": phone,
                "created_at": result["created_at"]
            }
            self.bots[bot_id] = {
                "bot_id": bot_id,
                "owner": name,
                "token": bot_token,
                "name": f"{name}的默认Bot",
                "status": "active"
            }

            print(f"✅ 用户注册成功！")
            print(f"   用户ID: {user_id}")
            print(f"   默认Bot ID: {bot_id}")
            return True
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ 注册失败: {error_msg}")
            if response.status_code == 409:
                print("   ℹ️  此身份证号已被注册")
            return False

    def create_bot(self, user_name: str, bot_name: str, description: Optional[str] = None) -> Optional[str]:
        """为用户创建新Bot。"""
        print(f"\n🤖 为 {user_name} 创建Bot")
        print(f"   Bot名称: {bot_name}")

        if user_name not in self.users:
            print(f"❌ 用户 {user_name} 未找到")
            return None

        # 获取用户的默认Bot令牌
        user_bots = [b for b in self.bots.values() if b["owner"] == user_name]
        if not user_bots:
            print(f"❌ 没有找到用户 {user_name} 的Bot")
            return None

        user_bot_token = user_bots[0]["token"]

        data = {
            "name": bot_name,
            "description": description or f"由 {user_name} 创建"
        }

        response = self._make_request("POST", "/api/v1/bots", data, user_bot_token)

        if response.status_code == 201:
            result = response.json()
            bot_id = result["bot_id"]
            bot_token = result["token"]

            self.bots[bot_id] = {
                "bot_id": bot_id,
                "owner": user_name,
                "token": bot_token,
                "name": bot_name,
                "status": result["status"]
            }

            print(f"✅ Bot创建成功！")
            print(f"   Bot ID: {bot_id}")
            return bot_id
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ Bot创建失败: {error_msg}")
            return None

    def list_user_bots(self, user_name: str) -> bool:
        """列出用户的所有Bot。"""
        print(f"\n📋 列出 {user_name} 的Bot")

        if user_name not in self.users:
            print(f"❌ 用户 {user_name} 未找到")
            return False

        # 获取用户的默认Bot令牌
        user_bots = [b for b in self.bots.values() if b["owner"] == user_name]
        if not user_bots:
            print(f"❌ 没有找到用户 {user_name} 的Bot")
            return False

        user_bot_token = user_bots[0]["token"]

        response = self._make_request("GET", "/api/v1/bots", token=user_bot_token)

        if response.status_code == 200:
            result = response.json()
            bots = result.get("bots", [])

            print(f"✅ 找到 {len(bots)} 个Bot：")
            for bot in bots:
                print(f"   - {bot['name']} ({bot['bot_id']})")
                print(f"     状态: {bot['status']}")
                if bot.get('description'):
                    print(f"     描述: {bot['description']}")
            return True
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ 获取Bot列表失败: {error_msg}")
            return False

    def send_message(self, from_user: str, to_user: str, content: str, from_bot_idx: int = 0, to_bot_idx: int = 0) -> bool:
        """从一个用户的Bot向另一个用户的Bot发送消息。"""
        print(f"\n💬 从 {from_user} 向 {to_user} 发送消息")
        print(f"   内容: {content}")

        # 获取发送者的Bot
        from_user_bots = [b for b in self.bots.values() if b["owner"] == from_user]
        if not from_user_bots or len(from_user_bots) <= from_bot_idx:
            print(f"❌ 未找到用户 {from_user} 的Bot")
            return False

        sender_bot = from_user_bots[from_bot_idx]
        sender_token = sender_bot["token"]

        # 获取接收者的Bot
        to_user_bots = [b for b in self.bots.values() if b["owner"] == to_user]
        if not to_user_bots or len(to_user_bots) <= to_bot_idx:
            print(f"❌ 未找到用户 {to_user} 的Bot")
            return False

        recipient_bot = to_user_bots[to_bot_idx]
        recipient_bot_id = recipient_bot["bot_id"]

        data = {
            "to_id": recipient_bot_id,
            "content": {"text": content},
            "msg_type": "text"
        }

        response = self._make_request("POST", "/api/v1/message/send", data, sender_token)

        if response.status_code == 201:
            result = response.json()
            msg_id = result["msg_id"]
            print(f"✅ 消息发送成功！(ID: {msg_id})")
            print(f"   从: {sender_bot['name']} ({sender_bot['bot_id']})")
            print(f"   至: {recipient_bot['name']} ({recipient_bot_id})")
            return True
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ 消息发送失败: {error_msg}")
            return False

    def delete_bot(self, user_name: str, bot_idx: int = 0) -> bool:
        """删除用户的Bot。"""
        if user_name not in self.users:
            print(f"❌ 用户 {user_name} 未找到")
            return False

        user_bots = [b for b in self.bots.values() if b["owner"] == user_name]
        if not user_bots or len(user_bots) <= bot_idx:
            print(f"❌ 未找到用户 {user_name} 的Bot")
            return False

        bot_to_delete = user_bots[bot_idx]
        bot_token = user_bots[0]["token"]  # Use first bot's token to authenticate

        print(f"\n🗑️  删除Bot: {bot_to_delete['name']}")

        response = self._make_request(
            "DELETE",
            f"/api/v1/bots/{bot_to_delete['bot_id']}",
            token=bot_token
        )

        if response.status_code == 200:
            print(f"✅ Bot删除成功！")
            # Remove from local storage
            del self.bots[bot_to_delete['bot_id']]
            return True
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ Bot删除失败: {error_msg}")
            return False

    def delete_user(self, user_name: str) -> bool:
        """删除用户账户。"""
        if user_name not in self.users:
            print(f"❌ 用户 {user_name} 未找到")
            return False

        user_bots = [b for b in self.bots.values() if b["owner"] == user_name]
        if not user_bots:
            print(f"❌ 没有找到用户 {user_name} 的Bot")
            return False

        user_bot_token = user_bots[0]["token"]

        print(f"\n⚠️  删除用户账户: {user_name}")
        print(f"   警告：此操作不可逆，将删除该用户和所有Bot")

        response = self._make_request(
            "DELETE",
            "/api/v1/users/delete",
            token=user_bot_token
        )

        if response.status_code == 200:
            print(f"✅ 用户账户删除成功！")
            # Remove from local storage
            del self.users[user_name]
            for bot_id in list(self.bots.keys()):
                if self.bots[bot_id]["owner"] == user_name:
                    del self.bots[bot_id]
            return True
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ 用户账户删除失败: {error_msg}")
            return False


def main():
    """运行测试场景。"""
    print("=" * 60)
    print("🚀 聊天平台测试 - 新架构")
    print("=" * 60)

    # 初始化测试器
    tester = ChatPlatformTester()

    # 检查服务器健康状态
    print("\n🔍 检查服务器状态...")
    if not tester.health_check():
        print("❌ 服务器没有响应。请使用 'cargo run --release' 启动它")
        sys.exit(1)
    print("✅ 服务器运行中！")

    # 测试场景1: 注册两个用户
    print("\n" + "=" * 60)
    print("场景1: 使用身份证号进行用户注册认证")
    print("=" * 60)

    user_a_success = tester.register_user(
        name="Alice",
        id_number="110101199003071234",
        phone="13800138001"
    )

    user_b_success = tester.register_user(
        name="Bob",
        id_number="110101199003071235",
        phone="13800138002"
    )

    if not (user_a_success and user_b_success):
        print("❌ 用户注册失败")
        sys.exit(1)

    # 测试场景2: 列出每个用户的Bot
    print("\n" + "=" * 60)
    print("场景2: 列出用户的Bot")
    print("=" * 60)

    tester.list_user_bots("Alice")
    tester.list_user_bots("Bob")

    # 测试场景3: 创建额外的Bot
    print("\n" + "=" * 60)
    print("场景3: 创建额外的Bot")
    print("=" * 60)

    alice_customer_service = tester.create_bot(
        "Alice",
        "Alice客服Bot",
        "处理Alice的客户咨询"
    )

    bob_support_bot = tester.create_bot(
        "Bob",
        "Bob支持Bot",
        "Bob的技术支持机器人"
    )

    # 测试场景4: 用户之间的消息交换
    print("\n" + "=" * 60)
    print("场景4: 用户之间的消息交换")
    print("=" * 60)

    print("\n第1阶段: 使用默认Bot")
    tester.send_message("Alice", "Bob", "你好Bob！这是来自Alice的默认Bot的消息。")
    tester.send_message("Bob", "Alice", "你好Alice！这是来自Bob的默认Bot的消息。")

    if alice_customer_service and bob_support_bot:
        print("\n第2阶段: 使用自定义Bot")
        tester.send_message("Alice", "Bob", "这是来自Alice的客服Bot的消息", 1, 0)
        tester.send_message("Bob", "Alice", "这是来自Bob的支持Bot的消息", 1, 0)

    # 测试场景5: 删除Bot
    print("\n" + "=" * 60)
    print("场景5: 删除Bot")
    print("=" * 60)

    if alice_customer_service:
        tester.delete_bot("Alice", 1)

    # 打印总结
    print("\n" + "=" * 60)
    print("📊 测试总结")
    print("=" * 60)

    print("\n✅ 已注册的用户:")
    for name, user_info in tester.users.items():
        print(f"  • {name}")
        print(f"    - 用户ID: {user_info['user_id']}")
        print(f"    - 身份证号: {user_info['id_number']}")

    print("\n✅ Bot列表:")
    for bot_id, bot_info in tester.bots.items():
        print(f"  • {bot_info['name']}")
        print(f"    - Bot ID: {bot_id}")
        print(f"    - 所有者: {bot_info['owner']}")
        print(f"    - 状态: {bot_info['status']}")

    print("\n" + "=" * 60)
    print("✨ 所有测试完成成功！")
    print("=" * 60)


if __name__ == "__main__":
    main()
