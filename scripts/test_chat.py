#!/usr/bin/env python3
"""
聊天平台测试脚本 - 群聊模式

本脚本演示新的群聊系统：
- 用户使用身份证号注册（实名认证）
- 用户可以创建群聊
- Bot可以加入群聊
- Bot在群聊中发送消息
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
        self.groups: Dict[str, Dict[str, Any]] = {}

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

    def create_group(self, user_name: str, group_name: str, description: Optional[str] = None) -> Optional[str]:
        """为用户创建新群聊。"""
        print(f"\n👥 为 {user_name} 创建群聊")
        print(f"   群名: {group_name}")

        if user_name not in self.users:
            print(f"❌ 用户 {user_name} 未找到")
            return None

        user_bots = [b for b in self.bots.values() if b["owner"] == user_name]
        if not user_bots:
            print(f"❌ 没有找到用户 {user_name} 的Bot")
            return None

        user_bot_token = user_bots[0]["token"]

        data = {
            "name": group_name,
            "description": description or f"由 {user_name} 创建的群聊"
        }

        response = self._make_request("POST", "/api/v1/groups", data, user_bot_token)

        if response.status_code == 201:
            result = response.json()
            group_id = result["group_id"]

            self.groups[group_id] = {
                "group_id": group_id,
                "creator": user_name,
                "name": group_name,
                "members": []
            }

            print(f"✅ 群聊创建成功！")
            print(f"   群ID: {group_id}")
            return group_id
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ 群聊创建失败: {error_msg}")
            return None

    def join_group(self, user_name: str, group_id: str, bot_idx: int = 0) -> bool:
        """Bot加入群聊。"""
        if user_name not in self.users:
            print(f"❌ 用户 {user_name} 未找到")
            return False

        user_bots = [b for b in self.bots.values() if b["owner"] == user_name]
        if not user_bots or len(user_bots) <= bot_idx:
            print(f"❌ 未找到用户 {user_name} 的Bot")
            return False

        bot = user_bots[bot_idx]
        bot_token = bot["token"]

        print(f"\n➕ {bot['name']} 加入群聊 {group_id}")

        response = self._make_request("POST", f"/api/v1/groups/{group_id}/join", token=bot_token)

        if response.status_code == 200:
            print(f"✅ Bot加入群聊成功！")
            if group_id in self.groups:
                self.groups[group_id]["members"].append(bot["bot_id"])
            return True
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ Bot加入群聊失败: {error_msg}")
            return False

    def send_message(self, user_name: str, group_id: str, content: str, bot_idx: int = 0) -> bool:
        """在群聊中发送消息。"""
        if user_name not in self.users:
            print(f"❌ 用户 {user_name} 未找到")
            return False

        user_bots = [b for b in self.bots.values() if b["owner"] == user_name]
        if not user_bots or len(user_bots) <= bot_idx:
            print(f"❌ 未找到用户 {user_name} 的Bot")
            return False

        bot = user_bots[bot_idx]
        bot_token = bot["token"]

        print(f"\n💬 {bot['name']} 在群聊中发送消息")
        print(f"   内容: {content}")

        data = {
            "group_id": group_id,
            "content": {"text": content},
            "msg_type": "text"
        }

        response = self._make_request("POST", "/api/v1/message/send", data, bot_token)

        if response.status_code == 201:
            result = response.json()
            msg_id = result["msg_id"]
            print(f"✅ 消息发送成功！(ID: {msg_id})")
            return True
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ 消息发送失败: {error_msg}")
            return False

    def list_group_members(self, group_id: str) -> bool:
        """列出群成员。"""
        print(f"\n👥 列出群 {group_id} 的成员")

        response = self._make_request("GET", f"/api/v1/groups/{group_id}/members")

        if response.status_code == 200:
            result = response.json()
            members = result.get("members", [])
            print(f"✅ 找到 {len(members)} 个成员：")
            for member in members:
                print(f"   - {member['member_id']} (加入时间: {member['joined_at']})")
            return True
        else:
            error_msg = response.json().get("error", "未知错误")
            print(f"❌ 获取群成员失败: {error_msg}")
            return False


def main():
    """运行测试场景。"""
    print("=" * 60)
    print("🚀 聊天平台测试 - 群聊模式")
    print("=" * 60)

    tester = ChatPlatformTester()

    # 检查服务器状态
    print("\n🔍 检查服务器状态...")
    if not tester.health_check():
        print("❌ 服务器没有响应。请使用 'cargo run --release' 启动它")
        sys.exit(1)
    print("✅ 服务器运行中！")

    # 场景1: 注册用户
    print("\n" + "=" * 60)
    print("场景1: 用户注册")
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

    # 场景2: 创建群聊
    print("\n" + "=" * 60)
    print("场景2: 创建群聊")
    print("=" * 60)

    group1 = tester.create_group("Alice", "技术讨论组", "讨论技术问题的群聊")
    group2 = tester.create_group("Bob", "产品反馈组", "收集产品反馈的群聊")

    if not (group1 and group2):
        print("❌ 群聊创建失败")
        sys.exit(1)

    # 场景3: Bot加入群聊
    print("\n" + "=" * 60)
    print("场景3: Bot加入群聊")
    print("=" * 60)

    # Alice的Bot加入两个群
    tester.join_group("Alice", group1)
    tester.join_group("Alice", group2)

    # Bob的Bot加入两个群
    tester.join_group("Bob", group1)
    tester.join_group("Bob", group2)

    # 场景4: 在群聊中发送消息
    print("\n" + "=" * 60)
    print("场景4: 在群聊中发送消息")
    print("=" * 60)

    print(f"\n--- 在 {group1} 中 ---")
    tester.send_message("Alice", group1, "大家好！这是技术讨论组")
    tester.send_message("Bob", group1, "很高兴加入，我想讨论一些技术问题")
    tester.send_message("Alice", group1, "太好了，我们一起讨论吧！")

    print(f"\n--- 在 {group2} 中 ---")
    tester.send_message("Bob", group2, "欢迎来到产品反馈组！")
    tester.send_message("Alice", group2, "感谢邀请，我有一些反馈要提供")
    tester.send_message("Bob", group2, "请继续，我们很想听听你的意见")

    # 场景5: 查看群成员
    print("\n" + "=" * 60)
    print("场景5: 查看群成员")
    print("=" * 60)

    tester.list_group_members(group1)
    tester.list_group_members(group2)

    # 打印总结
    print("\n" + "=" * 60)
    print("📊 测试总结")
    print("=" * 60)

    print("\n✅ 已注册的用户:")
    for name, user_info in tester.users.items():
        print(f"  • {name}")
        print(f"    - 用户ID: {user_info['user_id']}")
        print(f"    - 身份证号: {user_info['id_number']}")

    print("\n✅ 已创建的群聊:")
    for group_id, group_info in tester.groups.items():
        print(f"  • {group_info['name']}")
        print(f"    - 群ID: {group_id}")
        print(f"    - 创建者: {group_info['creator']}")
        print(f"    - 成员数: {len(group_info['members'])}")

    print("\n✅ 可用的Bot:")
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
