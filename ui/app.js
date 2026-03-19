// API 基础 URL
const API_BASE_URL = 'http://localhost:8080/api/v1';

// 应用状态
const appState = {
    currentUser: null,
    token: null,
    defaultBotToken: null,
    bots: [],           // 所有Bot及其token
    groups: [],         // 所有群聊
    selectedBot: null,  // 聊天页选中的Bot
    selectedGroup: null,// 聊天页选中的群聊
    messages: {},       // group_id -> messages[]
    ws: null,
    currentPage: 'bots', // 当前页面
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
            switchAuthTab(tab);
        });
    });

    // 登录表单
    document.getElementById('loginForm').addEventListener('submit', handleLogin);
    document.getElementById('registerForm').addEventListener('submit', handleRegister);

    // 顶部导航栏
    document.querySelectorAll('.nav-tab-btn').forEach(btn => {
        btn.addEventListener('click', (e) => {
            const page = e.target.dataset.page;
            switchPage(page);
        });
    });

    // 退出登录
    document.getElementById('logoutBtn').addEventListener('click', handleLogout);

    // Bot管理
    document.getElementById('createBotBtn').addEventListener('click', openCreateBotModal);
    document.getElementById('createBotForm').addEventListener('submit', handleCreateBot);

    // 群聊管理
    document.getElementById('createGroupBtn').addEventListener('click', openCreateGroupModal);
    document.getElementById('createGroupForm').addEventListener('submit', handleCreateGroup);
    document.getElementById('joinGroupBtn').addEventListener('click', openJoinGroupModal);
    document.getElementById('joinGroupForm').addEventListener('submit', handleJoinGroup);

    // 聊天
    document.getElementById('botSelector').addEventListener('change', onBotSelected);
    document.getElementById('groupSelector').addEventListener('change', onGroupSelected);
    document.getElementById('sendMessageForm').addEventListener('submit', handleSendMessage);

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
function switchAuthTab(tab) {
    document.querySelectorAll('.tab-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });

    event.target.classList.add('active');
    document.getElementById(tab + 'Tab').classList.add('active');
}

// 切换应用页面
function switchPage(page) {
    appState.currentPage = page;

    // 更新导航栏按钮状态
    document.querySelectorAll('.nav-tab-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    document.querySelector(`[data-page="${page}"]`).classList.add('active');

    // 更新页面内容显示
    document.querySelectorAll('.page-content').forEach(content => {
        content.classList.remove('active');
    });
    document.getElementById(page + 'Page').classList.add('active');

    // 加载相应数据
    if (page === 'bots') {
        loadBots();
    } else if (page === 'groups') {
        loadGroups();
    } else if (page === 'chat') {
        if (!appState.selectedBot || !appState.selectedGroup) {
            populateBotSelector();
        }
    }
}

// 检查已存在的会话
function checkExistingSession() {
    const savedToken = localStorage.getItem('token');
    const savedUser = localStorage.getItem('user');
    const savedDefaultBotToken = localStorage.getItem('defaultBotToken');

    if (savedToken && savedUser && savedDefaultBotToken) {
        appState.token = savedToken;
        appState.defaultBotToken = savedDefaultBotToken;
        appState.currentUser = JSON.parse(savedUser);
        showAppPage();
        loadBots();
    }
}

// ============ 认证相关 ============

async function handleLogin(e) {
    e.preventDefault();

    const phone = document.getElementById('loginPhone').value;
    const idNumber = document.getElementById('loginIdNumber').value;
    const errorEl = document.getElementById('loginError');

    if (idNumber.length !== 18) {
        showError(errorEl, '身份证号必须是18位');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ id_number: idNumber, phone }),
        });

        if (!response.ok) {
            const error = await response.json();
            showError(errorEl, error.error || '登录失败');
            return;
        }

        const data = await response.json();

        appState.token = data.token;
        appState.defaultBotToken = data.token;
        appState.currentUser = {
            user_id: data.user_id,
            name: data.name,
            id_number: idNumber,
            phone: phone,
        };

        localStorage.setItem('token', appState.token);
        localStorage.setItem('defaultBotToken', appState.defaultBotToken);
        localStorage.setItem('user', JSON.stringify(appState.currentUser));

        document.getElementById('loginForm').reset();
        clearError(errorEl);
        showAppPage();
        loadBots();

    } catch (error) {
        showError(errorEl, '网络错误: ' + error.message);
    }
}

async function handleRegister(e) {
    e.preventDefault();

    const name = document.getElementById('registerName').value;
    const idNumber = document.getElementById('registerIdNumber').value;
    const phone = document.getElementById('registerPhone').value;
    const errorEl = document.getElementById('registerError');

    if (idNumber.length !== 18) {
        showError(errorEl, '身份证号必须是18位');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/auth/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name, id_number: idNumber, phone }),
        });

        if (!response.ok) {
            const error = await response.json();
            showError(errorEl, error.error || '注册失败');
            return;
        }

        document.getElementById('registerForm').reset();
        clearError(errorEl);
        alert('✅ 注册成功！现在请登录');

        // 切换到登录标签页
        document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
        document.querySelector('[data-tab="login"]').classList.add('active');
        document.getElementById('loginTab').classList.add('active');

    } catch (error) {
        showError(errorEl, '网络错误: ' + error.message);
    }
}

function handleLogout() {
    if (confirm('确定要退出登录吗？')) {
        localStorage.removeItem('token');
        localStorage.removeItem('user');
        localStorage.removeItem('defaultBotToken');

        appState.token = null;
        appState.currentUser = null;
        appState.defaultBotToken = null;
        appState.bots = [];
        appState.groups = [];
        appState.messages = {};
        appState.selectedBot = null;
        appState.selectedGroup = null;

        if (appState.ws) {
            appState.ws.close();
            appState.ws = null;
        }

        showAuthPage();
    }
}

function showAuthPage() {
    document.getElementById('authPage').classList.add('active');
    document.getElementById('appPage').classList.remove('active');
}

function showAppPage() {
    document.getElementById('authPage').classList.remove('active');
    document.getElementById('appPage').classList.add('active');
    document.getElementById('userDisplay').textContent = `👤 ${appState.currentUser.name}`;
}

// ============ Bot管理 ============

async function loadBots() {
    try {
        const response = await fetch(`${API_BASE_URL}/bots`, {
            method: 'GET',
            headers: { 'Authorization': `Bearer ${appState.token}` },
        });

        if (!response.ok) {
            console.error('加载Bot失败');
            return;
        }

        const data = await response.json();
        appState.bots = data.bots || [];
        renderBotsList();

    } catch (error) {
        console.error('加载Bot错误:', error);
    }
}

function renderBotsList() {
    const botsList = document.getElementById('botsList');
    botsList.innerHTML = '';

    if (appState.bots.length === 0) {
        botsList.innerHTML = '<div style="grid-column: 1/-1; text-align: center; color: #999; padding: 40px;">还没有创建任何Bot</div>';
        return;
    }

    appState.bots.forEach(bot => {
        const card = document.createElement('div');
        card.className = 'card';
        card.innerHTML = `
            <div class="card-header">
                <h3>🤖 ${escapeHtml(bot.name)}</h3>
                <span class="badge status-active">活跃</span>
            </div>
            <div class="card-body">
                <p class="bot-id">ID: ${bot.bot_id}</p>
                <p class="bot-desc">${escapeHtml(bot.description || '无描述')}</p>
                <p class="bot-created">创建于: ${formatDate(bot.created_at)}</p>
            </div>
            <div class="card-footer">
                <button class="btn btn-small btn-danger" data-bot-id="${bot.bot_id}">删除</button>
            </div>
        `;
        card.querySelector('.btn-danger').addEventListener('click', () => deleteBot(bot.bot_id));
        botsList.appendChild(card);
    });
}

async function handleCreateBot(e) {
    e.preventDefault();

    const name = document.getElementById('botName').value;
    const description = document.getElementById('botDescription').value;
    const errorEl = document.getElementById('createBotError');

    if (!name.trim()) {
        showError(errorEl, 'Bot名称不能为空');
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/bots`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${appState.token}`,
            },
            body: JSON.stringify({ name, description: description || null }),
        });

        if (!response.ok) {
            const error = await response.json();
            showError(errorEl, error.error || '创建失败');
            return;
        }

        document.getElementById('createBotForm').reset();
        clearError(errorEl);
        document.getElementById('createBotModal').classList.remove('show');
        await loadBots();

    } catch (error) {
        showError(errorEl, '网络错误: ' + error.message);
    }
}

async function deleteBot(botId) {
    if (!confirm('确定要删除这个Bot吗？')) return;

    try {
        const response = await fetch(`${API_BASE_URL}/bots/${botId}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${appState.token}` },
        });

        if (!response.ok) {
            alert('删除失败');
            return;
        }

        await loadBots();

    } catch (error) {
        console.error('删除Bot错误:', error);
    }
}

function openCreateBotModal() {
    document.getElementById('createBotModal').classList.add('show');
}

// ============ 群聊管理 ============

async function loadGroups() {
    try {
        const response = await fetch(`${API_BASE_URL}/groups`, {
            method: 'GET',
            headers: { 'Authorization': `Bearer ${appState.token}` },
        });

        if (!response.ok) {
            console.error('加载群聊失败');
            return;
        }

        const data = await response.json();
        appState.groups = data.groups || [];
        renderGroupsList();

    } catch (error) {
        console.error('加载群聊错误:', error);
    }
}

function renderGroupsList() {
    const groupsList = document.getElementById('groupsList');
    groupsList.innerHTML = '';

    if (appState.groups.length === 0) {
        groupsList.innerHTML = '<div style="grid-column: 1/-1; text-align: center; color: #999; padding: 40px;">还没有任何群聊</div>';
        return;
    }

    appState.groups.forEach(group => {
        const card = document.createElement('div');
        card.className = 'card';
        card.innerHTML = `
            <div class="card-header">
                <h3>👥 ${escapeHtml(group.name)}</h3>
                ${group.group_code ? `<span class="badge">群号: ${escapeHtml(group.group_code)}</span>` : ''}
            </div>
            <div class="card-body">
                <p class="group-desc">${escapeHtml(group.description || '无描述')}</p>
                <p style="font-size: 12px; color: #999;">群ID: ${group.group_id}</p>
            </div>
            <div class="card-footer">
                <button class="btn btn-small" data-action="view-members" data-group-id="${group.group_id}">👥 成员</button>
                <button class="btn btn-small" data-action="assign-bot" data-group-id="${group.group_id}">➕ 分配Bot</button>
                <button class="btn btn-small btn-danger" data-action="delete-group" data-group-id="${group.group_id}">删除</button>
            </div>
        `;

        card.querySelector('[data-action="view-members"]').addEventListener('click', () => {
            openMembersModal(group.group_id);
        });
        card.querySelector('[data-action="assign-bot"]').addEventListener('click', () => {
            openAssignBotModal(group);
        });
        card.querySelector('[data-action="delete-group"]').addEventListener('click', () => {
            deleteGroup(group.group_id);
        });

        groupsList.appendChild(card);
    });
}

async function handleCreateGroup(e) {
    e.preventDefault();

    const name = document.getElementById('groupName').value;
    const groupCode = document.getElementById('groupCode').value.trim();
    const description = document.getElementById('groupDescription').value;
    const errorEl = document.getElementById('createGroupError');

    if (!name.trim()) {
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
                name,
                description: description || null,
                group_code: groupCode || null,
            }),
        });

        if (!response.ok) {
            const error = await response.json();
            showError(errorEl, error.error || '创建失败');
            return;
        }

        document.getElementById('createGroupForm').reset();
        clearError(errorEl);
        document.getElementById('createGroupModal').classList.remove('show');
        await loadGroups();

    } catch (error) {
        showError(errorEl, '网络错误: ' + error.message);
    }
}

async function deleteGroup(groupId) {
    if (!confirm('确定要删除这个群聊吗？')) return;

    try {
        const response = await fetch(`${API_BASE_URL}/groups/${groupId}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${appState.token}` },
        });

        if (!response.ok) {
            alert('删除失败');
            return;
        }

        await loadGroups();

    } catch (error) {
        console.error('删除群聊错误:', error);
    }
}

function openCreateGroupModal() {
    document.getElementById('createGroupModal').classList.add('show');
}

function openJoinGroupModal() {
    const modal = document.getElementById('joinGroupModal');
    const select = document.getElementById('joinBotSelect');
    select.innerHTML = '<option value="">-- 选择一个Bot --</option>';
    appState.bots.forEach(bot => {
        const option = document.createElement('option');
        option.value = bot.token;
        option.textContent = bot.name;
        select.appendChild(option);
    });
    document.getElementById('joinGroupCode').value = '';
    document.getElementById('joinGroupError').textContent = '';
    modal.classList.add('show');
}

async function handleJoinGroup(e) {
    e.preventDefault();

    const botToken = document.getElementById('joinBotSelect').value;
    const groupCode = document.getElementById('joinGroupCode').value.trim();
    const errorEl = document.getElementById('joinGroupError');

    if (!botToken || !groupCode) {
        errorEl.textContent = '请选择Bot和输入群号';
        return;
    }

    try {
        const response = await fetch(`${API_BASE_URL}/groups/join`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${botToken}`,
            },
            body: JSON.stringify({ group_code: groupCode }),
        });

        if (!response.ok) {
            const error = await response.json();
            errorEl.textContent = '加入失败: ' + (error.error || error.message || '未知错误');
            return;
        }

        alert('成功加入群聊!');
        document.getElementById('joinGroupModal').classList.remove('show');
        await loadGroups();

    } catch (error) {
        errorEl.textContent = '加入失败: ' + error.message;
    }
}

async function openMembersModal(groupId) {
    const modal = document.getElementById('membersModal');
    const container = document.getElementById('membersList');

    container.innerHTML = '<div class="loading"></div>';
    modal.classList.add('show');

    try {
        const response = await fetch(`${API_BASE_URL}/groups/${groupId}/members`);

        if (!response.ok) {
            container.innerHTML = '<div class="error-message">加载失败</div>';
            return;
        }

        const data = await response.json();
        const members = data.members || [];

        if (members.length === 0) {
            container.innerHTML = '<div style="text-align: center; color: #999; padding: 20px;">还没有成员</div>';
            return;
        }

        container.innerHTML = members.map(member => `
            <div class="member-item">
                <div>
                    <div class="member-name">${escapeHtml(member.member_id)}</div>
                    <div class="member-type">${member.member_type} • 加入于 ${formatDate(member.joined_at)}</div>
                </div>
            </div>
        `).join('');

    } catch (error) {
        container.innerHTML = '<div class="error-message">加载失败: ' + error.message + '</div>';
    }
}

function openAssignBotModal(group) {
    const modal = document.getElementById('assignBotModal');
    const select = document.getElementById('assignBotSelect');
    const info = document.getElementById('assignGroupInfo');

    select.innerHTML = '<option value="">-- 选择一个Bot --</option>';
    appState.bots.forEach(bot => {
        const option = document.createElement('option');
        option.value = bot.token;
        option.textContent = bot.name;
        select.appendChild(option);
    });

    info.textContent = `群聊: ${group.name} (${group.group_code ? '群号: ' + group.group_code : '群ID: ' + group.group_id})`;
    document.getElementById('assignBotError').textContent = '';

    // 保存groupId供表单提交使用
    modal.dataset.groupId = group.group_id;
    modal.classList.add('show');

    // 修改表单提交事件
    const form = document.getElementById('assignBotForm');
    form.onsubmit = async (e) => {
        e.preventDefault();
        const botToken = document.getElementById('assignBotSelect').value;
        const errorEl = document.getElementById('assignBotError');

        if (!botToken) {
            errorEl.textContent = '请选择Bot';
            return;
        }

        try {
            const response = await fetch(`${API_BASE_URL}/groups/join`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${botToken}`,
                },
                body: JSON.stringify({ group_id: group.group_id }),
            });

            if (!response.ok) {
                const error = await response.json();
                errorEl.textContent = '分配失败: ' + (error.error || error.message || '未知错误');
                return;
            }

            alert('Bot已分配到群聊!');
            modal.classList.remove('show');
            await loadGroups();

        } catch (error) {
            errorEl.textContent = '分配失败: ' + error.message;
        }
    };
}

// ============ 聊天 ============

async function populateBotSelector() {
    const select = document.getElementById('botSelector');
    select.innerHTML = '<option value="">-- 选择一个Bot --</option>';
    appState.bots.forEach(bot => {
        const option = document.createElement('option');
        option.value = bot.bot_id;
        option.dataset.token = bot.token;
        option.textContent = bot.name;
        select.appendChild(option);
    });
}

async function onBotSelected(e) {
    const botId = e.target.value;
    if (!botId) {
        appState.selectedBot = null;
        document.getElementById('groupSelector').innerHTML = '<option value="">-- 选择一个群聊 --</option>';
        return;
    }

    appState.selectedBot = appState.bots.find(b => b.bot_id === botId);
    await populateGroupSelector();
}

async function populateGroupSelector() {
    const select = document.getElementById('groupSelector');
    select.innerHTML = '<option value="">-- 选择一个群聊 --</option>';

    if (!appState.selectedBot) return;

    // 获取Bot所在的群聊（通过检查members）
    for (const group of appState.groups) {
        try {
            const response = await fetch(`${API_BASE_URL}/groups/${group.group_id}/members`);
            const data = await response.json();
            const members = data.members || [];
            const isMember = members.some(m => m.member_id === appState.selectedBot.bot_id);

            if (isMember) {
                const option = document.createElement('option');
                option.value = group.group_id;
                option.textContent = group.name + (group.group_code ? ` (${group.group_code})` : '');
                select.appendChild(option);
            }
        } catch (error) {
            console.error('检查群聊成员失败:', error);
        }
    }
}

async function onGroupSelected(e) {
    const groupId = e.target.value;
    if (!groupId) {
        appState.selectedGroup = null;
        document.getElementById('messagesArea').innerHTML = '<div class="messages-placeholder">选择Bot和群聊开始聊天</div>';
        return;
    }

    appState.selectedGroup = appState.groups.find(g => g.group_id === groupId);
    if (!appState.selectedGroup) return;

    // 加载消息历史
    await loadGroupMessages(groupId);

    // 连接WebSocket
    connectWebSocket();
}

async function loadGroupMessages(groupId) {
    try {
        const response = await fetch(`${API_BASE_URL}/groups/${groupId}/messages`, {
            method: 'GET',
            headers: { 'Authorization': `Bearer ${appState.token}` },
        });

        if (!response.ok) {
            throw new Error('获取历史记录失败');
        }

        const data = await response.json();
        appState.messages[groupId] = data.messages || [];
        renderMessages(appState.messages[groupId]);

    } catch (error) {
        console.error('加载消息错误:', error);
        document.getElementById('messagesArea').innerHTML = `<div class="error-message">加载消息失败: ${error.message}</div>`;
    }
}

function renderMessages(messages) {
    const messagesArea = document.getElementById('messagesArea');
    messagesArea.innerHTML = '';

    if (messages.length === 0) {
        messagesArea.innerHTML = '<div class="messages-placeholder">暂无消息</div>';
        return;
    }

    const sortedMessages = [...messages].sort((a, b) => new Date(a.created_at) - new Date(b.created_at));

    sortedMessages.forEach(msg => {
        const messageEl = document.createElement('div');
        const isOwn = msg.sender_id === appState.selectedBot?.bot_id;
        messageEl.className = 'message' + (isOwn ? ' own' : '');

        messageEl.innerHTML = `
            <div>
                ${!isOwn ? `<div class="message-sender">${escapeHtml(msg.sender_id)}</div>` : ''}
                <div class="message-bubble">${escapeHtml(typeof msg.content === 'string' ? msg.content : JSON.stringify(msg.content))}</div>
                <div class="message-time">${new Date(msg.created_at).toLocaleString()}</div>
            </div>
        `;

        messagesArea.appendChild(messageEl);
    });

    messagesArea.scrollTop = messagesArea.scrollHeight;
}

async function handleSendMessage(e) {
    e.preventDefault();

    if (!appState.selectedBot || !appState.selectedGroup) {
        alert('请先选择Bot和群聊');
        return;
    }

    const messageInput = document.getElementById('messageInput');
    const content = messageInput.value.trim();

    if (!content) return;

    try {
        const response = await fetch(`${API_BASE_URL}/message/send`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${appState.selectedBot.token}`,
            },
            body: JSON.stringify({
                group_id: appState.selectedGroup.group_id,
                content: content,
                msg_type: 'text',
            }),
        });

        if (!response.ok) {
            const error = await response.json();
            alert('发送失败: ' + (error.error || '未知错误'));
            return;
        }

        messageInput.value = '';

        // 本地显示消息
        const msg = {
            sender_id: appState.selectedBot.bot_id,
            content: content,
            created_at: new Date().toISOString(),
        };

        if (!appState.messages[appState.selectedGroup.group_id]) {
            appState.messages[appState.selectedGroup.group_id] = [];
        }
        appState.messages[appState.selectedGroup.group_id].push(msg);
        renderMessages(appState.messages[appState.selectedGroup.group_id]);

    } catch (error) {
        alert('网络错误: ' + error.message);
    }
}

function connectWebSocket() {
    if (appState.ws) {
        appState.ws.close();
    }

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
                    const groupId = data.group_id;

                    if (!appState.messages[groupId]) {
                        appState.messages[groupId] = [];
                    }

                    appState.messages[groupId].push({
                        sender_id: data.sender_id,
                        content: data.content,
                        created_at: data.created_at,
                    });

                    if (appState.selectedGroup && appState.selectedGroup.group_id === groupId) {
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

// ============ 工具函数 ============

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
