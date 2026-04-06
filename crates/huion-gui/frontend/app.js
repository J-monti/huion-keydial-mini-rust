// ── State ────────────────────────────────────────────
let config = null;
let originalConfig = null;
let availableKeys = [];
let installedApps = [];
let buttonLayout = [];
let currentProfile = 'default';
let activeButtonHid = null;
let editingButtonHid = null;
let editingDial = null;
let dirty = false;

// ── Tauri IPC ────────────────────────────────────────
function invoke(cmd, args) {
    return window.__TAURI__.core.invoke(cmd, args);
}

// ── Init ─────────────────────────────────────────────
document.addEventListener('DOMContentLoaded', async () => {
    try {
        const [cfg, keys, apps, layout] = await Promise.all([
            invoke('load_config'),
            invoke('list_available_keys'),
            invoke('list_installed_apps'),
            invoke('get_button_layout'),
        ]);
        config = cfg;
        originalConfig = structuredClone(cfg);
        availableKeys = keys;
        installedApps = apps;
        buttonLayout = layout;

        renderProfileSelector();
        renderDeviceOverlays();
        renderButtonList();
        renderDialSettings();
        renderWmClassSection();
        updateDirtyState();
        bindGlobalEvents();
    } catch (err) {
        console.error('Failed to initialize:', err);
        document.getElementById('config-panel').innerHTML =
            '<p style="color:var(--danger);padding:20px;">Failed to load: ' + err + '</p>';
    }
});

// ── Key Formatting ───────────────────────────────────
const KEY_DISPLAY_MAP = {
    'KEY_LEFTCTRL': 'Ctrl (L)', 'KEY_RIGHTCTRL': 'Ctrl (R)',
    'KEY_LEFTSHIFT': 'Shift (L)', 'KEY_RIGHTSHIFT': 'Shift (R)',
    'KEY_LEFTALT': 'Alt (L)', 'KEY_RIGHTALT': 'Alt (R)',
    'KEY_LEFTMETA': 'Super (L)', 'KEY_RIGHTMETA': 'Super (R)',
    'KEY_LEFTBRACE': '[', 'KEY_RIGHTBRACE': ']',
    'KEY_BACKSLASH': '\\', 'KEY_SEMICOLON': ';',
    'KEY_APOSTROPHE': "'", 'KEY_GRAVE': '`',
    'KEY_COMMA': ',', 'KEY_DOT': '.', 'KEY_SLASH': '/',
    'KEY_MINUS': '-', 'KEY_EQUAL': '=',
    'KEY_ENTER': 'Enter', 'KEY_ESC': 'Esc',
    'KEY_BACKSPACE': 'Backspace', 'KEY_TAB': 'Tab',
    'KEY_SPACE': 'Space', 'KEY_DELETE': 'Delete',
    'KEY_INSERT': 'Insert', 'KEY_HOME': 'Home', 'KEY_END': 'End',
    'KEY_PAGEUP': 'PgUp', 'KEY_PAGEDOWN': 'PgDn',
    'KEY_UP': 'Up', 'KEY_DOWN': 'Down', 'KEY_LEFT': 'Left', 'KEY_RIGHT': 'Right',
    'KEY_CAPSLOCK': 'CapsLock', 'KEY_NUMLOCK': 'NumLock',
    'KEY_SCROLLLOCK': 'ScrollLock', 'KEY_PAUSE': 'Pause',
    'KEY_SYSRQ': 'SysRq', 'KEY_PRINTSCREEN': 'PrtSc',
    'KEY_VOLUMEUP': 'Vol Up', 'KEY_VOLUMEDOWN': 'Vol Down', 'KEY_MUTE': 'Mute',
    'KEY_PLAYPAUSE': 'Play/Pause', 'KEY_NEXTSONG': 'Next', 'KEY_PREVIOUSSONG': 'Prev',
    'KEY_STOPCD': 'Stop', 'KEY_MENU': 'Menu',
};

function formatKeyName(raw) {
    if (!raw) return '';
    if (KEY_DISPLAY_MAP[raw]) return KEY_DISPLAY_MAP[raw];
    // Strip KEY_ prefix, title-case the remainder
    const name = raw.replace(/^KEY_/, '');
    return name.charAt(0).toUpperCase() + name.slice(1).toLowerCase();
}

// ── Profile Selector ─────────────────────────────────
function renderProfileSelector() {
    const select = document.getElementById('profile-select');
    select.innerHTML = '';

    const defaultOpt = document.createElement('option');
    defaultOpt.value = 'default';
    defaultOpt.textContent = 'Default';
    select.appendChild(defaultOpt);

    for (const name of Object.keys(config.profiles || {})) {
        const opt = document.createElement('option');
        opt.value = name;
        opt.textContent = name;
        select.appendChild(opt);
    }

    select.value = currentProfile;
    updateDeleteButtonVisibility();

    select.onchange = () => {
        closeAllEditors();
        currentProfile = select.value;
        activeButtonHid = null;
        renderButtonList();
        renderDialSettings();
        renderWmClassSection();
        updateDeleteButtonVisibility();
        updateOverlayHighlights();
    };
}

function updateDeleteButtonVisibility() {
    const btn = document.getElementById('btn-delete-profile');
    btn.classList.toggle('hidden', currentProfile === 'default');
}

// ── App Picker ───────────────────────────────────────
function bindGlobalEvents() {
    const addBtn = document.getElementById('btn-add-profile');
    const picker = document.getElementById('app-picker');
    const searchInput = document.getElementById('app-search');
    const closeBtn = document.getElementById('btn-close-app-picker');
    const deleteBtn = document.getElementById('btn-delete-profile');
    const saveBtn = document.getElementById('btn-save');
    const discardBtn = document.getElementById('btn-discard');

    addBtn.onclick = () => {
        picker.classList.toggle('hidden');
        if (!picker.classList.contains('hidden')) {
            searchInput.value = '';
            renderAppList('');
            searchInput.focus();
        }
    };

    closeBtn.onclick = () => picker.classList.add('hidden');

    searchInput.oninput = () => renderAppList(searchInput.value);

    deleteBtn.onclick = () => {
        if (currentProfile === 'default') return;
        delete config.profiles[currentProfile];
        currentProfile = 'default';
        renderProfileSelector();
        renderButtonList();
        renderDialSettings();
        renderWmClassSection();
        updateOverlayHighlights();
        markDirty();
    };

    saveBtn.onclick = saveConfig;
    discardBtn.onclick = discardChanges;

    // Close editors on click outside
    document.addEventListener('mousedown', (e) => {
        // Close app picker if clicking outside
        if (!picker.classList.contains('hidden') &&
            !picker.contains(e.target) &&
            e.target !== addBtn) {
            picker.classList.add('hidden');
        }

        // Close key editor if clicking outside
        if (editingButtonHid !== null) {
            const editRow = document.querySelector('.button-row.editing');
            if (editRow && !editRow.contains(e.target)) {
                commitButtonEdit();
            }
        }

        // Close dial editor if clicking outside
        if (editingDial !== null) {
            const editRow = document.querySelector('.dial-row.editing');
            if (editRow && !editRow.contains(e.target)) {
                commitDialEdit();
            }
        }
    });
}

function renderAppList(filter) {
    const list = document.getElementById('app-list');
    list.innerHTML = '';
    const lowerFilter = filter.toLowerCase();
    const existingProfiles = new Set(Object.keys(config.profiles || {}));

    for (const app of installedApps) {
        if (existingProfiles.has(app.wm_class)) continue;
        if (lowerFilter && !app.name.toLowerCase().includes(lowerFilter) &&
            !app.wm_class.toLowerCase().includes(lowerFilter)) continue;

        const li = document.createElement('li');
        li.innerHTML = `<span>${escapeHtml(app.name)}</span><span class="app-wm">${escapeHtml(app.wm_class)}</span>`;
        li.onclick = () => addProfileFromApp(app);
        list.appendChild(li);
    }

    if (list.children.length === 0) {
        const li = document.createElement('li');
        li.textContent = 'No matching apps found';
        li.style.color = 'var(--text-dim)';
        li.style.cursor = 'default';
        list.appendChild(li);
    }
}

function addProfileFromApp(app) {
    const profileName = app.wm_class;
    if (!config.profiles) config.profiles = {};
    config.profiles[profileName] = {
        wm_class: [app.wm_class],
        button_mappings: {},
        dial: { cw: null, ccw: null, click: null },
    };
    currentProfile = profileName;
    document.getElementById('app-picker').classList.add('hidden');
    renderProfileSelector();
    renderButtonList();
    renderDialSettings();
    renderWmClassSection();
    updateOverlayHighlights();
    markDirty();
}

// ── Device Overlays ──────────────────────────────────
function renderDeviceOverlays() {
    const container = document.getElementById('button-overlays');
    container.innerHTML = '';

    // Grid parameters (percentages of the image)
    const gridTop = 33.5;
    const gridLeft = 9.5;
    const cellW = 19.5;
    const cellH = 10.5;
    const gapX = 1.5;
    const gapY = 1.8;

    for (const btn of buttonLayout) {
        const div = document.createElement('div');
        div.className = 'btn-overlay';
        if (!btn.remappable) div.classList.add('non-remappable');
        div.dataset.hid = btn.hid_code;
        div.textContent = btn.label;

        const x = gridLeft + btn.col * (cellW + gapX);
        const y = gridTop + btn.row * (cellH + gapY);
        const w = btn.col_span * cellW + (btn.col_span - 1) * gapX;
        const h = btn.row_span * cellH + (btn.row_span - 1) * gapY;

        div.style.left = x + '%';
        div.style.top = y + '%';
        div.style.width = w + '%';
        div.style.height = h + '%';

        if (btn.remappable) {
            div.onclick = () => {
                activeButtonHid = btn.hid_code;
                updateOverlayHighlights();
                scrollToButton(btn.hid_code);
            };
        }

        container.appendChild(div);
    }
}

function updateOverlayHighlights() {
    document.querySelectorAll('.btn-overlay').forEach(el => {
        el.classList.toggle('active', el.dataset.hid === activeButtonHid);
    });
}

function scrollToButton(hid) {
    const row = document.querySelector(`.button-row[data-hid="${hid}"]`);
    if (row) {
        row.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        row.classList.add('active');
        setTimeout(() => row.classList.remove('active'), 800);
    }
}

// ── Button List ──────────────────────────────────────
function getRemappableButtons() {
    return buttonLayout.filter(b => b.remappable);
}

function getCurrentButtonMapping(hid) {
    if (currentProfile === 'default') {
        return { keys: config.default.button_mappings[hid] || [], inherited: false };
    }
    const profile = config.profiles[currentProfile];
    if (profile && profile.button_mappings && profile.button_mappings[hid]) {
        return { keys: profile.button_mappings[hid], inherited: false };
    }
    return { keys: config.default.button_mappings[hid] || [], inherited: true };
}

function setCurrentButtonMapping(hid, keys) {
    if (currentProfile === 'default') {
        if (keys.length === 0) {
            delete config.default.button_mappings[hid];
        } else {
            config.default.button_mappings[hid] = keys;
        }
    } else {
        const profile = config.profiles[currentProfile];
        if (!profile.button_mappings) profile.button_mappings = {};
        if (keys.length === 0) {
            // For app profiles, clearing means removing the override
            delete profile.button_mappings[hid];
        } else {
            profile.button_mappings[hid] = keys;
        }
    }
    markDirty();
}

function renderButtonList() {
    const list = document.getElementById('button-list');
    list.innerHTML = '';

    for (const btn of getRemappableButtons()) {
        const row = createButtonRow(btn);
        list.appendChild(row);
    }
}

function createButtonRow(btn) {
    const { keys, inherited } = getCurrentButtonMapping(btn.hid_code);
    const row = document.createElement('div');
    row.className = 'button-row';
    row.dataset.hid = btn.hid_code;

    const numEl = document.createElement('span');
    numEl.className = 'button-number';
    numEl.textContent = btn.label;

    const keysEl = document.createElement('div');
    keysEl.className = 'button-keys';

    if (keys.length === 0) {
        const noMap = document.createElement('span');
        noMap.className = 'no-mapping';
        noMap.textContent = 'No mapping';
        keysEl.appendChild(noMap);
    } else {
        for (const k of keys) {
            const tag = document.createElement('span');
            tag.className = 'key-tag';
            if (inherited) tag.classList.add('inherited');
            tag.textContent = formatKeyName(k);
            keysEl.appendChild(tag);
        }
        if (inherited) {
            const defLabel = document.createElement('span');
            defLabel.className = 'default-label';
            defLabel.textContent = '(default)';
            keysEl.appendChild(defLabel);
        }
    }

    row.appendChild(numEl);
    row.appendChild(keysEl);

    row.onclick = (e) => {
        if (row.classList.contains('editing')) return;
        closeAllEditors();
        editingButtonHid = btn.hid_code;
        activeButtonHid = btn.hid_code;
        updateOverlayHighlights();
        enterButtonEditMode(row, btn);
    };

    return row;
}

// ── Key Combo Editor ─────────────────────────────────
function enterButtonEditMode(row, btn) {
    const { keys } = getCurrentButtonMapping(btn.hid_code);
    const editKeys = [...keys];

    row.classList.add('editing');
    row.innerHTML = '';

    // Re-add button number
    const numEl = document.createElement('span');
    numEl.className = 'button-number';
    numEl.textContent = btn.label;

    const editor = document.createElement('div');
    editor.className = 'key-editor';

    const topRow = document.createElement('div');
    topRow.className = 'key-editor-top';
    topRow.appendChild(numEl);

    const tagsContainer = document.createElement('div');
    tagsContainer.className = 'key-editor-tags';

    const inputWrap = document.createElement('div');
    inputWrap.className = 'key-editor-input-wrap';

    const input = document.createElement('input');
    input.type = 'text';
    input.placeholder = 'Type to search keys...';
    input.autocomplete = 'off';

    const suggestions = document.createElement('div');
    suggestions.className = 'key-suggestions hidden';

    inputWrap.appendChild(input);
    inputWrap.appendChild(suggestions);

    topRow.appendChild(tagsContainer);
    topRow.appendChild(inputWrap);

    const actions = document.createElement('div');
    actions.className = 'key-editor-actions';

    const clearBtn = document.createElement('button');
    clearBtn.className = 'btn btn-secondary';
    clearBtn.textContent = currentProfile === 'default' ? 'Clear' : 'Revert to Default';
    clearBtn.onclick = (e) => {
        e.stopPropagation();
        editKeys.length = 0;
        if (currentProfile !== 'default') {
            const profile = config.profiles[currentProfile];
            delete profile.button_mappings[btn.hid_code];
        } else {
            delete config.default.button_mappings[btn.hid_code];
        }
        markDirty();
        closeAllEditors();
        renderButtonList();
    };

    const doneBtn = document.createElement('button');
    doneBtn.className = 'btn btn-primary';
    doneBtn.textContent = 'Done';
    doneBtn.onclick = (e) => {
        e.stopPropagation();
        setCurrentButtonMapping(btn.hid_code, editKeys);
        closeAllEditors();
        renderButtonList();
    };

    actions.appendChild(clearBtn);
    actions.appendChild(doneBtn);

    editor.appendChild(topRow);
    editor.appendChild(actions);
    row.appendChild(editor);

    let highlightedIdx = -1;

    function renderTags() {
        tagsContainer.innerHTML = '';
        for (let i = 0; i < editKeys.length; i++) {
            const tag = document.createElement('span');
            tag.className = 'key-tag';
            tag.innerHTML = `${escapeHtml(formatKeyName(editKeys[i]))}<span class="remove-key" data-idx="${i}">&times;</span>`;
            tag.querySelector('.remove-key').onclick = (e) => {
                e.stopPropagation();
                editKeys.splice(i, 1);
                renderTags();
            };
            tagsContainer.appendChild(tag);
        }
    }

    function filterKeys(query) {
        const lower = query.toLowerCase();
        if (!lower) return [];
        return availableKeys.filter(k => {
            if (editKeys.includes(k)) return false;
            const display = formatKeyName(k).toLowerCase();
            const raw = k.toLowerCase();
            return display.includes(lower) || raw.includes(lower);
        }).slice(0, 20);
    }

    function renderSuggestions(matches) {
        suggestions.innerHTML = '';
        highlightedIdx = -1;
        if (matches.length === 0) {
            suggestions.classList.add('hidden');
            return;
        }
        suggestions.classList.remove('hidden');
        for (let i = 0; i < matches.length; i++) {
            const div = document.createElement('div');
            div.className = 'suggestion';
            div.innerHTML = `${escapeHtml(formatKeyName(matches[i]))}<span class="raw-name">${escapeHtml(matches[i])}</span>`;
            div.onmousedown = (e) => {
                e.preventDefault();
                e.stopPropagation();
                editKeys.push(matches[i]);
                input.value = '';
                renderTags();
                renderSuggestions([]);
                input.focus();
            };
            suggestions.appendChild(div);
        }
    }

    input.oninput = () => {
        const matches = filterKeys(input.value);
        renderSuggestions(matches);
    };

    input.onkeydown = (e) => {
        const items = suggestions.querySelectorAll('.suggestion');
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            highlightedIdx = Math.min(highlightedIdx + 1, items.length - 1);
            updateHighlight(items);
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            highlightedIdx = Math.max(highlightedIdx - 1, 0);
            updateHighlight(items);
        } else if (e.key === 'Enter') {
            e.preventDefault();
            if (highlightedIdx >= 0 && items[highlightedIdx]) {
                items[highlightedIdx].onmousedown(e);
            } else if (input.value === '') {
                // Commit edit
                setCurrentButtonMapping(btn.hid_code, editKeys);
                closeAllEditors();
                renderButtonList();
            }
        } else if (e.key === 'Escape') {
            e.preventDefault();
            closeAllEditors();
            renderButtonList();
        } else if (e.key === 'Backspace' && input.value === '' && editKeys.length > 0) {
            editKeys.pop();
            renderTags();
        }
    };

    function updateHighlight(items) {
        items.forEach((el, i) => {
            el.classList.toggle('highlighted', i === highlightedIdx);
        });
        if (highlightedIdx >= 0 && items[highlightedIdx]) {
            items[highlightedIdx].scrollIntoView({ block: 'nearest' });
        }
    }

    renderTags();
    setTimeout(() => input.focus(), 0);
}

function commitButtonEdit() {
    if (editingButtonHid === null) return;
    const row = document.querySelector(`.button-row.editing`);
    if (!row) return;
    // Gather current tags from the editor
    const tags = row.querySelectorAll('.key-editor-tags .key-tag');
    const keys = [];
    tags.forEach(tag => {
        // Find the raw key by matching display name back
        const display = tag.childNodes[0].textContent.trim();
        const raw = availableKeys.find(k => formatKeyName(k) === display);
        if (raw) keys.push(raw);
    });
    setCurrentButtonMapping(editingButtonHid, keys);
    closeAllEditors();
    renderButtonList();
}

function closeAllEditors() {
    editingButtonHid = null;
    editingDial = null;
}

// ── Dial Settings ────────────────────────────────────

// Normalize dial value: always return an array or null.
// Handles legacy single-string format and new array format.
function normalizeDialValue(raw) {
    if (raw === null || raw === undefined) return null;
    if (typeof raw === 'string') return [raw];
    if (Array.isArray(raw) && raw.length > 0) return raw;
    return null;
}

function getCurrentDialValue(direction) {
    if (currentProfile === 'default') {
        return { value: normalizeDialValue(config.default.dial[direction]), inherited: false };
    }
    const profile = config.profiles[currentProfile];
    if (profile && profile.dial && profile.dial[direction] !== null && profile.dial[direction] !== undefined) {
        return { value: normalizeDialValue(profile.dial[direction]), inherited: false };
    }
    return { value: normalizeDialValue(config.default.dial[direction]), inherited: true };
}

function setCurrentDialValue(direction, value) {
    // value is an array of key names or null
    if (currentProfile === 'default') {
        config.default.dial[direction] = value;
    } else {
        const profile = config.profiles[currentProfile];
        if (!profile.dial) profile.dial = { cw: null, ccw: null, click: null };
        profile.dial[direction] = value;
    }
    markDirty();
}

function renderDialSettings() {
    const directions = ['cw', 'ccw', 'click'];
    for (const dir of directions) {
        const row = document.querySelector(`.dial-row[data-dial="${dir}"]`);
        const valueEl = row.querySelector('.dial-value');
        const { value, inherited } = getCurrentDialValue(dir);

        row.classList.remove('editing');
        valueEl.innerHTML = '';

        if (value && value.length > 0) {
            for (const k of value) {
                const tag = document.createElement('span');
                tag.className = 'key-tag';
                if (inherited) tag.classList.add('inherited');
                tag.textContent = formatKeyName(k);
                valueEl.appendChild(tag);
            }
            if (inherited) {
                const defLabel = document.createElement('span');
                defLabel.className = 'default-label';
                defLabel.textContent = '(default)';
                valueEl.appendChild(defLabel);
            }
        } else {
            const noMap = document.createElement('span');
            noMap.className = 'no-mapping';
            noMap.textContent = 'Not set';
            valueEl.appendChild(noMap);
        }

        row.onclick = (e) => {
            if (row.classList.contains('editing')) return;
            closeAllEditors();
            editingDial = dir;
            enterDialEditMode(row, dir);
        };
    }
}

function enterDialEditMode(row, direction) {
    const { value } = getCurrentDialValue(direction);
    const editKeys = value ? [...value] : [];

    row.classList.add('editing');

    const label = row.querySelector('.dial-label').cloneNode(true);
    const valueEl = row.querySelector('.dial-value');
    valueEl.innerHTML = '';

    const editor = document.createElement('div');
    editor.className = 'key-editor';

    const topRow = document.createElement('div');
    topRow.className = 'key-editor-top';
    topRow.appendChild(label);

    const tagsContainer = document.createElement('div');
    tagsContainer.className = 'key-editor-tags';

    const inputWrap = document.createElement('div');
    inputWrap.className = 'key-editor-input-wrap';

    const input = document.createElement('input');
    input.type = 'text';
    input.placeholder = 'Type to search keys...';
    input.autocomplete = 'off';

    const suggestions = document.createElement('div');
    suggestions.className = 'key-suggestions hidden';

    inputWrap.appendChild(input);
    inputWrap.appendChild(suggestions);

    topRow.appendChild(tagsContainer);
    topRow.appendChild(inputWrap);

    const actions = document.createElement('div');
    actions.className = 'key-editor-actions';

    const clearBtn = document.createElement('button');
    clearBtn.className = 'btn btn-secondary';
    clearBtn.textContent = currentProfile === 'default' ? 'Clear' : 'Revert to Default';
    clearBtn.onclick = (e) => {
        e.stopPropagation();
        if (currentProfile !== 'default') {
            const profile = config.profiles[currentProfile];
            if (profile.dial) profile.dial[direction] = null;
        } else {
            config.default.dial[direction] = null;
        }
        markDirty();
        closeAllEditors();
        renderDialSettings();
    };

    const doneBtn = document.createElement('button');
    doneBtn.className = 'btn btn-primary';
    doneBtn.textContent = 'Done';
    doneBtn.onclick = (e) => {
        e.stopPropagation();
        setCurrentDialValue(direction, editKeys.length > 0 ? editKeys : null);
        closeAllEditors();
        renderDialSettings();
    };

    actions.appendChild(clearBtn);
    actions.appendChild(doneBtn);

    editor.appendChild(topRow);
    editor.appendChild(actions);
    valueEl.appendChild(editor);

    let highlightedIdx = -1;

    function renderTags() {
        tagsContainer.innerHTML = '';
        for (let i = 0; i < editKeys.length; i++) {
            const tag = document.createElement('span');
            tag.className = 'key-tag';
            tag.innerHTML = `${escapeHtml(formatKeyName(editKeys[i]))}<span class="remove-key" data-idx="${i}">&times;</span>`;
            tag.querySelector('.remove-key').onclick = (e) => {
                e.stopPropagation();
                editKeys.splice(i, 1);
                renderTags();
            };
            tagsContainer.appendChild(tag);
        }
    }

    function filterKeys(query) {
        const lower = query.toLowerCase();
        if (!lower) return [];
        return availableKeys.filter(k => {
            if (editKeys.includes(k)) return false;
            const display = formatKeyName(k).toLowerCase();
            const raw = k.toLowerCase();
            return display.includes(lower) || raw.includes(lower);
        }).slice(0, 20);
    }

    function renderSuggestions(matches) {
        suggestions.innerHTML = '';
        highlightedIdx = -1;
        if (matches.length === 0) {
            suggestions.classList.add('hidden');
            return;
        }
        suggestions.classList.remove('hidden');
        for (let i = 0; i < matches.length; i++) {
            const div = document.createElement('div');
            div.className = 'suggestion';
            div.innerHTML = `${escapeHtml(formatKeyName(matches[i]))}<span class="raw-name">${escapeHtml(matches[i])}</span>`;
            div.onmousedown = (e) => {
                e.preventDefault();
                e.stopPropagation();
                editKeys.push(matches[i]);
                input.value = '';
                renderTags();
                renderSuggestions([]);
                input.focus();
            };
            suggestions.appendChild(div);
        }
    }

    input.oninput = () => {
        renderSuggestions(filterKeys(input.value));
    };

    input.onkeydown = (e) => {
        const items = suggestions.querySelectorAll('.suggestion');
        if (e.key === 'ArrowDown') {
            e.preventDefault();
            highlightedIdx = Math.min(highlightedIdx + 1, items.length - 1);
            updateHighlight(items);
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            highlightedIdx = Math.max(highlightedIdx - 1, 0);
            updateHighlight(items);
        } else if (e.key === 'Enter') {
            e.preventDefault();
            if (highlightedIdx >= 0 && items[highlightedIdx]) {
                items[highlightedIdx].onmousedown(e);
            } else if (input.value === '') {
                setCurrentDialValue(direction, editKeys.length > 0 ? editKeys : null);
                closeAllEditors();
                renderDialSettings();
            }
        } else if (e.key === 'Escape') {
            e.preventDefault();
            closeAllEditors();
            renderDialSettings();
        } else if (e.key === 'Backspace' && input.value === '' && editKeys.length > 0) {
            editKeys.pop();
            renderTags();
        }
    };

    function updateHighlight(items) {
        items.forEach((el, i) => el.classList.toggle('highlighted', i === highlightedIdx));
        if (highlightedIdx >= 0 && items[highlightedIdx]) {
            items[highlightedIdx].scrollIntoView({ block: 'nearest' });
        }
    }

    renderTags();
    setTimeout(() => input.focus(), 0);

    setTimeout(() => input.focus(), 0);
}

function commitDialEdit() {
    if (editingDial === null) return;
    const row = document.querySelector('.dial-row.editing');
    if (row) {
        const tags = row.querySelectorAll('.key-editor-tags .key-tag');
        const keys = [];
        tags.forEach(tag => {
            const display = tag.childNodes[0].textContent.trim();
            const raw = availableKeys.find(k => formatKeyName(k) === display);
            if (raw) keys.push(raw);
        });
        setCurrentDialValue(editingDial, keys.length > 0 ? keys : null);
    }
    closeAllEditors();
    renderDialSettings();
}

// ── WM Class Editor ──────────────────────────────────
function renderWmClassSection() {
    const section = document.getElementById('wm-class-section');
    const tagsEl = document.getElementById('wm-class-tags');
    const input = document.getElementById('wm-class-input');

    if (currentProfile === 'default') {
        section.classList.add('hidden');
        return;
    }

    section.classList.remove('hidden');
    const profile = config.profiles[currentProfile];
    if (!profile) return;

    tagsEl.innerHTML = '';
    const wmClasses = profile.wm_class || [];

    for (let i = 0; i < wmClasses.length; i++) {
        const tag = document.createElement('span');
        tag.className = 'wm-tag';
        tag.innerHTML = `${escapeHtml(wmClasses[i])}<span class="remove-wm" data-idx="${i}">&times;</span>`;
        tag.querySelector('.remove-wm').onclick = () => {
            wmClasses.splice(i, 1);
            markDirty();
            renderWmClassSection();
        };
        tagsEl.appendChild(tag);
    }

    input.onkeydown = (e) => {
        if (e.key === 'Enter' && input.value.trim()) {
            e.preventDefault();
            wmClasses.push(input.value.trim());
            input.value = '';
            markDirty();
            renderWmClassSection();
        }
    };
}

// ── Dirty State ──────────────────────────────────────
function markDirty() {
    dirty = true;
    updateDirtyState();
}

function updateDirtyState() {
    const indicator = document.getElementById('change-indicator');
    const discardBtn = document.getElementById('btn-discard');
    indicator.classList.toggle('hidden', !dirty);
    discardBtn.disabled = !dirty;
}

// ── Save / Discard ───────────────────────────────────
async function saveConfig() {
    try {
        await invoke('save_config', { config: config });
        originalConfig = structuredClone(config);
        dirty = false;
        updateDirtyState();
        showToast('Configuration saved');
    } catch (err) {
        showToast('Save failed: ' + err, true);
    }
}

function discardChanges() {
    config = structuredClone(originalConfig);
    dirty = false;
    currentProfile = 'default';
    activeButtonHid = null;
    closeAllEditors();
    renderProfileSelector();
    renderButtonList();
    renderDialSettings();
    renderWmClassSection();
    updateDirtyState();
    updateOverlayHighlights();
}

function showToast(message, isError) {
    const toast = document.getElementById('toast');
    toast.textContent = message;
    toast.style.background = isError ? 'var(--danger)' : 'var(--success)';
    toast.classList.remove('hidden');
    toast.classList.add('show');
    setTimeout(() => {
        toast.classList.remove('show');
        setTimeout(() => toast.classList.add('hidden'), 300);
    }, 2000);
}

// ── Utilities ────────────────────────────────────────
function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}
