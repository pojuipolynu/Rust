const apiBase = 'http://127.0.0.1:8080';

document.getElementById('login')?.addEventListener('click', async () => {
    const username = document.getElementById('username').value.trim();
    const password = document.getElementById('password').value.trim();

    if (!username || !password) {
        alert('Please fill in both fields.');
        return;
    }

    try {
        const response = await fetch(`${apiBase}/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password }),
        });

        const result = await response.json();
        if (result.success) {
            localStorage.setItem('username', username);
            window.location.href = 'chat.html';
        } else {
            alert(result.message);
        }
    } catch (error) {
        console.error('Login error:', error);
        alert('Failed to login. Please try again.');
    }
});

document.getElementById('register')?.addEventListener('click', async () => {
    const username = document.getElementById('username').value.trim();
    const password = document.getElementById('password').value.trim();

    if (!username || !password) {
        alert('Please fill in both fields.');
        return;
    }

    try {
        const response = await fetch(`${apiBase}/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password }),
        });

        const result = await response.json();
        alert(result.message);
    } catch (error) {
        console.error('Register error:', error);
        alert('Failed to register. Please try again.');
    }
});

if (window.location.pathname.endsWith('chat.html')) {
    const username = localStorage.getItem('username');
    if (!username) {
        window.location.href = 'index.html';
    }

    document.getElementById('username-display').textContent = `Logged in as: ${username}`;

    const messages = document.getElementById('messages');
    const input = document.getElementById('input');
    const ws = new WebSocket('ws://127.0.0.1:8080/ws');

    ws.onopen = async () => {
        console.log('WebSocket connection established.');
        try {
            const response = await fetch(`${apiBase}/messages`);
            const savedMessages = await response.json();
            for (const [user, msg] of savedMessages) {
                const messageDiv = document.createElement('div');
                messageDiv.className = 'message';
                messageDiv.textContent = `${user}: ${msg}`;
                messages.appendChild(messageDiv);
            }
            messages.scrollTop = messages.scrollHeight;
        } catch (error) {
            console.error('Failed to load messages:', error);
        }
    };

    ws.onclose = () => console.log('WebSocket connection closed.');
    ws.onerror = (err) => console.error('WebSocket error:', err);

    ws.onmessage = (event) => {
        const messageDiv = document.createElement('div');
        messageDiv.className = 'message';
        messageDiv.textContent = event.data;
        messages.appendChild(messageDiv);
        messages.scrollTop = messages.scrollHeight;
    };

    input.addEventListener('keypress', (e) => {
        if (e.key === 'Enter' && input.value) {
            ws.send(`${username}: ${input.value}`);
            input.value = '';
        }
    });

    document.getElementById('logout').addEventListener('click', () => {
        localStorage.removeItem('username');
        ws.close();
        window.location.href = 'index.html';
    });
}
