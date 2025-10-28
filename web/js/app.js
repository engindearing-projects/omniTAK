// OmniTAK Web Interface - Main Application
// Handles TAK server connections, certificate management, and real-time monitoring

const API_BASE = 'http://localhost:8080/api/v1';

// Application State
const state = {
    connections: [],
    stats: {
        messagesReceived: 0,
        messagesSent: 0,
        messagesFiltered: 0,
        messagesDuplicated: 0,
        throughput: 0,
        errors: 0
    },
    certificates: {
        clientCert: null,
        clientKey: null,
        caCert: null
    },
    systemStatus: 'offline'
};

// Initialize Application
document.addEventListener('DOMContentLoaded', () => {
    console.log('OmniTAK Web Interface Loaded');
    initializeEventListeners();
    checkSystemStatus();
    loadConnections();
    startStatusPolling();
});

// Event Listeners
function initializeEventListeners() {
    // Protocol change handler
    const protocolSelect = document.getElementById('protocol');
    protocolSelect.addEventListener('change', handleProtocolChange);

    // File upload handlers
    setupFileUpload('client-cert', 'client-cert-name');
    setupFileUpload('client-key', 'client-key-name');
    setupFileUpload('ca-cert', 'ca-cert-name');

    // Form submission
    const form = document.getElementById('connection-form');
    form.addEventListener('submit', handleAddConnection);

    // Test connection button
    const testBtn = document.getElementById('test-connection');
    testBtn.addEventListener('click', handleTestConnection);

    // Clear messages button
    const clearMessagesBtn = document.getElementById('clear-messages');
    clearMessagesBtn.addEventListener('click', clearMessages);
}

// Protocol Selection Handler
function handleProtocolChange(event) {
    const tlsSection = document.getElementById('tls-section');
    if (event.target.value === 'tls') {
        tlsSection.style.display = 'block';
    } else {
        tlsSection.style.display = 'none';
    }
}

// File Upload Setup
function setupFileUpload(inputId, displayId) {
    const input = document.getElementById(inputId);
    const display = document.getElementById(displayId);

    input.addEventListener('change', (event) => {
        const file = event.target.files[0];
        if (file) {
            display.textContent = file.name;

            // Store file for upload
            const fileType = inputId.replace('-', '_');
            readFileAsBase64(file, (base64Data) => {
                state.certificates[fileType] = {
                    name: file.name,
                    data: base64Data,
                    size: file.size
                };
                showToast(`${file.name} loaded successfully`, 'success');
            });
        }
    });
}

// Read file as Base64
function readFileAsBase64(file, callback) {
    const reader = new FileReader();
    reader.onload = (e) => {
        const base64 = e.target.result.split(',')[1];
        callback(base64);
    };
    reader.readAsDataURL(file);
}

// Handle Add Connection
async function handleAddConnection(event) {
    event.preventDefault();

    const host = document.getElementById('server-host').value;
    const port = document.getElementById('server-port').value;
    const address = `${host}:${port}`;

    const connectionData = {
        id: document.getElementById('connection-id').value,
        name: document.getElementById('connection-name').value,
        address: address,
        protocol: document.getElementById('protocol').value,
        priority: parseInt(document.getElementById('priority').value),
        autoReconnect: document.getElementById('auto-reconnect').checked,
        config: {
            connectTimeout: parseInt(document.getElementById('connect-timeout').value),
            readTimeout: parseInt(document.getElementById('read-timeout').value),
            retryAttempts: parseInt(document.getElementById('retry-attempts').value),
            bufferSize: parseInt(document.getElementById('buffer-size').value) * 1024
        }
    };

    // Add TLS configuration if protocol is TLS
    if (connectionData.protocol === 'tls') {
        const certPassword = document.getElementById('cert-password').value;

        connectionData.tls = {
            clientCert: state.certificates.client_cert,
            clientKey: state.certificates.client_key,
            caCert: state.certificates.ca_cert,
            password: certPassword || null,
            verifyHostname: document.getElementById('verify-hostname').checked,
            minTlsVersion: document.getElementById('tls-version').value
        };

        // Validate certificates
        if (!state.certificates.client_cert && !state.certificates.client_key) {
            showToast('Warning: TLS selected but no certificates uploaded', 'warning');
        }
    }

    try {
        // Send to backend API
        const response = await fetch(`${API_BASE}/connections`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(connectionData)
        });

        if (response.ok) {
            const result = await response.json();
            showToast(`Connection "${connectionData.name}" added successfully!`, 'success');

            // Add to state
            state.connections.push(connectionData);

            // Update UI
            renderConnections();

            // Reset form
            event.target.reset();
            clearCertificates();
        } else {
            const error = await response.json();
            throw new Error(error.message || 'Failed to add connection');
        }
    } catch (error) {
        console.error('Error adding connection:', error);
        showToast(`Error: ${error.message}`, 'error');

        // For demo purposes, add locally if backend is not available
        state.connections.push({
            ...connectionData,
            status: 'connecting',
            messagesRx: 0,
            messagesTx: 0
        });
        renderConnections();
        showToast(`Connection added (local mode)`, 'warning');
    }
}

// Handle Test Connection
async function handleTestConnection() {
    const host = document.getElementById('server-host').value;
    const port = document.getElementById('server-port').value;
    const protocol = document.getElementById('protocol').value;

    if (!host || !port) {
        showToast('Please enter server host and port', 'warning');
        return;
    }

    const address = `${host}:${port}`;

    showToast('Testing connection...', 'warning');

    try {
        const response = await fetch(`${API_BASE}/test-connection`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ address, protocol })
        });

        if (response.ok) {
            const result = await response.json();
            showToast(`Connection test successful! Latency: ${result.latency}ms`, 'success');
        } else {
            throw new Error('Connection test failed');
        }
    } catch (error) {
        console.error('Connection test error:', error);
        showToast('Connection test failed - server may be unreachable', 'error');
    }
}

// Clear Certificates
function clearCertificates() {
    state.certificates = {
        clientCert: null,
        clientKey: null,
        caCert: null
    };
    document.getElementById('client-cert-name').textContent = 'Choose file...';
    document.getElementById('client-key-name').textContent = 'Choose file...';
    document.getElementById('ca-cert-name').textContent = 'Choose file...';
}

// Render Connections
function renderConnections() {
    const connectionsList = document.getElementById('connections-list');

    if (state.connections.length === 0) {
        connectionsList.innerHTML = `
            <div class="empty-state">
                <p>No active connections. Add a TAK server above to get started.</p>
            </div>
        `;
        return;
    }

    connectionsList.innerHTML = state.connections.map(conn => {
        const statusClass = conn.status === 'connected' ? 'connected' :
                          conn.status === 'error' ? 'error' : '';
        const statusText = conn.status === 'connected' ? 'Connected' :
                          conn.status === 'connecting' ? 'Connecting...' :
                          conn.status === 'error' ? 'Error' : 'Disconnected';
        const statusBadgeClass = conn.status === 'connected' ? 'status-connected' :
                                conn.status === 'connecting' ? 'status-connecting' :
                                'status-disconnected';

        return `
            <div class="connection-item ${statusClass}" data-id="${conn.id}">
                <div class="connection-info">
                    <div class="connection-name">${conn.name}</div>
                    <div class="connection-details">
                        <span class="connection-badge badge-${conn.protocol}">${conn.protocol.toUpperCase()}</span>
                        <span>📍 ${conn.address}</span>
                        <span>📊 Priority: ${conn.priority}</span>
                        <span>📨 RX: ${conn.messagesRx || 0}</span>
                        <span>📤 TX: ${conn.messagesTx || 0}</span>
                    </div>
                </div>
                <div class="connection-status ${statusBadgeClass}">
                    ${statusText}
                </div>
                <div class="connection-actions">
                    <button class="btn btn-small btn-secondary" onclick="reconnectConnection('${conn.id}')">
                        🔄 Reconnect
                    </button>
                    <button class="btn btn-small btn-danger" onclick="removeConnection('${conn.id}')">
                        🗑️ Remove
                    </button>
                </div>
            </div>
        `;
    }).join('');
}

// Reconnect Connection
async function reconnectConnection(connectionId) {
    showToast('Reconnecting...', 'warning');

    try {
        const response = await fetch(`${API_BASE}/connections/${connectionId}/reconnect`, {
            method: 'POST'
        });

        if (response.ok) {
            showToast('Reconnected successfully', 'success');
            loadConnections();
        } else {
            throw new Error('Reconnection failed');
        }
    } catch (error) {
        console.error('Reconnection error:', error);
        showToast('Failed to reconnect', 'error');
    }
}

// Remove Connection
async function removeConnection(connectionId) {
    if (!confirm('Are you sure you want to remove this connection?')) {
        return;
    }

    try {
        const response = await fetch(`${API_BASE}/connections/${connectionId}`, {
            method: 'DELETE'
        });

        if (response.ok) {
            showToast('Connection removed', 'success');
            state.connections = state.connections.filter(c => c.id !== connectionId);
            renderConnections();
        } else {
            throw new Error('Failed to remove connection');
        }
    } catch (error) {
        console.error('Remove connection error:', error);
        showToast('Failed to remove connection', 'error');

        // Remove locally anyway
        state.connections = state.connections.filter(c => c.id !== connectionId);
        renderConnections();
    }
}

// Load Connections from API
async function loadConnections() {
    try {
        const response = await fetch(`${API_BASE}/connections`);
        if (response.ok) {
            const connections = await response.json();
            state.connections = connections;
            renderConnections();
        }
    } catch (error) {
        console.log('Backend not available, using local state');
    }
}

// Check System Status
async function checkSystemStatus() {
    try {
        const response = await fetch(`${API_BASE}/status`);
        if (response.ok) {
            const status = await response.json();
            updateSystemStatus('online');
            updateStats(status);
        } else {
            updateSystemStatus('offline');
        }
    } catch (error) {
        updateSystemStatus('offline');
    }
}

// Update System Status
function updateSystemStatus(status) {
    state.systemStatus = status;
    const statusElement = document.getElementById('system-status');
    statusElement.textContent = status === 'online' ? 'Online' : 'Offline';
    statusElement.className = `status-value ${status}`;
}

// Update Statistics
function updateStats(stats) {
    document.getElementById('active-connections').textContent = stats.activeConnections || 0;
    document.getElementById('message-count').textContent = stats.totalMessages || 0;
    document.getElementById('messages-received').textContent = stats.messagesReceived || 0;
    document.getElementById('messages-sent').textContent = stats.messagesSent || 0;
    document.getElementById('messages-filtered').textContent = stats.messagesFiltered || 0;
    document.getElementById('messages-duplicated').textContent = stats.messagesDuplicated || 0;
    document.getElementById('throughput').textContent = `${(stats.throughput || 0).toFixed(2)}/s`;
    document.getElementById('errors').textContent = stats.errors || 0;
}

// Start Status Polling
function startStatusPolling() {
    setInterval(() => {
        checkSystemStatus();
        loadConnections();
    }, 5000); // Poll every 5 seconds
}

// Add Message to Log
function addMessageToLog(message) {
    const messagesLog = document.getElementById('messages-log');
    const autoScroll = document.getElementById('auto-scroll').checked;

    // Remove empty state if present
    const emptyState = messagesLog.querySelector('.empty-state');
    if (emptyState) {
        emptyState.remove();
    }

    const timestamp = new Date().toLocaleTimeString();
    const messageElement = document.createElement('div');
    messageElement.className = 'message-entry';
    messageElement.innerHTML = `
        <span class="message-time">[${timestamp}]</span>
        <span class="message-source">${message.source}</span>
        <span class="message-type">${message.type}</span>
        ${message.content}
    `;

    messagesLog.appendChild(messageElement);

    // Limit to last 100 messages
    while (messagesLog.children.length > 100) {
        messagesLog.removeChild(messagesLog.firstChild);
    }

    // Auto-scroll if enabled
    if (autoScroll) {
        messagesLog.scrollTop = messagesLog.scrollHeight;
    }
}

// Clear Messages
function clearMessages() {
    const messagesLog = document.getElementById('messages-log');
    messagesLog.innerHTML = `
        <div class="empty-state">
            <p>No messages yet. Messages will appear here when connections are active.</p>
        </div>
    `;
}

// Show Toast Notification
function showToast(message, type = 'success') {
    const toast = document.getElementById('toast');
    toast.textContent = message;
    toast.className = `toast ${type} show`;

    setTimeout(() => {
        toast.classList.remove('show');
    }, 3000);
}

// WebSocket Connection for Real-Time Updates
function connectWebSocket() {
    const ws = new WebSocket('ws://localhost:8080/api/v1/stream');

    ws.onopen = () => {
        console.log('WebSocket connected');
        showToast('Real-time monitoring connected', 'success');
    };

    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);

            if (data.type === 'message') {
                addMessageToLog(data.message);
            } else if (data.type === 'stats') {
                updateStats(data.stats);
            } else if (data.type === 'connection_update') {
                loadConnections();
            }
        } catch (error) {
            console.error('WebSocket message error:', error);
        }
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
        console.log('WebSocket disconnected, attempting to reconnect...');
        setTimeout(connectWebSocket, 5000);
    };
}

// Attempt WebSocket connection (disabled until backend WebSocket is implemented)
// setTimeout(connectWebSocket, 2000);

// Export functions to global scope for onclick handlers
window.reconnectConnection = reconnectConnection;
window.removeConnection = removeConnection;
