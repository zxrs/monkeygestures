let port = browser.runtime.connectNative("MonkeyGestures");

let directionChain = [];

port.onMessage.addListener(async (direction) => {
    if (direction == "?") {
        // Start Gesture
        directionChain = [];
    } else if (direction == "!") {
        // Stop Gesture
        let direction = directionChain.join("");
        switch (direction) {
            case "DR":
                {
                    // Close current tab
                    let tab = await getCurrentTab();
                    await browser.tabs.remove(tab.id);
                }
                break;
            case "L":
                {
                    // Go back
                    let tab = await getCurrentTab();
                    browser.tabs.executeScript(tab.id, {
                        code: 'history.back();',
                        runAt: 'document_start'
                    });
                }
                break;
            case "R":
                {
                    // Go forward.
                    let tab = await getCurrentTab();
                    browser.tabs.executeScript(tab.id, {
                        code: 'history.forward();',
                        runAt: 'document_start'
                    });
                }
                break;
        }
    } else {
        // Progress Gesture
        let lastDirection = directionChain[directionChain.length - 1];
        if (direction != lastDirection) {
            directionChain.push(direction);
        }
        // Wheel Gesture
        if (direction == "+" || direction == "-") {
            let tabs = await browser.tabs.query({
                currentWindow: true,
                active: false,
                hidden: false,
                discarded: false
            });
            let current_tab = await getCurrentTab();
            let nextTab;
            // +: Select left tab
            if (direction == "+") {
                if (tabs.some(cur => cur.index < current_tab.index)) {
                    nextTab = tabs.reduce((acc, cur) =>
                        (acc.index >= current_tab.index && cur.index < acc.index) || (cur.index < current_tab.index && cur.index > acc.index) ? cur : acc
                    );
                } else {
                    nextTab = tabs.reduce((acc, cur) => acc.index > cur.index ? acc : cur);
                }
            } else {
            // -: Select right tab
                if (tabs.some(cur => cur.index > current_tab.index)) {
                    nextTab = tabs.reduce((acc, cur) =>
                        (acc.index <= current_tab.index && cur.index > acc.index) || (cur.index > current_tab.index && cur.index < acc.index) ? cur : acc
                    );
                } else {
                    nextTab = tabs.reduce((acc, cur) => acc.index < cur.index ? acc : cur);
                }
            }
            if (nextTab) {
                browser.tabs.update(nextTab.id, { active: true });
            }
        }
    }
});

async function getCurrentTab() {
    let tabs = await browser.tabs.query({ active: true, windowId: browser.windows.WINDOW_ID_CURRENT });
    return await browser.tabs.get(tabs[0].id);
}

browser.menus.onShown.addListener((info, tab) => {
    if (directionChain.length > 0) {
        port.postMessage("suppressContextMenu");
    }
});
