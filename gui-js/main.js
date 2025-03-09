const { app, BrowserWindow, dialog, ipcMain } = require('electron');
const fs = require('fs');
const crypto = require('crypto');
const path = require('path');

let mainWindow;

app.whenReady().then(() => {
    mainWindow = new BrowserWindow({
        width: 800,
        height: 600,
        webPreferences: {
            nodeIntegration: true,
            contextIsolation: false
        }
    });
    mainWindow.loadFile('index.html');
});

const KEYMAPS = {
    "A": [0x73FDA, 0x5E],
    "B": [0x7401E, 0x5F],
    "C": [0x73FEC, 0x5C],
    "D": [0x73FEA, 0x5D],
    "E": [0x73FE6, 0x52],
    "F": [0x7401A, 0x53],
    "G": [0x74012, 0x50],
    "H": [0x74022, 0x51],
    "I": [0x74036, 0x56],
    "J": [0x7402A, 0x57],
    "K": [0x7403A, 0x54],
    "L": [0x7404A, 0x55],
    "M": [0x7402C, 0x4A],
    "N": [0x7402E, 0x4B],
    "O": [0x74046, 0x48],
    "P": [0x74056, 0x49],
    "Q": [0x73FD6, 0x4E],
    "R": [0x74016, 0x4F],
    "S": [0x73FFA, 0x4C],
    "T": [0x74020, 0x4D],
    "U": [0x74026, 0x42],
    "V": [0x7401C, 0x43],
    "W": [0x73FF6, 0x40],
    "X": [0x73FFC, 0x41],
    "Y": [0x74030, 0x46],
    "Z": [0x73FDC, 0x47],
    "1": [0x73FD8, 0x44],
    "2": [0x73FF8, 0x45],
    "3": [0x73FE8, 0x7A],
    "4": [0x74018, 0x7B],
    "5": [0x74014, 0x78],
    "6": [0x74024, 0x79],
    "7": [0x74028, 0x7E],
    "8": [0x74038, 0x7F],
    "9": [0x74048, 0x7C],
    "0": [0x74058, 0x7D],
    "Return Enter": [0x7407C, 0x72],
    "Escape": [0x73FD2, 0x73],
    "Delete": [0x74080, 0x70],
    "Tab": [0x73FE0, 0x71],
    "Spacebar": [0x7407E, 0x76],
    "Dash and Underscore": [0x74054, 0x77],
    "Equals and Plus": [0x74034, 0x74],
    "Left Brace": [0x74060, 0x75],
    "Right Brace": [0x74040, 0x6A],
    "Pipe and Slash": [0x7407A, 0x6B],
    "Non-US": [0x7405C, 0x68],
    "SemiColon and Colon": [0x7405A, 0x69],
    "Apostrophe and Double Quotation Mark": [0x74052, 0x6E],
    "Grave Accent and Tilde": [0x73FD4, 0x6F],
    "Comma": [0x7403C, 0x6C],
    "Period": [0x7404C, 0x6D],
    "QuestionMark": [0x7405E, 0x62],
    "Caps Lock": [0x74000, 0x63],
    "F1": [0x73FF4, 0x60],
    "F2": [0x73FE4, 0x61],
    "F3": [0x73FF0, 0x66],
    "F4": [0x73FE2, 0x67],
    "F5": [0x74072, 0x64],
    "F6": [0x74032, 0x65],
    "F7": [0x74050, 0x1A],
    "F8": [0x74044, 0x1B],
    "F9": [0x74074, 0x18],
    "F10": [0x74078, 0x19],
    "F11": [0x74088, 0x1E],
    "F12": [0x740A8, 0x1F],
    "PrintScreen": [0x740CA, 0x1C],
    "Pause": [0x740BC, 0x12],
    "Insert": [0x740C8, 0x13],
    "Home": [0x74084, 0x10],
    "PageUp": [0x740CC, 0x11],
    "Delete Forward": [0x740C4, 0x16],
    "End": [0x740B8, 0x17],
    "PageDown": [0x740CE, 0x14],
    "RightArrow": [0x740AE, 0x15],
    "LeftArrow": [0x740BE, 0xA],
    "DownArrow": [0x7408E, 0xB],
    "UpArrow": [0x740B2, 0x8],
    "Non-US Slash Bar": [0x73FF2, 0x3E],
    "International1": [0x7403E, 0xDD],
    "International2": [0x7404E, 0xD2],
    "International3": [0x74076, 0xD3],
    "International4": [0x74042, 0xD0],
    "International5": [0x73FDE, 0xD1],
    "LeftControl": [0x74004, 0xBA],
    "LeftShift": [0x74070, 0xBB],
    "LeftAlt": [0x74092, 0xB8],
    "Left GUI": [0x740B0, 0xB9],
    "RightControl": [0x7400C, 0xBE],
    "RightShift": [0x7406C, 0xBF],
    "RightAlt": [0x7409E, 0xBC],
    "Mute": [0x74092, 0xB8],
    "Fn": [0x740BA, 0xF5],
    "System Power Down": [null, 0xDB],
    "System Sleep": [null, 0xD8],
    "System Wake Up": [null, 0xD9],
    "ErrorRollOver": [null, 0x5B],
    "Scroll Lock": [null, 0x1D],
    "Keypad Num Lock and Clear": [null, 0x9],
    "Keypad Forward Slash": [null, 0xE],
    "Keypad Star": [null, 0xF],
    "Keypad Dash": [null, 0xC],
    "Keypad Plus": [null, 0xD],
    "Keypad ENTER": [null, 0x2],
    "Keypad 1 and End": [null, 0x3],
    "Keypad 2 and Down Arrow": [null, 0x0],
    "Keypad 3 and PageDn": [null, 0x1],
    "Keypad 4 and Left Arrow": [null, 0x6],
    "Keypad 5": [null, 0x7],
    "Keypad 6 and Right Arrow": [null, 0x4],
    "Keypad 7 and Home": [null, 0x5],
    "Keypad 8 and Up Arrow": [null, 0x3A],
    "Keypad 9 and PageUp": [null, 0x3B],
    "Keypad 0 and Insert": [null, 0x38],
    "Keypad Period": [null, 0x39],
    "Application": [null, 0x3F],
    "Power": [null, 0x3C],
    "Keypad Equals": [null, 0x3D],
    "F13": [null, 0x32],
    "F14": [null, 0x33],
    "F15": [null, 0x30],
    "F16": [null, 0x31],
    "F17": [null, 0x36],
    "F18": [null, 0x37],
    "F19": [null, 0x34],
    "F20": [null, 0x35],
    "F21": [null, 0x2A],
    "F22": [null, 0x2B],
    "F23": [null, 0x28],
    "F24": [null, 0x29],
    "Keypad Comma": [null, 0xDF],
    "International6": [null, 0xD6],
    "LANG1": [null, 0xCA],
    "LANG2": [null, 0xCB],
    "LANG3": [null, 0xC8],
    "LANG4": [null, 0xC9],
    "LANG5": [null, 0xCE],
    "Right GUI": [null, 0xBD],
    "Scan Next Track": [null, 0xEF],
    "Scan Previous Track": [null, 0xEC],
    "Stop": [null, 0xED],
    "Play/Pause": [null, 0x97],
    "Volume Increment": [null, 0xB3],
    "Volume Decrement": [null, 0xB0],
};

ipcMain.handle('select-original-firmware', async () => {
    const result = await dialog.showOpenDialog({
        properties: ['openFile'],
        filters: [{ name: 'Executable', extensions: ['exe'] }]
    });
    if (result.canceled) return null;
    return result.filePaths[0];
});

ipcMain.handle('compute-sha256', async (event, filePath) => {
    const data = fs.readFileSync(filePath);
    return crypto.createHash('sha256').update(data).digest('hex');
});

ipcMain.handle('select-json', async () => {
    const result = await dialog.showOpenDialog({
        properties: ['openFile'],
        filters: [{ name: 'JSON', extensions: ['json'] }]
    });
    if (result.canceled) return null;
    return result.filePaths[0];
});

ipcMain.handle('read-json', async (event, filePath) => {
    try {
        const jsonData = fs.readFileSync(filePath, 'utf8');
        return JSON.parse(jsonData);
    } catch (error) {
        return { error: 'Invalid JSON format!' };
    }
});

ipcMain.handle('modify-firmware', async (event, firmwarePath, jsonPath) => {
    const saveResult = await dialog.showSaveDialog({
        filters: [{ name: 'Executable', extensions: ['exe'] }]
    });
    if (saveResult.canceled) return null;
    
    const savePath = saveResult.filePath;
    const keyRemap = JSON.parse(fs.readFileSync(jsonPath, 'utf8'));
    let data = fs.readFileSync(firmwarePath);
    let buffer = Buffer.from(data);

    for (const [sourceKey, targetKey] of Object.entries(keyRemap)) {
        if (!(sourceKey in KEYMAPS) || !(targetKey in KEYMAPS)) {
            return { error: `Key ${sourceKey} or ${targetKey} not found in keymap!` };
        }
        
        const [byteOffset, expectedScanCode] = KEYMAPS[sourceKey];
        const [, newScanCode] = KEYMAPS[targetKey];

        if (buffer[byteOffset] !== expectedScanCode) {
            return { error: `Unexpected value at ${byteOffset.toString(16)}. Expected ${expectedScanCode.toString(16)}, found ${buffer[byteOffset].toString(16)}.` };
        }
        buffer[byteOffset] = newScanCode;
    }
    
    fs.writeFileSync(savePath, buffer);
    const modifiedSHA256 = crypto.createHash('sha256').update(buffer).digest('hex');
    return { savePath, modifiedSHA256 };
});
