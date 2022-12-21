nw.Window.open('index.html', {
    //new_instance: true,
    id: 'main',
    title: 'NW Background Test',
    width: 1027,
    height: 768,
    resizable: true,
    frame: true,
    transparent: false,
    show: true,
    // icon: 'icons/application.png'
    // http://docs.nwjs.io/en/latest/References/Manifest%20Format/#window-subfields
}, (win) => {
    console.log("window", win)
});
