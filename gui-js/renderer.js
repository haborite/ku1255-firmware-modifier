const { ipcRenderer } = require('electron');

document.getElementById('selectOriginal').addEventListener('click', async () => {
    const filePath = await ipcRenderer.invoke('select-original-firmware');
    if (filePath) {
        document.getElementById('originalPath').innerText = `Original Firmware: ${filePath}`;
        const sha256 = await ipcRenderer.invoke('compute-sha256', filePath);
        document.getElementById('originalSha').innerText = sha256;
    }
});

document.getElementById('selectJson').addEventListener('click', async () => {
    const filePath = await ipcRenderer.invoke('select-json');
    if (filePath) {
        document.getElementById('jsonPath').innerText = `Key Remap JSON: ${filePath}`;
        const jsonData = await ipcRenderer.invoke('read-json', filePath);
        if (jsonData.error) {
            alert(jsonData.error);
        } else {
            const formattedJson = Object.entries(jsonData).map(([key, value]) => `${key} --> ${value}`).join('\n');
            document.getElementById('jsonContent').innerText = formattedJson;
        }
    }
});

document.getElementById('modifyFirmware').addEventListener('click', async () => {
    const firmwarePath = document.getElementById('originalPath').innerText.replace('Original Firmware: ', '');
    const jsonPath = document.getElementById('jsonPath').innerText.replace('Key Remap JSON: ', '');
    
    if (!firmwarePath || !jsonPath) {
        alert('Please select both firmware and JSON files first.');
        return;
    }
    
    const result = await ipcRenderer.invoke('modify-firmware', firmwarePath, jsonPath);
    if (result.error) {
        alert(result.error);
    } else {
        document.getElementById('modifiedSha').innerText = result.modifiedSHA256;
        alert('Firmware modified successfully!');
    }
});
