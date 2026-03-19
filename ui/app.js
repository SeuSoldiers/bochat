// API 基础 URL
const API_BASE_URL = 'http://localhost:8080/api/v1';

// 应用状态
const appState = {
    currentUser: null,
    currentBot: null,
    currentGroup: null,
    groups: [],
    messages: {},
    ws: null,
    token: null,
};

// 初始化应用
document.addEventListener('DOMContentLoaded', () => {
    setupEventListeners();
    checkExistingSession();
});

// 设置事件监听
function setupEventListeners() {
    // 认证标签页
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const tab = e.target.dataset.tab;
            switchTab(tab);
        });
    });

    // 登录表单
    document.getElementById('loginForm').addEventListener('submit', handleLogin);

    // 注册表单
    document.getElementById('registerForm').addEventListener('submit', handleRegister);

    // 退出登录
    document.getElementById('logoutBtn').addEventListener('click', handleLogout);

    // 创建群聊按钮
    document.getElementById('createGroupBtn').addEventListener('click', openCreateGroupModal);

    // 创建群聊表单
    document.getElementById('createGroupForm').addEventListener('submit', handleCreateGroup);

    // 加入群聊按钮
    document.getElementById('joinGroupModalBtn').addEventListener('click', openJoinGroupModal);

    // 发送消息表单
    document.getElementById('sendMessageForm').addEventListener('submit', handleSendMessage);

    // 查看成员按钮
    document.getElementById('viewMembersBtn').addEventListener('click', openMembersModal);

    // 模态框关闭按钮
    document.querySelectorAll('.close').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.target.closest('.modal').classList.remove('show');
        });
    });

    // 模态框外点击关闭
    document.querySelectorAll('.modal').forEach(modal => {
        modal.addEventListener('click', (e) => {
            if (e.target === modal) {
                modal.classList.remove('show');
            }
        });
    });
}

// 切换认证标签页
function switchTab(tab) {
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });

    event.target.classList.add('active');
    document.getElementById(tab + 'Tab').classList.add('active');
}

// 检查已存在的会话
function checkExistingSession() {
    const savedToken = localStorage.getItem('token');
    const savedUser = localStorage.getItem('user');
    const savedBot = localStorage.getItem('bot');

    if (savedToken && savedUser && savedBot) {
        appState.token = savedToken;
        appState.currentUser = JSON.parse(savedUser);
        appState.currentBot = JSON.parse(savedBot);
        showChatPage();
        loadGroups();
    }
}

// 处理登录
async function handleLogin(e) {
    e.preventDefault();

    const phone = document.getElementById('loginPhone').value;
    const idNumber = document.getElementById('loginIdNumber').value;
    const errorEl = document.getElementById('loginError');

    // 简单的客户端验证
    if (idNumber.length !== 18) {
        showError(errorEl, '身份证号必须是18位');
        return;
    }

    try {
        // 由于后端没有单独的登录端点，我们使用注册端点，让用户登录
        // 但实际上后端应该有登录功能，这里假设我们有某种登录方式
        // 为了演示，我们假设可以通过已注册用户的信息重新获取token

        // 显示错误提示 - 后端应该实现登录接口
        showError(errorEl, '请先注册账户');

    } catch (error) {
        showError(errorEl, '登录失败: ' + error.message);
    }
}

// 处理注册
async function handleRegister(e) {
    e.preventDefault();

    const name = document.getElementById('registerName').value;
    const idNumber = document.getElementById('registerIdNumber').value;
    const phone = document.getElementById('registerPhone').value;
    const errorEl = document.getElementById('registerError');

    // 验证身份证号
    if (idNumber.length !== 18) {
        showError(errorEl, '身份证号必须是18位');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/auth/register`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                name: name,
                id_number: idNumber,
                phone: phone,
            }),
        });

        if (!response.ok) {
            const error = await response.json();
            showError(errorEl, error.error || '注册失败');
            return;
        }

        const data = await response.json();

        // 保存用户信息和 token
        appState.token = data.token;
        appState.currentUser = {
            user_id: data.user_id,
            name: name,
            id_number: idNumber,
            phone: phone,
        };
        appState.currentBot = {
            bot_id: data.bot_id,
            owner_id: data.user_id,
            name: '默认Bot',
            status: 'active',
            secret: data.secret,
        };

        // 本地存储
        localStorage.setItem('token', appState.token);
        localStorage.setItem('user', JSON.stringify(appState.currentUser));
        localStorage.setItem('bot', JSON.stringify(appState.currentBot));

        // 清空表单
        document.getElementById('registerForm').reset();
        clearError(errorEl);

        // 显示聊天页面
        showChatPage();
        loadGroups();

    } catch (error) {
        showError(errorEl, '网络错误: ' + error.message);
    }
}

// 处理退出登录
function handleLogout() {
    if (confirm('确定要退出登录吗？')) {
        localStorage.removeItem('token');
        localStorage.removeItem('user');
        localStorage.removeItem('bot');

        appState.token = null;
        appState.currentUser = null;
        appState.currentBot = null;
        appState.currentGroup = null;
        appState.groups = [];
        appState.messages = {};

        // 关闭 WebSocket 连接
        if (appState.ws) {
            appState.ws.close();
            appState.ws = null;
        }

        // 返回认证页面
        showAuthPage();
    }
}

// 显示认证页面
function showAuthPage() {
    document.getElementById('authPage').classList.add('active');
    document.getElementById('chatPage').classList.remove('active');
}

// 显示聊天页面
function showChatPage() {
    document.getElementById('authPage').classList.remove('active');
    document.getElementById('chatPage').classList.add('active');
    document.getElementById('userName').textContent = appState.currentUser.name;
}

// 加载用户的群聊
async function loadGroups() {
    try {
        const response = await fetch(`${API_BASE_URL}/groups`, {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${appState.token}`,
            },
        });

        if (!response.ok) {
            console.error('加载群聊失败:', response.status);
            return;
        }

        const data = await response.json();
        appState.groups = data.groups || [];

        // 加载所有可用的群聊（用于加入）
        await loadAllAvailableGroups();

        // 刷新群聊列表
        renderGroupsList();

    } catch (error) {
        console.error('加载群聊错误:', error);
    }
}

// 加载所有可用的群聊
async function loadAllAvailableGroups() {
    // 这是一个辅助函数，实际上后端可能需要提供一个获取所有群聊的端点
    // 为了演示，我们暂时使用已有的群聊列表
}

// 渲染群聊列表
function renderGroupsList() {
    const groupsList = document.getElementById('groupsList');
    groupsList.innerHTML = '';

    if (appState.groups.length === 0) {
        groupsList.innerHTML = '<div class="groups-placeholder">还没有加入任何群聊</div>';
        return;
    }

    appState.groups.forEach(group => {
        const groupEl = document.createElement('div');
        groupEl.className = 'group-item';
        if (appState.currentGroup && appState.currentGroup.group_id === group.group_id) {
            groupEl.classList.add('active');
        }

        groupEl.innerHTML = `
            <div class="group-item-name">${escapeHtml(group.name)}</div>
            <div class="group-item-desc">${escapeHtml(group.description || '暂无描述')}</div>
        `;

        groupEl.addEventListener('click', () => selectGroup(group));
        groupsList.appendChild(groupEl);
    });
}

// 选择群聊
async function selectGroup(group) {
    appState.currentGroup = group;
    renderGroupsList();

    // 更新聊天头部
    document.getElementById('currentGroupName').textContent = escapeHtml(group.name);
    document.getElementById('currentGroupDesc').textContent = escapeHtml(group.description || '暂无描述');

    // 清空消息区域
    const messagesArea = document.getElementById('messagesArea');
    messagesArea.innerHTML = '<div class="loading"></div>';

    // 加载消息历史
    await loadGroupMessages(group.group_id);

    // 初始化 WebSocket 连接（用于接收实时消息）
    connectWebSocket();
}

// 加载群聊消息历史
async function loadGroupMessages(groupId) {
    try {
        // 注意：后端可能需要提供一个获取群聊消息的端点
        // 为了演示，我们假设消息通过 WebSocket 接收
        const messagesArea = document.getElementById('messagesArea');
        messagesArea.innerHTML = ''; // 清空加载提示

        // 如果之前加载过这个群聊的消息，使用缓存
        if (appState.messages[groupId]) {
            renderMessages(appState.messages[groupId]);
        } else {
            appState.messages[groupId] = [];
            messagesArea.innerHTML = '<div class="messages-placeholder">消息列表为空</div>';
        }

    } catch (error) {
        console.error('加载消息错误:', error);
        document.getElementById('messagesArea').innerHTML = '<div class="error-message">加载消息失败</div>';
    }
}

// 渲染消息
function renderMessages(messages) {
    const messagesArea = document.getElementById('messagesArea');
    messagesArea.innerHTML = '';

    if (messages.length === 0) {
        messagesArea.innerHTML = '<div class="messages-placeholder">暂无消息</div>';
        return;
    }

    messages.forEach(msg => {
        const messageEl = document.createElement('div');
        const isOwn = msg.sender_id === appState.currentBot.bot_id;
        messageEl.className = 'message' + (isOwn ? ' own' : '');

        messageEl.innerHTML = `
            <div>
                ${!isOwn ? `<div class="message-sender">${escapeHtml(msg.sender_name || msg.sender_id)}</div>` : ''}
                <div class="message-bubble">${escapeHtml(msg.content)}</div>
            </div>
        `;

        messagesArea.appendChild(messageEl);
    });

    // 滚动到底部
    messagesArea.scrollTop = messagesArea.scrollHeight;
}

// 处理发送消息
async function handleSendMessage(e) {
    e.preventDefault();

    if (!appState.currentGroup) {
        alert('请先选择一个群聊');
        return;
    }

    const messageInput = document.getElementById('messageInput');
    const content = messageInput.value.trim();

    if (!content) {
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/message/send`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${appState.token}`,
            },
            body: JSON.stringify({
                group_id: appState.currentGroup.group_id,
                content: content,
                msg_type: 'text',
            }),
        });

        if (!response.ok) {
            const error = await response.json();
            alert('发送失败: ' + (error.error || '未知错误'));
            return;
        }

        // 清空输入框
        messageInput.value = '';

        // 本地显示消息
        const msg = {
            group_id: appState.currentGroup.group_id,
            sender_id: appState.currentBot.bot_id,
            sender_name: appState.currentUser.name,
            content: content,
            created_at: new Date().toISOString(),
        };

        if (!appState.messages[appState.currentGroup.group_id]) {
            appState.messages[appState.currentGroup.group_id] = [];
        }
        appState.messages[appState.currentGroup.group_id].push(msg);

        renderMessages(appState.messages[appState.currentGroup.group_id]);

    } catch (error) {
        alert('网络错误: ' + error.message);
    }
}

// WebSocket 连接
function connectWebSocket() {
    if (appState.ws) {
        appState.ws.close();
    }

    // 使用 WebSocket 连接接收实时消息
    const wsUrl = `ws://localhost:8080/ws?token=${appState.token}`;

    try {
        appState.ws = new WebSocket(wsUrl);

        appState.ws.onopen = () => {
            console.log('WebSocket 已连接');
        };

        appState.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);

                if (data.type === 'message') {
                    // 接收新消息
                    const groupId = data.group_id;

                    if (!appState.messages[groupId]) {
                        appState.messages[groupId] = [];
                    }

                    appState.messages[groupId].push({
                        group_id: groupId,
                        sender_id: data.sender_id,
                        sender_name: data.sender_name,
                        content: data.content,
                        created_at: data.created_at,
                    });

                    // 如果当前选中的就是这个群聊，刷新消息显示
                    if (appState.currentGroup && appState.currentGroup.group_id === groupId) {
                        renderMessages(appState.messages[groupId]);
                    }
                }

            } catch (error) {
                console.error('处理 WebSocket 消息错误:', error);
            }
        };

        appState.ws.onerror = (error) => {
            console.error('WebSocket 错误:', error);
        };

        appState.ws.onclose = () => {
            console.log('WebSocket 已断开');
        };

    } catch (error) {
        console.error('创建 WebSocket 连接错误:', error);
    }
}

// 打开创建群聊模态框
function openCreateGroupModal() {
    document.getElementById('createGroupModal').classList.add('show');
}

// 处理创建群聊
async function handleCreateGroup(e) {
    e.preventDefault();

    const groupName = document.getElementById('groupName').value;
    const groupDesc = document.getElementById('groupDescription').value;
    const errorEl = document.getElementById('createGroupError');

    if (!groupName.trim()) {
        showError(errorEl, '群名称不能为空');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/groups`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${appState.token}`,
            },
            body: JSON.stringify({
                name: groupName,
                description: groupDesc || null,
            }),
        });

        if (!response.ok) {
            const error = await response.json();
            showError(errorEl, error.error || '创建失败');
            return;
        }

        const data = await response.json();

        // 添加到群聊列表
        const newGroup = {
            group_id: data.group_id,
            creator_id: appState.currentUser.user_id,
            name: groupName,
            description: groupDesc || null,
            status: 'active',
        };

        appState.groups.push(newGroup);
        renderGroupsList();

        // 清空表单
        document.getElementById('createGroupForm').reset();
        clearError(errorEl);

        // 关闭模态框
        document.getElementById('createGroupModal').classList.remove('show');

        // 自动选择新创建的群聊
        selectGroup(newGroup);

    } catch (error) {
        showError(errorEl, '网络错误: ' + error.message);
    }
}

// 打开加入群聊模态框
async function openJoinGroupModal() {
    const modal = document.getElementById('joinGroupModal');
    const container = document.getElementById('availableGroups');

    // 显示加载中
    container.innerHTML = '<div class="loading"></div>';
    modal.classList.add('show');

    try {
        // 获取所有群聊
        const response = await fetch(`${API_BASE_URL}/groups`, {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${appState.token}`,
            },
        });

        if (!response.ok) {
            container.innerHTML = '<div class="error-message">加载群聊列表失败</div>';
            return;
        }

        const data = await response.json();
        const allGroups = data.groups || [];

        // 获取当前用户已加入的群聊
        const joinedGroupIds = new Set(appState.groups.map(g => g.group_id));

        // 过滤出未加入的群聊
        const availableGroups = allGroups.filter(g => !joinedGroupIds.has(g.group_id));

        if (availableGroups.length === 0) {
            container.innerHTML = '<div style="text-align: center; color: #999; padding: 20px;">没有可加入的群聊</div>';
            return;
        }

        // 显示可加入的群聊
        container.innerHTML = availableGroups.map(group => `
            <div class="group-card">
                <div>
                    <h4>${escapeHtml(group.name)}</h4>
                    <p>${escapeHtml(group.description || '暂无描述')}</p>
                </div>
                <button class="btn-join" data-group-id="${group.group_id}">加入</button>
            </div>
        `).join('');

        // 为加入按钮添加事件监听
        container.querySelectorAll('.btn-join').forEach(btn => {
            btn.addEventListener('click', () => {
                const groupId = btn.dataset.groupId;
                const group = allGroups.find(g => g.group_id === groupId);
                joinGroup(group);
            });
        });

    } catch (error) {
        container.innerHTML = '<div class="error-message">加载失败: ' + error.message + '</div>';
    }
}

// 加入群聊
async function joinGroup(group) {
    try {
        const response = await fetch(`${API_BASE_URL}/groups/${group.group_id}/join`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${appState.token}`,
            },
        });

        if (!response.ok) {
            const error = await response.json();
            alert('加入失败: ' + (error.error || '未知错误'));
            return;
        }

        // 添加到群聊列表
        appState.groups.push(group);
        renderGroupsList();

        // 关闭模态框
        document.getElementById('joinGroupModal').classList.remove('show');

        // 自动选择刚加入的群聊
        selectGroup(group);

    } catch (error) {
        alert('网络错误: ' + error.message);
    }
}

// 打开成员列表模态框
async function openMembersModal() {
    if (!appState.currentGroup) {
        return;
    }

    const modal = document.getElementById('membersModal');
    const container = document.getElementById('membersList');

    // 显示加载中
    container.innerHTML = '<div class="loading"></div>';
    modal.classList.add('show');

    try {
        const response = await fetch(`${API_BASE_URL}/groups/${appState.currentGroup.group_id}/members`, {
            method: 'GET',
        });

        if (!response.ok) {
            container.innerHTML = '<div class="error-message">加载成员失表失败</div>';
            return;
        }

        const data = await response.json();
        const members = data.members || [];

        if (members.length === 0) {
            container.innerHTML = '<div style="text-align: center; color: #999; padding: 20px;">还没有成员</div>';
            return;
        }

        // 显示成员列表
        container.innerHTML = members.map(member => `
            <div class="member-item">
                <div class="member-info">
                    <div class="member-name">${escapeHtml(member.member_id)}</div>
                    <div class="member-type">${escapeHtml(member.member_type)} • 加入于 ${formatDate(member.joined_at)}</div>
                </div>
            </div>
        `).join('');

    } catch (error) {
        container.innerHTML = '<div class="error-message">加载失败: ' + error.message + '</div>';
    }
}

// 工具函数
function showError(el, message) {
    el.textContent = message;
    el.classList.add('show');
}

function clearError(el) {
    el.textContent = '';
    el.classList.remove('show');
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString('zh-CN', {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
    });
}
