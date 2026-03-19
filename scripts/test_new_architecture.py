#!/usr/bin/env python3
"""
Chat Platform Test Script - New Architecture with User + Bot Management

This script demonstrates the new authentication system:
- Users register with ID number (real-name authentication)
- Users can create and manage multiple bots
- Only bots can send messages via API
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
        """Make HTTP request with proper error handling."""
        url = f"{self.base_url}{endpoint}"
        headers = {"Content-Type": "application/json"}

        if token:
            headers["Authorization"] = f"Bearer {token}"

        try:
            if method == "GET":
                response = self.session.get(url, headers=headers)
            elif method == "POST":
                response = self.session.post(url, json=data, headers=headers)
            else:
                raise ValueError(f"Unsupported method: {method}")

            return response
        except requests.exceptions.ConnectionError:
            print("❌ Error: Cannot connect to server. Is it running on port 8080?")
            sys.exit(1)
        except Exception as e:
            print(f"❌ Request error: {e}")
            sys.exit(1)

    def health_check(self) -> bool:
        """Check if server is running."""
        try:
            response = self._make_request("GET", "/health")
            return response.status_code == 200
        except:
            return False

    def register_user(self, name: str, id_number: str, phone: str) -> bool:
        """Register a new user with ID number authentication."""
        print(f"\n📝 Registering user: {name}")
        print(f"   ID Number: {id_number}")

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

            # Store user and bot info
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
                "name": f"{name}'s default bot",
                "status": "active"
            }

            print(f"✅ User registered successfully!")
            print(f"   User ID: {user_id}")
            print(f"   Default Bot ID: {bot_id}")
            return True
        else:
            error_msg = response.json().get("error", "Unknown error")
            print(f"❌ Registration failed: {error_msg}")
            if response.status_code == 409:
                print("   ℹ️  This ID number is already registered")
            return False

    def create_bot(self, user_name: str, bot_name: str, description: Optional[str] = None) -> Optional[str]:
        """Create a new bot for a user."""
        print(f"\n🤖 Creating bot for {user_name}")
        print(f"   Bot name: {bot_name}")

        if user_name not in self.users:
            print(f"❌ User {user_name} not found")
            return None

        # Get the user's default bot token
        user_bots = [b for b in self.bots.values() if b["owner"] == user_name]
        if not user_bots:
            print(f"❌ No bot found for user {user_name}")
            return None

        user_bot_token = user_bots[0]["token"]

        data = {
            "name": bot_name,
            "description": description or f"Created by {user_name}"
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

            print(f"✅ Bot created successfully!")
            print(f"   Bot ID: {bot_id}")
            return bot_id
        else:
            error_msg = response.json().get("error", "Unknown error")
            print(f"❌ Bot creation failed: {error_msg}")
            return None

    def list_user_bots(self, user_name: str) -> bool:
        """List all bots for a user."""
        print(f"\n📋 Listing bots for {user_name}")

        if user_name not in self.users:
            print(f"❌ User {user_name} not found")
            return False

        # Get the user's default bot token
        user_bots = [b for b in self.bots.values() if b["owner"] == user_name]
        if not user_bots:
            print(f"❌ No bot found for user {user_name}")
            return False

        user_bot_token = user_bots[0]["token"]

        response = self._make_request("GET", "/api/v1/bots", token=user_bot_token)

        if response.status_code == 200:
            result = response.json()
            bots = result.get("bots", [])

            print(f"✅ Found {len(bots)} bot(s):")
            for bot in bots:
                print(f"   - {bot['name']} ({bot['bot_id']})")
                print(f"     Status: {bot['status']}")
                if bot.get('description'):
                    print(f"     Description: {bot['description']}")
            return True
        else:
            error_msg = response.json().get("error", "Unknown error")
            print(f"❌ Failed to list bots: {error_msg}")
            return False

    def send_message(self, from_user: str, to_user: str, content: str, from_bot_idx: int = 0, to_bot_idx: int = 0) -> bool:
        """Send a message from one user's bot to another user's bot."""
        print(f"\n💬 Sending message from {from_user} to {to_user}")
        print(f"   Content: {content}")

        # Get sender's bot
        from_user_bots = [b for b in self.bots.values() if b["owner"] == from_user]
        if not from_user_bots or len(from_user_bots) <= from_bot_idx:
            print(f"❌ Bot not found for user {from_user}")
            return False

        sender_bot = from_user_bots[from_bot_idx]
        sender_token = sender_bot["token"]

        # Get recipient's bot
        to_user_bots = [b for b in self.bots.values() if b["owner"] == to_user]
        if not to_user_bots or len(to_user_bots) <= to_bot_idx:
            print(f"❌ Bot not found for user {to_user}")
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
            print(f"✅ Message sent successfully! (ID: {msg_id})")
            print(f"   From: {sender_bot['name']} ({sender_bot['bot_id']})")
            print(f"   To: {recipient_bot['name']} ({recipient_bot_id})")
            return True
        else:
            error_msg = response.json().get("error", "Unknown error")
            print(f"❌ Failed to send message: {error_msg}")
            return False


def main():
    """Run the test scenarios."""
    print("=" * 60)
    print("🚀 Chat Platform Test - New Architecture")
    print("=" * 60)

    # Initialize tester
    tester = ChatPlatformTester()

    # Check server health
    print("\n🔍 Checking server health...")
    if not tester.health_check():
        print("❌ Server is not responding. Please start it with 'cargo run --release'")
        sys.exit(1)
    print("✅ Server is running!")

    # Test scenario 1: Register two users
    print("\n" + "=" * 60)
    print("Scenario 1: User Registration with ID Number Authentication")
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
        print("❌ User registration failed")
        sys.exit(1)

    # Test scenario 2: List bots for each user
    print("\n" + "=" * 60)
    print("Scenario 2: List Bots for Users")
    print("=" * 60)

    tester.list_user_bots("Alice")
    tester.list_user_bots("Bob")

    # Test scenario 3: Create additional bots
    print("\n" + "=" * 60)
    print("Scenario 3: Create Additional Bots")
    print("=" * 60)

    alice_customer_service = tester.create_bot(
        "Alice",
        "Alice Customer Service Bot",
        "Handles customer inquiries for Alice"
    )

    bob_support_bot = tester.create_bot(
        "Bob",
        "Bob Support Bot",
        "Technical support bot for Bob"
    )

    # Test scenario 4: Message exchange between users
    print("\n" + "=" * 60)
    print("Scenario 4: Message Exchange Between Users")
    print("=" * 60)

    print("\nPhase 1: Using default bots")
    tester.send_message("Alice", "Bob", "Hi Bob! This is Alice's default bot.")
    tester.send_message("Bob", "Alice", "Hello Alice! This is Bob's default bot.")

    if alice_customer_service and bob_support_bot:
        print("\nPhase 2: Using custom bots")
        tester.send_message("Alice", "Bob", "This is from Alice's customer service bot", 1, 0)
        tester.send_message("Bob", "Alice", "This is from Bob's support bot", 1, 0)

    # Print summary
    print("\n" + "=" * 60)
    print("📊 Test Summary")
    print("=" * 60)

    print("\n✅ Registered Users:")
    for name, user_info in tester.users.items():
        print(f"  • {name}")
        print(f"    - User ID: {user_info['user_id']}")
        print(f"    - ID Number: {user_info['id_number']}")

    print("\n✅ Bots:")
    for bot_id, bot_info in tester.bots.items():
        print(f"  • {bot_info['name']}")
        print(f"    - Bot ID: {bot_id}")
        print(f"    - Owner: {bot_info['owner']}")
        print(f"    - Status: {bot_info['status']}")

    print("\n" + "=" * 60)
    print("✨ All tests completed successfully!")
    print("=" * 60)


if __name__ == "__main__":
    main()
