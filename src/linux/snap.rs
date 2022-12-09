use serde::{Serialize,Deserialize};

#[derive(Serialize, Deserialize)]
struct Snap {
    name: String,
    version: String,
    summary: String,
    description: String,
    confinement: Confinement,
    architectures: Vec<String>,
    apps: Vec<App>,
    plugs: Vec<Plug>,
}

#[derive(Serialize, Deserialize)]
struct Confinement {
    value: String,
}

#[derive(Serialize, Deserialize)]
struct App {
    name: String,
    command: String,
    plugs: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Plug {
    name: String,
    interface: String,
    attrs: Vec<Attr>,
}

#[derive(Serialize, Deserialize)]
struct Attr {
    key: String,
    value: String,
}

pub fn create_snap_data(ctx: &Context) -> Snap {
    let mut snap = Snap {
        name: ctx.manifest.application.title, 
        version: ctx.manifest.application.version,
        summary: ctx.manifest.application.description,
        description: String::new(),
        confinement: Confinement {
            value: String::new(),
        },
        architectures: Vec::new(),
        apps: Vec::new(),
        plugs: Vec::new(),
    };

    snap

}