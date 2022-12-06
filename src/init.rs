use crate::prelude::*;

pub struct Project {
    name : String,
    title : String,
}

impl Project {
    pub fn try_new() -> Result<Project> {
        let project = Project {
            name : "default".into(),
            title : "Default".into(),
        };

        Ok(project)
    }

    pub fn generate(&self) -> Result<()> {

        // ^ TODO - create files

        Ok(())
    }
}
