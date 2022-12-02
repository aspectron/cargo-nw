nw.Window.open('index.html', {
    //new_instance: true,
    id: 'main',
    title: 'NWJS Example',
    width: 1027,
    height: 768,
    resizable: true,
    frame: true,
    transparent: false,
    show: true,
    // icon: 'resources/images/kdx-icon.png'
    // http://docs.nwjs.io/en/latest/References/Manifest%20Format/#window-subfields
}, (win) => {
    console.log("win", win)
});
