export function createStatus() {
    return () => {};
}

export function initUI(setStatus, playbackControllers) {
    document.querySelectorAll('.tab-btn').forEach(button => {
        button.addEventListener('click', async () => {
            document.querySelectorAll('.tab-btn').forEach(item => item.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(item => item.classList.remove('active'));
            button.classList.add('active');
            document.getElementById(button.dataset.target)?.classList.add('active');
            playbackControllers.forEach(controller => controller.stopTimer());
            try {
                const { invoke } = await import('./tauri.js');
                await invoke('stop_playback');
            } catch (error) {
                setStatus(error.message);
            }
        });
    });

    document.querySelectorAll('[data-export-format]').forEach(select => {
        const update = () => showFormatSettings(select);
        select.addEventListener('change', update);
        update();
    });
}

function showFormatSettings(select) {
    const container = select.closest('.export-grid')?.querySelector('.export-settings');
    if (!container) {
        return;
    }
    container.querySelectorAll('[data-format-settings]').forEach(element => {
        element.classList.toggle('active', element.dataset.formatSettings === select.value);
    });
}
