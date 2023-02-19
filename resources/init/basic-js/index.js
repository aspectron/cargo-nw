
nw.Window.open('app/index.html', {
    id: '___SNAKE___',
    title: '___TITLE___',
    width: 1027,
    height: 768,
    resizable: true,
    frame: false,
    transparent: false,
    show: true,
    icon: 'resources/icons/default-application-icon.png'
    // http://docs.nwjs.io/en/latest/References/Manifest%20Format/#window-subfields
}, (win) => {
    // console.log("index window:", win)
    resolve();
});
