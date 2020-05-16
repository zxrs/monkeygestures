let port = browser.runtime.connectNative("MonkeyGestures");

let shouldSuppressContextMenu = false;

port.onMessage.addListener(async (direction) => {
    console.log(direction);
    shouldSuppressContextMenu = true;
    switch (direction) {
        case "DR":
            let tabs = await browser.tabs.query({ active: true, windowId: browser.windows.WINDOW_ID_CURRENT });
            let tab = await browser.tabs.get(tabs[0].id);
            await browser.tabs.remove(tab.id)
            break;
        case "L":
            browser.tabs.executeScript(this.id, {
                code: 'history.back();',
                runAt: 'document_start'
            });            break;
        case "R":
            browser.tabs.executeScript(this.id, {
                code: 'history.forward();',
                runAt: 'document_start'
            });            break;
        case "W+":
            break;
        case "W-":
            break;
    }
});

browser.menus.onShown.addListener((info, tab) => {
    console.log(shouldSuppressContextMenu);
    if (shouldSuppressContextMenu) {
        port.postMessage("suppressContextMenu");
    }
});

browser.menus.onHidden.addListener((info, tab) => {
    shouldSuppressContextMenu = false;
});