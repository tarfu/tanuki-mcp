// Dashboard state
let lastData = null;

// Format uptime
function formatUptime(seconds) {
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${mins}m`;
}

// Format timestamp
function formatTime(timestamp) {
    const date = new Date(timestamp * 1000);
    return date.toLocaleTimeString();
}

// Format relative time
function formatRelativeTime(timestamp) {
    if (!timestamp) return '-';
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;
    if (diff < 60) return 'Just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
}

// Update overview stats
function updateOverview(data) {
    document.getElementById('total-requests').textContent = data.total_requests.toLocaleString();
    document.getElementById('total-errors').textContent = data.total_errors.toLocaleString();
    document.getElementById('requests-rate').textContent = data.requests_per_minute.toFixed(1);
    document.getElementById('uptime').textContent = formatUptime(data.uptime_secs);
}

// Update config section
function updateConfig(config) {
    document.getElementById('server-name').textContent = config.server_name;
    document.getElementById('server-version').textContent = config.server_version;
    document.getElementById('gitlab-url').textContent = config.gitlab_url;
    document.getElementById('transport-mode').textContent = config.transport_mode;
    document.getElementById('access-level').textContent = config.access_level;
    document.getElementById('tool-count').textContent = config.tool_count;
}

// Update projects table
function updateProjects(projects) {
    const tbody = document.getElementById('projects-body');

    if (projects.length === 0) {
        tbody.innerHTML = '<tr class="empty-row"><td colspan="4">No projects accessed yet</td></tr>';
        return;
    }

    tbody.innerHTML = projects.map(p => `
        <tr>
            <td><code>${escapeHtml(p.name)}</code></td>
            <td>${p.access_count}</td>
            <td>${p.tools_used.slice(0, 3).map(t => t.tool).join(', ')}${p.tools_used.length > 3 ? '...' : ''}</td>
            <td>${formatRelativeTime(p.last_accessed)}</td>
        </tr>
    `).join('');
}

// Update tools table
function updateTools(tools) {
    const tbody = document.getElementById('tools-body');

    if (tools.length === 0) {
        tbody.innerHTML = '<tr class="empty-row"><td colspan="4">No tools called yet</td></tr>';
        return;
    }

    tbody.innerHTML = tools.slice(0, 15).map(t => `
        <tr>
            <td><code>${escapeHtml(t.name)}</code></td>
            <td>${t.call_count}</td>
            <td>${t.error_count > 0 ? `<span class="status-badge status-error">${t.error_count}</span>` : '-'}</td>
            <td>${t.avg_duration_ms}ms</td>
        </tr>
    `).join('');
}

// Update categories grid
function updateCategories(categories) {
    const grid = document.getElementById('categories-grid');

    if (categories.length === 0) {
        grid.innerHTML = '<div class="category-placeholder">No category data yet</div>';
        return;
    }

    grid.innerHTML = categories.map(c => `
        <div class="category-card">
            <div class="category-name">${escapeHtml(c.name)}</div>
            <div class="category-count">${c.call_count}</div>
            ${c.error_count > 0 ? `<div class="category-errors">${c.error_count} errors</div>` : ''}
        </div>
    `).join('');
}

// Update recent requests table
function updateRecent(requests) {
    const tbody = document.getElementById('recent-body');

    if (requests.length === 0) {
        tbody.innerHTML = '<tr class="empty-row"><td colspan="5">No recent requests</td></tr>';
        return;
    }

    // Show most recent first
    const reversed = [...requests].reverse();

    tbody.innerHTML = reversed.slice(0, 20).map(r => `
        <tr>
            <td>${formatTime(r.timestamp)}</td>
            <td><code>${escapeHtml(r.tool)}</code></td>
            <td>${r.project ? `<code>${escapeHtml(r.project)}</code>` : '-'}</td>
            <td><span class="status-badge ${r.success ? 'status-success' : 'status-error'}">${r.success ? 'OK' : 'Error'}</span></td>
            <td>${r.duration_ms}ms</td>
        </tr>
    `).join('');
}

// Escape HTML to prevent XSS
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Fetch and update metrics
async function fetchMetrics() {
    try {
        const response = await fetch('/api/metrics');
        const data = await response.json();

        updateOverview(data);
        updateProjects(data.projects);
        updateTools(data.tools);
        updateCategories(data.categories);
        updateRecent(data.recent_requests);

        document.getElementById('status-text').textContent = 'Connected';
        document.querySelector('.status-dot').style.backgroundColor = 'var(--accent-green)';

        lastData = data;
    } catch (error) {
        console.error('Failed to fetch metrics:', error);
        document.getElementById('status-text').textContent = 'Disconnected';
        document.querySelector('.status-dot').style.backgroundColor = 'var(--accent-red)';
    }
}

// Fetch config once
async function fetchConfig() {
    try {
        const response = await fetch('/api/config');
        const config = await response.json();
        updateConfig(config);
    } catch (error) {
        console.error('Failed to fetch config:', error);
    }
}

// Check for updates
async function checkForUpdates() {
    try {
        const response = await fetch('/api/update');
        const status = await response.json();

        if (status.update_available && status.latest_version) {
            document.getElementById('current-ver').textContent = 'v' + status.current_version;
            document.getElementById('latest-ver').textContent = 'v' + status.latest_version;
            document.getElementById('update-banner').style.display = 'block';
        }
    } catch (error) {
        console.error('Failed to check for updates:', error);
    }
}

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    fetchConfig();
    fetchMetrics();
    checkForUpdates();

    // Refresh metrics every 2 seconds
    setInterval(fetchMetrics, 2000);

    // Check for updates every 30 minutes
    setInterval(checkForUpdates, 30 * 60 * 1000);
});
