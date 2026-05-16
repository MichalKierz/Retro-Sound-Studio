export function invoke(command, args = {}) {
    const api = window.__TAURI__;
    const invoker = api?.tauri?.invoke || api?.invoke;
    if (!invoker) {
        return Promise.reject(new Error('Tauri API is not available'));
    }
    return invoker(command, args);
}

export function openFile(options = {}) {
    const opener = window.__TAURI__?.dialog?.open;
    if (!opener) {
        return Promise.reject(new Error('Tauri file dialog is not available'));
    }
    return opener(options);
}

export function saveFile(options = {}) {
    const saver = window.__TAURI__?.dialog?.save;
    if (!saver) {
        return Promise.reject(new Error('Tauri save dialog is not available'));
    }
    return saver(options);
}

export function confirmDialog(message, title = 'Confirm') {
    const confirmer = window.__TAURI__?.dialog?.confirm;
    if (confirmer) {
        return confirmer(message, { title, type: 'warning' });
    }
    return Promise.resolve(window.confirm(message));
}

export function openDirectory(defaultPath = undefined) {
    return openFile({ directory: true, multiple: false, defaultPath });
}

export function listen(event, handler) {
    const listener = window.__TAURI__?.event?.listen;
    if (!listener) {
        return Promise.resolve(() => {});
    }
    return listener(event, handler);
}
